use crate::context::{ApplicationContext, RequestContext, UserContext};
use crate::core::{
    Account, AccountStatus, AccountType, AuditEventType, AuditLog, Currency, EntityType,
    UpdateAccountReq, WalletHolding,
};
use crate::core::{BeneficiaryAccount, EntryType};
use crate::error::OrchestrateError;
use crate::orchestrator::create_wallet_holding;
use crate::storage::{
    fetch_user_accounts_by_currencies_and_types, fetch_user_wallets, find_account_by_acct_type,
    find_account_by_currency_and_acct_type, find_account_by_id, save_account,
    save_beneficiary_account,
};
use crate::{
    commit_db_transaction, create_initial_block_chain, create_new_audit,
    find_user_wallets_for_acct, rollback_db_transaction, start_db_transaction,
};
use cassandra_cpp::{PreparedStatement, Session};
use config::Map;
use sqlx::{PgConnection, PgPool};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{error, info};

pub async fn create_account(
    pool: &PgPool,
    currency: String,
    acct_type: String,
    user_ctx: &UserContext,
    cassandra_session: &Session,
    app_cxt: &ApplicationContext,
) -> Result<(Account, WalletHolding), OrchestrateError> {
    let event = "createAccount";
    let mut db_tx = start_db_transaction(pool, event).await?;

    let acct_type = AccountType::from_str(&acct_type)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    let curr = Currency::from_str(&currency)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    let mut ledger_description = "initialization for newly created account".to_string();

    let created_or_saved_acct =
        match find_account_by_acct_type(&mut *db_tx, &user_ctx.user_fp, acct_type.clone()).await? {
            Some(saved_acct) => {
                if saved_acct.currency == curr {
                    return Err(OrchestrateError::RecordAlreadyExists(
                        "account already exists".to_string(),
                    ));
                }
                ledger_description = "creating another wallet for an existing account".to_string();
                saved_acct
            }
            None => {
                if let Some(acct) = create_new_acct(&mut db_tx, curr, acct_type, &user_ctx).await? {
                    acct
                } else {
                    rollback_db_transaction(db_tx, event).await?;
                    return Err(OrchestrateError::ServerError(
                        "failed to create new account".to_string(),
                    ));
                }
            }
        };

    ////// 3. create a wallet that belongs to the account
    let wallet_holding = if let Some(wallet) = create_wallet_holding(
        &mut *db_tx,
        created_or_saved_acct.id.clone(),
        created_or_saved_acct.currency.clone(),
    )
    .await?
    {
        wallet
    } else {
        rollback_db_transaction(db_tx, event).await?;
        return Err(OrchestrateError::ServerError(
            "failed to create wallet holding".to_string(),
        ));
    };

    let mut ledger_desc = Vec::new();
    ledger_desc.push(ledger_description);

    let block = match create_initial_block_chain(
        created_or_saved_acct.id.clone(),
        EntryType::Initialization,
        user_ctx,
        cassandra_session,
        &app_cxt,
        ledger_desc,
        &app_cxt.statements.insert_block_stmt,
        &mut db_tx,
    )
    .await
    {
        Ok(block) => {
            commit_db_transaction(db_tx, event).await?;
            block
        }
        Err(err) => {
            error!("failed to create blockchain: {}", err);
            rollback_db_transaction(db_tx, event).await?;
            return Err(err.into());
        }
    };
    info!(
        "created new account with id: {} and blockId: {}",
        created_or_saved_acct.id, block.id
    );

    Ok((created_or_saved_acct, wallet_holding))
}

async fn create_new_acct(
    tx: &mut PgConnection,
    currency: Currency,
    acct_type: AccountType,
    user_ctx: &UserContext,
) -> Result<Option<Account>, OrchestrateError> {
    ////// 1. create an account
    let account = Account::new(
        user_ctx.user_fp.clone(),
        user_ctx.timezone.clone(),
        currency,
        acct_type.clone(),
    );

    ///// 1.1.1 check if there's an existing account for user with this account type
    let saved_acct = find_account_by_acct_type(&mut *tx, &user_ctx.user_fp, acct_type).await?;
    if saved_acct.is_some() {
        return Ok(saved_acct);
    }

    ///// 1.1 Save the new account to DB
    let acct_created = save_account(tx, &account).await?;
    if !acct_created {
        return Ok(None);
    }

    Ok(Some(account))
}

pub async fn update_user_account(
    pool: &PgPool,
    acct_id: &str,
    user_ctx: &UserContext,
    request: UpdateAccountReq,
    req_context: RequestContext,
) -> Result<bool, OrchestrateError> {
    if !is_valid_request(&request) {
        return Err(OrchestrateError::InvalidArgument(
            "invalid_request".to_string(),
        ));
    }
    let event = "updateAccount";
    let mut db_tx = start_db_transaction(pool, event).await?;
    let saved_acct = match find_account_by_id(&mut *db_tx, acct_id).await? {
        Some(saved_account) => {
            // Owners are the only ones allowed to update their accounts for now.
            if saved_account.user_fp != user_ctx.user_fp {
                return Err(OrchestrateError::NotFoundError(
                    "account not found".to_string(),
                ));
            }
            saved_account
        }
        None => {
            return Err(OrchestrateError::NotFoundError(
                "account not found".to_string(),
            ));
        }
    };
    if saved_acct.status == AccountStatus::Frozen {
        return Err(OrchestrateError::IllegalState(
            "account is frozen".to_string(),
        ));
    }
    if request.locked.is_none() && saved_acct.locked {
        return Err(OrchestrateError::IllegalState(
            "Can not make updates to a locked account".to_string(),
        ));
    }

    let updated_acct_details = update_account_mapper(saved_acct.clone(), &request);
    if !save_account(&mut *db_tx, &updated_acct_details).await? {
        // roll back if the account is not updated
        rollback_db_transaction(db_tx, event).await?;
    }

    // create audit log
    let audit_log = AuditLog::build(
        user_ctx.user_fp,
        acct_id.to_string(),
        EntityType::Account,
        AuditEventType::UPDATE,
        req_context.request_ip.clone(),
        req_context.request_ip,
        req_context.user_agent,
        Some(saved_acct),
        Some(updated_acct_details),
    )
    .map_err(|err| OrchestrateError::ServerError(format!("failed to build audit log: {}", err)))?;
    if !create_new_audit(audit_log, &mut **db_tx).await? {
        // roll back if the account audit log is not created
        rollback_db_transaction(db_tx, event).await?;
    };

    commit_db_transaction(db_tx, event).await?;
    Ok(true)
}

fn update_account_mapper(mut account: Account, update_req: &UpdateAccountReq) -> Account {
    if update_req.locked.is_some() {
        account.locked = update_req.locked.clone().unwrap()
    }
    if update_req.account_type.is_some() {
        account.account_type = update_req.account_type.clone().unwrap()
    }
    if update_req.timezone.is_some() {
        account.timezone = update_req.timezone.clone().unwrap()
    }
    if update_req.status.is_some() {
        account.status = update_req.status.clone().unwrap()
    }
    account
}

pub async fn find_account_by_currency_and_type(
    pool: &PgPool,
    currency: &str,
    acct_type: &str,
) -> Result<Option<(Account, Vec<WalletHolding>)>, OrchestrateError> {
    let currency = Currency::from_str(currency)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;
    let acct_type = AccountType::from_str(acct_type)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    match find_account_by_currency_and_acct_type(pool, currency, acct_type).await? {
        None => Ok(None),
        Some(account) => {
            let wallets = find_user_wallets_for_acct(pool, &account.id).await?;
            Ok(Some((account, wallets)))
        }
    }
}

pub async fn create_new_beneficiary_acct(
    pool: &PgPool,
    currency: &String,
    user_ctx: &UserContext,
    cassandra_session: &Session,
    app_cxt: &ApplicationContext,
    account_admins_fps: Vec<String>,
    account_holders_fps: Vec<String>,
    insert_block_stmt: &PreparedStatement,
) -> Result<Option<BeneficiaryAccount>, OrchestrateError> {
    let event = "createNewBeneficiaryAccount";
    if account_admins_fps.is_empty() {
        return Err(OrchestrateError::InvalidArgument(
            "account admins should not be empty".to_string(),
        ));
    }
    if account_holders_fps.is_empty() {
        return Err(OrchestrateError::InvalidArgument(
            "account holders should not be empty".to_string(),
        ));
    }

    let mut db_tx = start_db_transaction(pool, event).await?;

    let curr = Currency::from_str(&currency)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;
    let acct_type = AccountType::SystemFee;

    let beneficiary_acct = BeneficiaryAccount::new(
        Some(app_cxt.app_id.clone().to_string()),
        acct_type,
        account_admins_fps,
        account_holders_fps,
        Some(app_cxt.block_region.clone()),
    );

    //////// 1.1 Save the new account to DB
    let ben_acct_saved = save_beneficiary_account(&mut *db_tx, &beneficiary_acct).await?;
    if !ben_acct_saved {
        return Ok(None);
    }

    ////// 2. create a wallet that belongs to the account
    if let Some(wallet) =
        create_wallet_holding(&mut *db_tx, beneficiary_acct.id.clone(), curr).await?
    {
        wallet
    } else {
        rollback_db_transaction(db_tx, event).await?;
        return Err(OrchestrateError::ServerError(
            "failed to create wallet holding for a BEN account".to_string(),
        ));
    };

    let mut ledger_desc = Vec::new();
    ledger_desc.push("initialization for a newly created BEN account".to_string());

    let block = match create_initial_block_chain(
        beneficiary_acct.id.clone(),
        EntryType::Initialization,
        user_ctx,
        cassandra_session,
        app_cxt,
        ledger_desc,
        insert_block_stmt,
        &mut db_tx,
    )
    .await
    {
        Ok(block) => {
            commit_db_transaction(db_tx, event).await?;
            block
        }
        Err(err) => {
            error!("failed to create blockchain: {}", err);
            rollback_db_transaction(db_tx, event).await?;
            return Err(err.into());
        }
    };
    info!(
        "created new BEN account with id: {} and blockId: {}",
        beneficiary_acct.id, block.id
    );

    Ok(Some(beneficiary_acct))
}

pub async fn get_user_accounts_by_currencies_or_types(
    pool: &PgPool,
    currencies: &[String],
    acct_types: &[String],
    user_ctx: &UserContext,
) -> Result<Vec<(Account, Vec<WalletHolding>)>, OrchestrateError> {
    if currencies.is_empty() && acct_types.is_empty() {
        return Err(OrchestrateError::InvalidArgument(
            "no filter criteria specified to find user account".to_string(),
        ));
    }

    let query_currencies = currencies
        .iter()
        .map(|s| Currency::from_str(&s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;
    let query_acct_types = acct_types
        .iter()
        .map(|s| AccountType::from_str(&s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    let user_accounts = fetch_user_accounts_by_currencies_and_types(
        pool,
        &user_ctx.user_fp,
        &query_currencies,
        &query_acct_types,
    )
    .await?;

    let mut account_ids_to_fetch = Vec::new();
    user_accounts.iter().for_each(|account| {
        account_ids_to_fetch.push(account.id.clone());
    });

    let wallets = fetch_user_wallets(pool, &account_ids_to_fetch).await?;

    let mut user_wallet_holdings: Map<String, Vec<WalletHolding>> = HashMap::new();

    for wallet in wallets {
        user_wallet_holdings
            .entry(wallet.account_id.clone())
            .or_insert_with(Vec::new)
            .push(wallet);
    }

    let mut result: Vec<(Account, Vec<WalletHolding>)> = Vec::new();
    user_accounts
        .iter()
        .for_each(|account| match user_wallet_holdings.get(&account.id) {
            Some(wallets) => {
                result.push((account.clone(), wallets.clone()));
            }
            None => {
                error!(
                    ":::INVALID RECORD STATE:::: accountId {} does not have a wallet holding",
                    account.id
                );
            }
        });

    Ok(result)
}

pub async fn get_user_account_by_id(
    pool: &PgPool,
    account_id: &str,
    include_wallet: bool,
) -> Result<(Account, Vec<WalletHolding>), OrchestrateError> {
    let account = match find_account_by_id(pool, account_id).await? {
        Some(account) => account,
        None => {
            return Err(OrchestrateError::NotFoundError(
                "no account found".to_string(),
            ));
        }
    };
    if !include_wallet {
        return Ok((account, vec![]));
    }

    let wallets = find_user_wallets_for_acct(pool, account_id).await?;
    if wallets.is_empty() {
        return Err(OrchestrateError::InvalidRecordState(
            "no wallets exist for an existing account".to_string(),
        ));
    }

    Ok((account, wallets))
}

fn is_valid_request(request: &UpdateAccountReq) -> bool {
    if request.locked.is_none()
        && request.status.is_none()
        && request.timezone.is_none()
        && request.account_type.is_none()
    {
        return false;
    }
    true
}
