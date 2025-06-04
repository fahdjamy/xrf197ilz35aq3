use crate::core::{MonetaryTransaction, TransactionStatus, TransactionType};
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres};

#[tracing::instrument(skip(pool, transaction))]
pub async fn save_monetary_tx<'a, E>(
    pool: E,
    transaction: MonetaryTransaction,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = sqlx::query!(
        "
INSERT INTO monetary_transaction
(
 status,
 amount,
 timestamp,
 account_id,
 transaction_id,
 transaction_type,
 modification_date
 )
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT(transaction_type) DO NOTHING
",
        transaction.status as TransactionStatus,
        transaction.amount,
        transaction.timestamp,
        transaction.account_id,
        transaction.id,
        transaction.transaction_type as TransactionType,
        transaction.modification_date,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
