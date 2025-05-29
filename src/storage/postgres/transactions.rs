use crate::core::ActivityTransaction;
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres};
use tracing::info;

#[tracing::instrument(
    level = "debug",
    skip(pool, transaction),
    name = "Create new activity transaction"
)]
pub async fn create_activity_transaction<'a, E>(
    pool: E,
    transaction: &ActivityTransaction,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!(
        "creating activity transaction :: transaction={}",
        transaction
    );
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
        transaction.id,
        transaction.timestamp,
        transaction.modification_time,
        transaction.block_id,
        transaction.chain_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
