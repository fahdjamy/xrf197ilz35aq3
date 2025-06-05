use crate::context::{ApplicationContext, UserContext};
use crate::core::EntryType::Credit;
use crate::core::{Account, AccountType, Block, BlockRegion, Currency, EntryType, WalletHolding};
use crate::error::OrchestrateError;
use crate::orchestrator::{create_ledger, create_wallet_holding};
use crate::storage::{save_account, save_block_chain, PreparedAppStatements};
use crate::{
    commit_db_transaction, create_activity, create_chain_stamp, rollback_db_transaction,
    start_db_transaction, DomainError, CREATE_NEW_USER_ACCOUNT,
};
use cassandra_cpp::Session;
use sqlx::{PgConnection, PgPool};
use std::str::FromStr;
use tracing::log;

pub async fn create_account(
    pool: &PgPool,
    currency: String,
    acct_type: String,
    user_ctx: UserContext,
    cassandra_session: Session,
    app_cxt: ApplicationContext,
    statements: PreparedAppStatements,
) -> Result<(Account, WalletHolding), OrchestrateError> {
    let block_region = BlockRegion::from_str(&app_cxt.region)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

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

    ////// 2. Create the ledger to keep track of the entire transaction [wallet, block, ledger].
    // Should have a ledger for record keeping
    let description = Some("initialization for newly created account".to_string());
    let ledger = create_ledger(&mut *db_tx, Credit, new_acct.id.clone(), description).await?;

    ////// 3. create a wallet that belongs to the account
    let wallet_holding = if let Some(wallet) =
        create_wallet_holding(&mut *db_tx, new_acct.id.clone(), ledger.id.clone()).await?
    {
        wallet
    } else {
        rollback_db_transaction(db_tx, event).await?;
        return Err(OrchestrateError::ServerError(
            "failed to create wallet holding".to_string(),
        ));
    };

    let mut entry_ids = Vec::new();
    entry_ids.push(ledger.id.clone());

    ////// 4. Create a chain_stamp for blocks that group ledger entries.
    let chain_stamp = create_chain_stamp(&mut db_tx, None).await?;

    //// Create a block for ledger-entry grouping. This block will contain the root chain_stamp
    let block = Block::build(
        app_cxt.app_id.to_string(),
        block_region,
        entry_ids,
        chain_stamp.stamp.clone(),
    )
    .map_err(|err| match err {
        DomainError::ParseError(er) => OrchestrateError::InvalidArgument(er),
        DomainError::InvalidArgument(er) => OrchestrateError::InvalidArgument(er),
        DomainError::InvalidState(er) => {
            log::error!("invalid record/row state: {}", er);
            OrchestrateError::ServerError(er)
        }
    })?;

    ///// 5. Create an activity to group create an account and wallet holding task

    match create_activity(
        &mut *db_tx,
        block.id.clone(),
        chain_stamp.stamp.clone(),
        CREATE_NEW_USER_ACCOUNT.to_string(),
        &user_ctx,
    )
    .await?
    {
        Some(created_activity) => {
            log::info!("activity created: {:?}", created_activity);
        }
        None => {
            rollback_db_transaction(db_tx, event).await?;
            return Err(OrchestrateError::ServerError(
                "failed to create activity".to_string(),
            ));
        }
    }

    ///// 6 save block to cassandra DB
    let block_saved = save_block_chain(block, cassandra_session, statements.insert_block_stmt)
        .await
        .map_err(|err| {
            log::error!("failed to save block to cassandra DB: {}", err);
            OrchestrateError::ServerError(err.to_string())
        })?;

    if block_saved {
        commit_db_transaction(db_tx, event).await?;
    } else {
        rollback_db_transaction(db_tx, event).await?;
        return Err(OrchestrateError::ServerError(
            "failed to save block to DB".to_string(),
        ));
    }

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
