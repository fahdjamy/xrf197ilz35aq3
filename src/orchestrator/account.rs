use crate::context::{ApplicationContext, UserContext};
use crate::core::{Account, AccountType, Currency, WalletHolding};
use crate::core::{BeneficiaryAccount, EntryType};
use crate::error::OrchestrateError;
use crate::orchestrator::create_wallet_holding;
use crate::storage::{
    fetch_user_accounts_by_currencies_and_types, fetch_user_wallets, find_account_by_acct_type,
    find_account_by_currency_and_acct_type, find_account_by_id, save_account,
    save_beneficiary_account,
};
use crate::{
    commit_db_transaction, create_initial_block_chain, find_user_wallet_for_acct,
    rollback_db_transaction, start_db_transaction,
};
use cassandra_cpp::{PreparedStatement, Session};
use config::Map;
use sqlx::{PgConnection, PgPool};
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
                    return Err(OrchestrateError::RowConstraintViolation(
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

pub async fn find_user_acct_by_id(
    pool: &PgPool,
    account_id: &str,
) -> Result<Option<Account>, OrchestrateError> {
    match find_account_by_id(pool, account_id).await? {
        None => Ok(None),
        Some(account) => Ok(Some(account)),
    }
}

pub async fn find_account_by_currency_and_type(
    pool: &PgPool,
    currency: &str,
    acct_type: &str,
) -> Result<Option<(Account, WalletHolding)>, OrchestrateError> {
    let currency = Currency::from_str(currency)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;
    let acct_type = AccountType::from_str(acct_type)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    match find_account_by_currency_and_acct_type(pool, currency, acct_type).await? {
        None => Ok(None),
        Some(account) => match find_user_wallet_for_acct(pool, &account.id).await? {
            None => Ok(None),
            Some(saved_wallet) => Ok(Some((account, saved_wallet))),
        },
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
) -> Result<Vec<(Account, WalletHolding)>, OrchestrateError> {
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

    let user_wallet_holdings: Map<String, WalletHolding> =
        fetch_user_wallets(pool, &account_ids_to_fetch)
            .await?
            .iter()
            .map(|wallet| (wallet.account_id.clone(), wallet.clone()))
            .collect();

    let mut result: Vec<(Account, WalletHolding)> = Vec::new();
    user_accounts
        .iter()
        .for_each(|account| match user_wallet_holdings.get(&account.id) {
            Some(wallet_holding) => {
                result.push((account.clone(), wallet_holding.clone()));
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
