use crate::core::ActivityTransaction;
use crate::PgDatabaseError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Executor, Postgres};
use tracing::info;

#[derive(Debug, Clone, Serialize)]
pub struct ActivityDO {
    pub user_fp: String,
    pub block_id: String,
    pub chain_id: String,
    pub description: String,
    pub transaction_id: String,
    pub timestamp: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
}

impl From<ActivityDO> for ActivityTransaction {
    fn from(activity: ActivityDO) -> Self {
        Self {
            user_fp: activity.user_fp,
            id: activity.transaction_id,
            block_id: activity.block_id,
            chain_id: activity.chain_id,
            timestamp: activity.timestamp,
            description: activity.description,
            modification_time: activity.modification_time,
        }
    }
}

#[tracing::instrument(level = "debug", skip(pool, activity), name = "Create new activity")]
pub async fn save_activity<'a, E>(
    pool: E,
    activity: &ActivityTransaction,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("creating activity  :: activity={}", activity);
    let result = sqlx::query!(
        "
INSERT INTO activity (
            user_fp,
            block_id,
            chain_id,
            timestamp,
            description,
            transaction_id,
            modification_time
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
        activity.user_fp,
        activity.block_id,
        activity.chain_id,
        activity.timestamp,
        activity.description,
        activity.id,
        activity.modification_time,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(level = "debug", skip(pool, user_fp), name = "Find last user activity")]
pub async fn find_last_activity<'a, E>(
    pool: E,
    user_fp: &str,
) -> Result<ActivityTransaction, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = sqlx::query_as!(
        ActivityDO,
        "
SELECT * FROM activity
WHERE user_fp = $1
ORDER BY timestamp DESC
LIMIT 1",
        user_fp,
    )
    .fetch_one(pool)
    .await?;

    Ok(result.into())
}
