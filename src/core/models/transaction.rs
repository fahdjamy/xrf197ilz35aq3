use chrono::{DateTime, Utc};
use serde::Serialize;

enum TransactionType {}

#[derive(Debug, Clone, Serialize)]
enum TransactionStatus {
    Failed,
    Pending,
    Accepted,
    Rejected,
    Reverted,
}

pub struct Transaction {
    pub id: i64,
    pub amount: i64,
    pub account_id: i64,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
}

impl Transaction {
    pub fn new(amount: i64, account_id: i64) -> Self {
        Transaction {
            id: 0,
            amount,
            account_id,
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
        }
    }
}
