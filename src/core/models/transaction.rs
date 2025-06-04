use crate::core::generate_timebase_str_id;
use crate::DomainError;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::{write, Display, Formatter};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, sqlx::Type)]
pub enum TransactionType {
    Payment,
    Transfer,
    Reversal,
    Commission,
    Correction,
    Initialization,
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Payment => {
                write!(f, "Payment")
            }
            TransactionType::Transfer => {
                write!(f, "Transfer")
            }
            TransactionType::Reversal => {
                write!(f, "Reversal")
            }
            TransactionType::Commission => {
                write!(f, "Commission")
            }
            TransactionType::Correction => {
                write!(f, "Correction")
            }
            TransactionType::Initialization => {
                write!(f, "Initiation")
            }
        }
    }
}

impl TransactionType {
    pub fn must_be_positive(&self) -> bool {
        *self == TransactionType::Payment
            || *self == TransactionType::Transfer
            || *self == TransactionType::Commission
    }
}

impl FromStr for TransactionType {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Payment" | "payment" => Ok(TransactionType::Payment),
            "Transfer" | "transfer" => Ok(TransactionType::Transfer),
            "Reversal" | "reversal" => Ok(TransactionType::Reversal),
            "Commission" | "commission" => Ok(TransactionType::Commission),
            "Correction" | "correction" => Ok(TransactionType::Correction),
            _ => Err(DomainError::InvalidArgument(
                "unsupported transaction type".to_string(),
            )),
        }
    }
}

/// ***!IMPORTANT***: _Once a Transaction is Completed, its associated LedgerEntry records should never be changed or deleted.
/// Corrections should be made via new transactions (e.g., a Reversal or Correction transaction type)
/// that create new offsetting LedgerEntry records. Enforce this through app logic & DB permissions_
#[derive(Debug, Clone, Serialize, Eq, PartialEq, sqlx::Type, Deserialize)]
pub enum TransactionStatus {
    Failed,
    Pending,
    Rejected,
    Reverted,
    Completed,
}

impl TransactionStatus {
    fn is_final(&self) -> bool {
        *self == TransactionStatus::Completed
            || *self == TransactionStatus::Rejected
            || *self == TransactionStatus::Failed
            || *self == TransactionStatus::Reverted
    }
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Failed => {
                write!(f, "Failed")
            }
            TransactionStatus::Pending => {
                write!(f, "Pending")
            }
            TransactionStatus::Rejected => {
                write!(f, "Rejected")
            }
            TransactionStatus::Reverted => {
                write!(f, "Reverted")
            }
            TransactionStatus::Completed => {
                write!(f, "Completed")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MonetaryTransaction {
    pub id: String,
    pub amount: Decimal,
    pub account_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    pub modification_date: DateTime<Utc>,
    pub transaction_type: TransactionType,
}

impl MonetaryTransaction {
    pub fn payment(amount: Decimal, account_id: String, tx_type: TransactionType) -> Self {
        MonetaryTransaction {
            amount,
            account_id,
            timestamp: Utc::now(),
            transaction_type: tx_type,
            modification_date: Utc::now(),
            id: generate_timebase_str_id(),
            status: TransactionStatus::Pending,
        }
    }

    pub fn build(
        amount: Decimal,
        account_id: String,
        tx_type: TransactionType,
        status: TransactionStatus,
    ) -> Result<Self, DomainError> {
        let now = Utc::now();
        // validate transactions if tx_type is Initialization
        if tx_type == TransactionType::Initialization {
            if !amount.is_zero() {
                return Err(DomainError::InvalidArgument(
                    "amount must be zero for initialization transactions".to_string(),
                ));
            } else if status != TransactionStatus::Completed {
                return Err(DomainError::InvalidArgument(
                    "status must be completed for initialization transactions".to_string(),
                ));
            }
        } else if amount <= Decimal::zero() {
            return Err(DomainError::InvalidArgument(
                "amount must be greater than zero".to_string(),
            ));
        } else if tx_type != TransactionType::Initialization && status != TransactionStatus::Pending
        {
            return Err(DomainError::InvalidArgument(
                "Status must be pending".to_string(),
            ));
        }

        Ok(MonetaryTransaction {
            amount,
            status,
            account_id,
            timestamp: now,
            modification_date: now,
            transaction_type: tx_type,
            id: generate_timebase_str_id(),
        })
    }

    pub fn change_status(&mut self, status: TransactionStatus) -> Result<(), DomainError> {
        if status == TransactionStatus::Pending {
            return Err(DomainError::InvalidArgument(
                "invalid transaction status".to_string(),
            ));
        }
        if self.status.is_final() {
            return Err(DomainError::InvalidState(
                "cannot change transaction status".to_string(),
            ));
        }
        if self.transaction_type == TransactionType::Initialization {
            return Err(DomainError::InvalidState(
                "Initialization transaction are final. Status change is invalid".to_string(),
            ));
        }

        self.status = status;
        self.modification_date = Utc::now();

        Ok(())
    }
}

impl Display for MonetaryTransaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "transaction for acctId={}, txId={}",
                self.account_id, self.account_id
            ),
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityTransaction {
    pub id: String,
    pub user_fp: String,
    pub block_id: String,
    pub chain_id: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
}

impl ActivityTransaction {
    pub fn new(block_id: String, chain_id: String, desc: String, user_fp: String) -> Self {
        let now = Utc::now();
        ActivityTransaction {
            user_fp,
            block_id,
            chain_id,
            timestamp: now,
            description: desc,
            modification_time: now,
            id: generate_timebase_str_id(),
        }
    }
}

impl Display for ActivityTransaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "activity transaction id={} :: for blockId={}, chainId={}",
                self.id, self.block_id, self.chain_id
            ),
        )
    }
}
