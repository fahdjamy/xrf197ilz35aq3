use crate::context::{ApplicationContext, UserContext};
use crate::core::{Block, EntryType, LedgerEntry};
use crate::error::OrchestrateError;
use crate::storage::{bulk_save_ledger, find_chain_stamp_by_id, save_block_chain};
use crate::{
    create_activity, create_chain_stamp, find_last_user_activity, DomainError,
    CREATE_NEW_USER_ACCOUNT,
};
use cassandra_cpp::Session;
use sqlx::{Postgres, Transaction};
use tracing::log;

pub async fn create_block_chain(
    acct_id: String,
    entry: EntryType,
    user_ctx: UserContext,
    cassandra_session: Session,
    app_cxt: ApplicationContext,
    ledger_descriptions: Vec<String>,
    db_tx: &mut Transaction<'_, Postgres>,
) -> Result<Block, OrchestrateError> {
    ////// 1. Get last user activity
    let last_user_activity = match find_last_user_activity(&mut **db_tx, &user_ctx.user_fp).await? {
        Some(last_user_activity) => last_user_activity,
        None => {
            // if no activity is found, then the wallet is not valid
            // at least the creation account activity should have been created.
            return Err(OrchestrateError::InvalidRecordState(
                "No user activity found".to_string(),
            ));
        }
    };

    ////// 2. Create ledgers
    let mut entry_ids = Vec::new();
    let mut ledgers: Vec<LedgerEntry> = Vec::new();

    for desc in ledger_descriptions {
        let ledger = LedgerEntry::new(acct_id.clone(), Some(desc), entry.clone());
        entry_ids.push(ledger.id.clone());
        ledgers.push(ledger);
    }

    let ledgers_saved = bulk_save_ledger(&mut **db_tx, ledgers).await? as usize;
    if ledgers_saved != entry_ids.len() {
        return Err(OrchestrateError::InvalidRecordState(
            "ledgers count is not equal".to_string(),
        ));
    }

    ////// 3. Create a chain_stamp to chain blocks together.
    ////// 3.1 Find last activity chain stamp which will be the parent.
    let parent_chain_stamp =
        match find_chain_stamp_by_id(&mut **db_tx, &last_user_activity.chain_id).await? {
            Some(chain_stamp) => chain_stamp,
            None => {
                return Err(OrchestrateError::InvalidRecordState(
                    "can not debit account with no prior activity".to_string(),
                ));
            }
        };

    ////// 3.2 Create a new chain stamp for this transaction.
    let chain_stamp = match create_chain_stamp(db_tx, Some(parent_chain_stamp)).await {
        Ok(chain_stamp) => chain_stamp,
        Err(err) => {
            return Err(err);
        }
    };

    ///// 4 create a block
    //// Create a block for ledger-entry grouping. This block will contain the root chain_stamp
    let block = Block::build(
        app_cxt.app_id.to_string(),
        app_cxt.block_region,
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

    ///// 5. Create an activity chain stamp and the block created
    match create_activity(
        &mut **db_tx,
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
            return Err(OrchestrateError::ServerError(
                "failed to create activity".to_string(),
            ));
        }
    }

    ///// 6 save block to cassandra DB
    let block_saved = save_block_chain(
        &block,
        cassandra_session,
        app_cxt.statements.insert_block_stmt,
    )
    .await
    .map_err(|err| {
        log::error!("failed to save block to cassandra DB: {}", err);
        OrchestrateError::ServerError(err.to_string())
    })?;

    if !block_saved {
        return Err(OrchestrateError::ServerError(
            "failed to save block to DB".to_string(),
        ));
    }
    Ok(block)
}
