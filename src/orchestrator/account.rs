use crate::context::{ApplicationContext, UserContext};
use crate::core::{Account, AccountType, Block, BlockRegion, Currency, EntryType, WalletHolding};
use crate::error::OrchestrateError;
use crate::orchestrator::ledger::create_ledger;
use crate::storage::save_account;
use crate::{ChainStamp, DomainError, PgDatabaseError};
use sqlx::{PgConnection, PgPool};
use std::str::FromStr;
use tracing::log;

pub async fn create_account(
    pool: &PgPool,
    currency: String,
    acct_type: String,
    user_ctx: UserContext,
    app_cxt: ApplicationContext,
) -> Result<(Account, WalletHolding), OrchestrateError> {
    let block_region = BlockRegion::from_str(&app_cxt.region)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    let mut transaction = pool.begin().await.map_err(|err| {
        log::error!(
            "failed to begin transaction for creating a new account: {}",
            err
        );
        OrchestrateError::ServerError(err.to_string())
    })?;

    let new_acct = if let Some(acct) =
        create_new_acct(&mut transaction, &currency, &acct_type, user_ctx).await?
    {
        acct
    } else {
        transaction.rollback().await.map_err(|err| {
            log::error!(
                "failed to rollback transaction for creating a new account: {}",
                err
            );
            OrchestrateError::DatabaseError(PgDatabaseError::TransactionStepError(format!(
                "failed to rollback transaction for creating a new account: {}",
                err
            )))
        })?;

        return Err(OrchestrateError::ServerError(
            "failed to create new account".to_string(),
        ));
    };

    ////// 2. create a wallet that belongs to the account
    let wallet_holding = WalletHolding::new(new_acct.id.clone());

    ////// 3. Create the initialization transaction. Should have a ledger for record keeping
    let description = Some("initialization for newly created account".to_string());
    let ledger = create_ledger(
        pool,
        EntryType::Credit.to_string(),
        new_acct.id.clone(),
        description,
    )
    .await?;

    let mut entry_ids = Vec::new();
    entry_ids.push(ledger.id.clone());

    ////// 4. Create a block for ledger-entry grouping. This block will contain the root chain_stamp
    let _ = Block::build(
        app_cxt.app_id.to_string(),
        block_region,
        entry_ids,
        ChainStamp::build(None),
    )
    .map_err(|err| match err {
        DomainError::ParseError(er) => OrchestrateError::InvalidArgument(er),
        DomainError::InvalidArgument(er) => OrchestrateError::InvalidArgument(er),
        DomainError::InvalidState(er) => {
            log::error!("invalid record/row state: {}", er);
            OrchestrateError::ServerError(er)
        }
    })?;

    Ok((new_acct, wallet_holding))
}

async fn create_new_acct(
    tx: &mut PgConnection,
    currency: &String,
    acct_type: &String,
    user_ctx: UserContext,
) -> Result<Option<Account>, OrchestrateError> {
    let curr = Currency::from_str(&currency)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    let acct_type = AccountType::from_str(&acct_type)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    ////// 1. create an account
    let account = Account::new(user_ctx.user_fp, user_ctx.timezone, curr, acct_type);

    //////// 1.1 Save the new account to DB
    let acct_created = save_account(tx, &account).await?;
    if !acct_created {
        return Ok(None);
    }

    Ok(Some(account))
}
