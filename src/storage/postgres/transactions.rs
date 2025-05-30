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
    info!("creating activity  :: transaction={}", activity);
    let result = sqlx::query!(
        "
INSERT INTO activity_transaction (
            transaction_id,
            timestamp,
            modification_time,
            block_id,
            chain_id
            )
            VALUES ($1, $2, $3, $4, $5)",
        activity.id,
        activity.timestamp,
        activity.modification_time,
        activity.block_id,
        activity.chain_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
