use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

enum TransactionType {
    Payment,
    Transfer,
    Reversal,
    Commission,
    Correction,
}

/// Once a Transaction is Completed, its associated LedgerEntry records should never be changed or deleted.
/// Corrections should be made via new transactions (e.g., a Reversal or Correction transaction type)
/// that create new offsetting LedgerEntry records. Enforce this through app logic & DB permissions
#[derive(Debug, Clone, Serialize)]
enum TransactionStatus {
    Failed,
    Pending,
    Rejected,
    Reverted,
    Completed,
}

pub struct Transaction {
    pub id: String,
    pub amount: Decimal,
    pub account_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    pub transaction_type: TransactionType,
}

impl Transaction {
    pub fn new(amount: Decimal, account_id: String) -> Self {
        Transaction {
            amount,
            account_id,
            id: "0".to_string(),
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
            transaction_type: TransactionType::Payment,
        }
    }
}
