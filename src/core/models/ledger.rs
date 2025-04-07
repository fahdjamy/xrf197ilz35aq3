use crate::core::generate_timebase_str_id;
use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Debit,  // Increases assets/expenses, decreases liability/equity/revenue
    Credit, // Increases liability/equity/revenue, decreases assets/expenses
}

impl Display for EntryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::Debit => {
                write!(f, "Debit")
            }
            EntryType::Credit => {
                write!(f, "Credit")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub id: String,
    pub block: String,
    pub account_id: String,
    pub sequence_number: u64,
    pub entry_type: EntryType,
    pub transaction_id: String,
    pub timestamp: DateTime<Utc>,
    pub description: Option<String>,
}

impl LedgerEntry {
    pub fn new(
        block: String,
        account_id: String,
        desc: Option<String>,
        entry_type: EntryType,
        transaction_id: String,
    ) -> Self {
        LedgerEntry {
            block,
            account_id,
            entry_type,
            transaction_id,
            description: desc,
            sequence_number: 0,
            timestamp: Utc::now(),
            id: generate_timebase_str_id(),
        }
    }
}

impl Display for LedgerEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "blockId={}, acctId={}, ledgerEntry={}",
            self.block, self.account_id, self.id
        )
    }
}
