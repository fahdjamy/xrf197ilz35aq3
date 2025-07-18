use crate::context::{ApplicationContext, UserContext};
use crate::core::{Account, AccountType, Currency, WalletHolding};
use crate::core::{BeneficiaryAccount, EntryType};
use crate::error::OrchestrateError;
use crate::orchestrator::create_wallet_holding;
use crate::storage::{save_account, save_beneficiary_account};
use crate::{
    commit_db_transaction, create_initial_block_chain, rollback_db_transaction,
    start_db_transaction,
};
use cassandra_cpp::Session;
use sqlx::{PgConnection, PgPool};
use std::str::FromStr;
use tracing::{error, info};

pub async fn create_account(
    pool: &PgPool,
    currency: String,
    acct_type: String,
    user_ctx: UserContext,
    cassandra_session: Session,
    app_cxt: ApplicationContext,
) -> Result<(Account, WalletHolding), OrchestrateError> {
    let event = "createAccount";
    let mut db_tx = start_db_transaction(pool, event).await?;

    let new_acct =
        if let Some(acct) = create_new_acct(&mut db_tx, &currency, &acct_type, &user_ctx).await? {
            acct
        } else {
            rollback_db_transaction(db_tx, event).await?;
            return Err(OrchestrateError::ServerError(
                "failed to create new account".to_string(),
            ));
        };

    ////// 3. create a wallet that belongs to the account
    let wallet_holding = if let Some(wallet) =
        create_wallet_holding(&mut *db_tx, new_acct.id.clone(), new_acct.currency.clone()).await?
    {
        wallet
    } else {
        rollback_db_transaction(db_tx, event).await?;
        return Err(OrchestrateError::ServerError(
            "failed to create wallet holding".to_string(),
        ));
    };

    let mut ledger_desc = Vec::new();
    ledger_desc.push("initialization for newly created account".to_string());

    let block = match create_initial_block_chain(
        new_acct.id.clone(),
        EntryType::Initialization,
        user_ctx,
        cassandra_session,
        app_cxt,
        ledger_desc,
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
        new_acct.id, block.id
    );

    Ok((new_acct, wallet_holding))
}

async fn create_new_acct(
    tx: &mut PgConnection,
    currency: &String,
    acct_type: &String,
    user_ctx: &UserContext,
) -> Result<Option<Account>, OrchestrateError> {
    let curr = Currency::from_str(&currency)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    let acct_type = AccountType::from_str(&acct_type)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    ////// 1. create an account
    let account = Account::new(
        user_ctx.user_fp.clone(),
        user_ctx.timezone.clone(),
        curr,
        acct_type,
    );

    //////// 1.1 Save the new account to DB
    let acct_created = save_account(tx, &account).await?;
    if !acct_created {
        return Ok(None);
    }

    Ok(Some(account))
}

pub async fn create_new_beneficiary_acct(
    pool: &PgPool,
    currency: &String,
    user_ctx: UserContext,
    cassandra_session: Session,
    app_cxt: ApplicationContext,
    account_admins_fps: Vec<String>,
    account_holders_fps: Vec<String>,
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
