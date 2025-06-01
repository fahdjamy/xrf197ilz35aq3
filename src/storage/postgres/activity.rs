use crate::core::ActivityTransaction;
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres};
use tracing::info;

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
