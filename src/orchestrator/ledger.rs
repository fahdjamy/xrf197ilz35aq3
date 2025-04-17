use crate::core::{EntryType, LedgerEntry};
use crate::DomainError;
use std::str::FromStr;

pub fn create_ledger(
    entry: String,
    acct_id: String,
    desc: Option<String>,
) -> Result<LedgerEntry, DomainError> {
    let entry_type = EntryType::from_str(&entry)?;

    let ledger = LedgerEntry::new(acct_id, desc, entry_type);

    Ok(ledger)
}
