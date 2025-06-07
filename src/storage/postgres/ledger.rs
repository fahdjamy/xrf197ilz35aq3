use crate::core::{EntryType, LedgerEntry};
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres, Transaction};

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

#[tracing::instrument(skip(db_tx, entries))]
pub async fn bulk_save_ledger<'a, E>(
    entries: Vec<LedgerEntry>,
    db_tx: &mut Transaction<'_, Postgres>,
) -> Result<u64, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    let mut ids = Vec::new();
    let mut entry_types = Vec::new();
    let mut seq_numbers = Vec::new();
    let mut account_ids = Vec::new();
    let mut timestamps = Vec::new();
    let mut descriptions = Vec::new();
    for entry in entries {
        ids.push(entry.id);
        account_ids.push(entry.account_id);
        timestamps.push(entry.timestamp.naive_utc());
        entry_types.push(entry.entry_type.to_string());
        seq_numbers.push(entry.sequence_number as i64);
        descriptions.push(entry.description.unwrap_or_else(|| "".to_string()));
    }
    let query = sqlx::query!(
        r#"
INSERT INTO ledger_entry (
                          id,
                          account_id,
                          description,
                          sequence_number,
                          timestamp,
                          entry_type
)
SELECT * FROM UNNEST(
                $1::VARCHAR[],
                $2::VARCHAR[],
                $3::TEXT[],
                $4::BIGINT[],
                $5::TIMESTAMP[],
                $6::text[]::entry_type[]
            )
"#,
        ids.as_slice(),
        account_ids.as_slice(),
        descriptions.as_slice(),
        seq_numbers.as_slice(),
        timestamps.as_slice(),
        entry_types.as_slice(),
    );

    let rows_affected = db_tx.execute(query).await?.rows_affected();
    Ok(rows_affected)
}
