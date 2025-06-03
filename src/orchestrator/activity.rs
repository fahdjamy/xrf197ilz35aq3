use crate::context::UserContext;
use crate::core::ActivityTransaction;
use crate::error::OrchestrateError;
use crate::storage::{find_last_activity, save_activity};
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres};

pub async fn create_activity<'a, E>(
    pool: E,
    block_id: String,
    chain_stamp: String,
    description: String,
    user_cxt: &UserContext,
) -> Result<Option<ActivityTransaction>, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let activity_tx =
        ActivityTransaction::new(block_id, chain_stamp, description, user_cxt.user_fp.clone());
    let activity_saved = save_activity(pool, &activity_tx).await?;

    if activity_saved {
        Ok(Some(activity_tx))
    } else {
        Ok(None)
    }
}

pub async fn find_last_user_activity<'a, E>(
    pool: E,
    user_fp: &str,
) -> Result<Option<ActivityTransaction>, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    match find_last_activity(pool, user_fp).await {
        Ok(activity) => Ok(Some(activity)),
        Err(err) => match err {
            PgDatabaseError::NotFound => Ok(None),
            _ => Err(OrchestrateError::DatabaseError(err)),
        },
    }
}
