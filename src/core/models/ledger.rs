use crate::core::generate_timebase_str_id;
use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "entry_type")]
pub enum EntryType {
    Debit,  // Increases assets/expenses, decreases liability/equity/revenue
    Credit, // Increases liability/equity/revenue, decreases assets/expenses
    Initialization,
}

impl FromStr for EntryType {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debit" | "Debit" => Ok(EntryType::Debit),
            "credit" | "Credit" => Ok(EntryType::Credit),
            "init" | "Initialization" => Ok(EntryType::Initialization),
            _ => Err(DomainError::ParseError("Unknown entry type".to_string())),
        }
    }
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
            EntryType::Initialization => {
                write!(f, "Initialization")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub id: String,
    pub account_id: String,
    pub sequence_number: u64,
    pub entry_type: EntryType,
    pub timestamp: DateTime<Utc>,
    pub description: Option<String>,
}

impl LedgerEntry {
    pub fn new(account_id: String, desc: Option<String>, entry_type: EntryType) -> Self {
        LedgerEntry {
            account_id,
            entry_type,
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
            "entry={}, for acctId={}, at={}",
            self.id,
            self.account_id,
            self.timestamp.format("%Y-%m-%d %H:%M:%S")
        )
    }
}
