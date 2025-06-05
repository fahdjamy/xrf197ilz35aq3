use crate::core::{EntryType, LedgerEntry};
use crate::error::OrchestrateError;
use crate::storage::save_ledger;
use sqlx::{Executor, Postgres};

pub async fn create_ledger<'a, E>(
    pool: E,
    entry: EntryType,
    account_id: String,
    desc: Option<String>,
) -> Result<LedgerEntry, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let ledger = LedgerEntry::new(account_id.clone(), desc, entry);

    // store ledger entry into the database
    let ledger_entry_created = save_ledger(pool, &ledger).await?;
    if !ledger_entry_created {
        return Err(OrchestrateError::ServerError(format!(
            "could not create a new ledger entry for acctId={}",
            account_id
        )));
    }

    Ok(ledger)
}
