use chrono::{DateTime, Utc};
use serde::Serialize;

enum TransactionType {
    Payment,
}

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
    pub amount: u64,
    pub account_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    transaction_type: TransactionType,
}

impl Transaction {
    pub fn new(amount: u64, account_id: String) -> Self {
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
