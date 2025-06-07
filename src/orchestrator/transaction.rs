use crate::context::{ApplicationContext, UserContext};
use crate::core::EntryType::Debit;
use crate::core::{Block, BlockRegion, MonetaryTransaction, TransactionType};
use crate::error::OrchestrateError;
use crate::storage::{
    find_chain_stamp_by_id, save_block_chain, save_monetary_tx, PreparedAppStatements,
};
use crate::{
    commit_db_transaction, create_activity, create_chain_stamp, create_ledger, debit_wallet,
    find_last_user_activity, rollback_db_transaction, start_db_transaction, DomainError,
    CREATE_NEW_USER_ACCOUNT,
};
use cassandra_cpp::Session;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use tracing::log;

pub async fn start_debit_transaction(
    pool: &PgPool,
    user_fp: &str,
    amount: String,
    tx_type: String,
    account_id: String,
    user_ctx: UserContext,
    cassandra_session: Session,
    app_cxt: ApplicationContext,
    statements: PreparedAppStatements,
) -> Result<MonetaryTransaction, OrchestrateError> {
    let event = "debitTransaction";
    let block_region = BlockRegion::from_str(&app_cxt.region)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;
    let mut db_tx = start_db_transaction(pool, event).await?;

    ////// 0. Validate user request
    let decimal_amount = Decimal::from_str(&amount).map_err(|e| {
        return OrchestrateError::InvalidArgument("cannot parse amount".to_string());
    })?;

    let transaction_type = TransactionType::from_str(&tx_type).map_err(|err| {
        return OrchestrateError::InvalidArgument(err.to_string());
    })?;

    if transaction_type.must_be_positive()
        && (decimal_amount.is_sign_negative() || decimal_amount.is_zero())
    {
        return Err(OrchestrateError::InvalidArgument(
            "amount cannot be less than or equal to 0 for tx".to_string(),
        ));
    }

    ////// Get last user activity

    let last_user_activity = match find_last_user_activity(&mut *db_tx, user_fp).await? {
        Some(last_user_activity) => last_user_activity,
        None => {
            // if no activity is found, then the wallet is not valid
            // at least the creation account activity should have been created.
            return Err(OrchestrateError::InvalidRecordState(
                "No user activity found".to_string(),
            ));
        }
    };

    ////// 1. Create ledgers for the debit transaction actions
    let description = Some("debit user wallet".to_string());
    let ledger = match create_ledger(&mut *db_tx, Debit, account_id.clone(), description).await {
        Ok(ledger) => ledger,
        Err(err) => {
            rollback_db_transaction(db_tx, event).await?;
            return Err(err);
        }
    };

    let mut entry_ids = Vec::new();
    entry_ids.push(ledger.id.clone());

    ////// 2. Create a chain_stamp to chain blocks together.
    ////// 2.1 Find last activity chain stamp which will be the parent.
    let parent_chain_stamp =
        match find_chain_stamp_by_id(&mut *db_tx, &last_user_activity.chain_id).await? {
            Some(chain_stamp) => chain_stamp,
            None => {
                rollback_db_transaction(db_tx, event).await?;
                return Err(OrchestrateError::InvalidRecordState(
                    "can not debit account with no prior activity".to_string(),
                ));
            }
        };

    ////// 2.2 Create a new chain stamp for this transaction.
    let chain_stamp = match create_chain_stamp(&mut db_tx, Some(parent_chain_stamp)).await {
        Ok(chain_stamp) => chain_stamp,
        Err(err) => {
            rollback_db_transaction(db_tx, event).await?;
            return Err(err);
        }
    };

    ///// 3 create a block
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

    ///// 4. Create an activity chain stamp and the block created

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

    let debit_tx = MonetaryTransaction::payment(decimal_amount, account_id.clone());
    debit_wallet(&mut db_tx, decimal_amount, account_id, "todo".to_string()).await?;

    let account_debited = save_monetary_tx(&mut *db_tx, &debit_tx).await?;
    if !account_debited {
        rollback_db_transaction(db_tx, event).await?;
        return Err(OrchestrateError::ServerError(
            "could not debit user account, try again later".to_string(),
        ));
    }

    ///// 5 save block to cassandra DB
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

    Ok(debit_tx)
}
