use crate::core::{EntryType, LedgerEntry};
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres};

#[tracing::instrument(skip(pg_pool, ledger_entry))]
pub async fn save_ledger<'a, E>(
    pg_pool: E,
    ledger_entry: &LedgerEntry,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    tracing::info!("Creating new ledger :: entry={}", ledger_entry);
    let result = sqlx::query!(
        "
INSERT INTO ledger_entry (
            id,
            account_id,
            description,
            sequence_number,
            timestamp,
            entry_type
        )
VALUES ($1, $2, $3, $4, $5, $6)",
        ledger_entry.id.clone(),
        ledger_entry.account_id.clone(),
        ledger_entry.description,
        ledger_entry.sequence_number.clone() as i64,
        ledger_entry.timestamp,
        ledger_entry.entry_type.clone() as EntryType,
    )
    .execute(pg_pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
