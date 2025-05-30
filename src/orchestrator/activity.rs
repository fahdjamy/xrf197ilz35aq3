use crate::core::ActivityTransaction;
use crate::error::OrchestrateError;
use crate::storage::save_activity;
use sqlx::{Executor, Postgres};

pub async fn create_activity<'a, E>(
    pool: E,
    chain_id: String,
    block_id: String,
) -> Result<Option<ActivityTransaction>, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let activity_tx = ActivityTransaction::new(block_id, chain_id);
    let activity_saved = save_activity(pool, &activity_tx).await?;

    if activity_saved {
        Ok(Some(activity_tx))
    } else {
        Ok(None)
    }
}
