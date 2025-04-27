use crate::core::{EntryType, LedgerEntry};
use crate::error::OrchestrateError;
use crate::storage::save_ledger;
use sqlx::PgPool;
use std::str::FromStr;

pub async fn create_ledger(
    pool: &PgPool,
    entry: String,
    acct_id: String,
    desc: Option<String>,
) -> Result<LedgerEntry, OrchestrateError> {
    let entry_type = EntryType::from_str(&entry)
        .map_err(|err| OrchestrateError::InvalidArgument(err.to_string()))?;

    let ledger = LedgerEntry::new(acct_id.clone(), desc, entry_type);

    // store ledger entry into the database
    let ledger_entry_created = save_ledger(pool, &ledger).await?;
    if !ledger_entry_created {
        return Err(OrchestrateError::ServerError(format!(
            "could not create a new ledger entry for acctId={}",
            acct_id
        )));
    }

    Ok(ledger)
}
