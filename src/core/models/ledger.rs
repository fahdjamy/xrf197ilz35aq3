use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Debit,  // Increases assets/expenses, decreases liability/equity/revenue
    Credit, // Increases liability/equity/revenue, decreases assets/expenses
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub id: String,
    pub block: String,
    pub amount: String,
    pub account_id: String,
    pub entry_type: EntryType,
    pub transaction_id: String,
    pub sequence_number: String,
    pub timestamp: DateTime<Utc>,
    pub description: Option<String>,
}
