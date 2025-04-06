use crate::DomainError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::{write, Display};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TransactionType {
    Payment,
    Transfer,
    Reversal,
    Commission,
    Correction,
}

impl FromStr for TransactionType {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Payment" => Ok(TransactionType::Payment),
            "Transfer" => Ok(TransactionType::Transfer),
            "Reversal" => Ok(TransactionType::Reversal),
            "Commission" => Ok(TransactionType::Commission),
            "Correction" => Ok(TransactionType::Correction),
            _ => Err(DomainError::InvalidArgument(
                "unsupported transaction type".to_string(),
            )),
        }
    }
}

/// ***!IMPORTANT***: _Once a Transaction is Completed, its associated LedgerEntry records should never be changed or deleted.
/// Corrections should be made via new transactions (e.g., a Reversal or Correction transaction type)
/// that create new offsetting LedgerEntry records. Enforce this through app logic & DB permissions_
#[derive(Debug, Clone, Serialize)]
enum TransactionStatus {
    Failed,
    Pending,
    Rejected,
    Reverted,
    Completed,
}

#[derive(Debug, Clone, Serialize)]
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

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "transaction for acctId={}, txId={}",
                self.account_id, self.account_id
            ),
        )
    }
}
