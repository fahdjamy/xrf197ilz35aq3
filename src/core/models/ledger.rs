use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Debit,  // Increases assets/expenses, decreases liability/equity/revenue
    Credit, // Increases liability/equity/revenue, decreases assets/expenses
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub id: String,
    pub block: String,
    pub amount: Decimal,
    pub account_id: String,
    pub entry_type: EntryType,
    pub transaction_id: String,
    pub sequence_number: String,
    pub timestamp: DateTime<Utc>,
    pub description: Option<String>,
}

impl LedgerEntry {
    pub fn new(
        block: String,
        amount: Decimal,
        account_id: String,
        desc: Option<String>,
        entry_type: EntryType,
        transaction_id: String,
    ) -> Self {
        LedgerEntry {
            block,
            amount,
            account_id,
            entry_type,
            transaction_id,
            description: desc,
            id: "".to_string(),
            timestamp: Utc::now(),
            sequence_number: "".to_string(),
        }
    }
}
