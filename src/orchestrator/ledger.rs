use crate::core::{EntryType, LedgerEntry};
use std::str::FromStr;

pub fn create_ledger(
    entry: String,
    acct_id: String,
    desc: Option<String>,
) -> Result<LedgerEntry, String> {
    let entry_type = EntryType::from_str(&entry).map_err(|e| e.to_string())?;

    let ledger = LedgerEntry::new(acct_id, desc, entry_type);

    Ok(ledger)
}
