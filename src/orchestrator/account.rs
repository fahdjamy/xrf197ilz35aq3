use crate::context::{ApplicationContext, UserContext};
use crate::core::EntryType;
use crate::core::{Account, AccountType, Currency, WalletHolding};
use crate::error::OrchestrateError;
use crate::orchestrator::create_wallet_holding;
use crate::storage::save_account;
use crate::{
    commit_db_transaction, create_block_chain, rollback_db_transaction, start_db_transaction,
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
    let wallet_holding =
        if let Some(wallet) = create_wallet_holding(&mut *db_tx, new_acct.id.clone()).await? {
            wallet
        } else {
            rollback_db_transaction(db_tx, event).await?;
            return Err(OrchestrateError::ServerError(
                "failed to create wallet holding".to_string(),
            ));
        };

    let mut ledger_desc = Vec::new();
    ledger_desc.push("initialization for newly created account".to_string());

    let block = match create_block_chain(
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
