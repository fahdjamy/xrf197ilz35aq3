use crate::core::{generate_str_id, Currency};
use crate::DomainError;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use serde::Serialize;
use std::fmt::{write, Display, Formatter};
use std::str::FromStr;

#[derive(Serialize, Debug, Clone)]
pub enum AccountStatus {
    Frozen,
    Active,
    Inactive,
}

impl Display for AccountStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::Frozen => {
                write!(f, "Frozen")
            }
            AccountStatus::Active => {
                write!(f, "Active")
            }
            AccountStatus::Inactive => {
                write!(f, "Inactive")
            }
        }
    }
}

/// AccountType:
///
/// ***SystemFee*** Represents an account owned by the platform/system itself, used to collect fees,
/// commissions, or other revenue generated from transactions.
///
/// ***Escrow*** Represents an account that holds funds temporarily during a transaction, pending certain conditions being met
///
/// ***Wallet*** Represents the primary account holding funds directly attributable to the user
///
/// ***Payable*** Funds owed by the system to external entities (e.g., payouts pending to banks)
///
/// ***Normal*** These are account types used for users who are not saving w/ the system
///
/// Examples transaction of an accountTypes in action
///
///     1. Buyer A pays $50 and Buyer A's UserWallet is debited $50.
///     3. An Escrow account is created for the Seller, and it is credited $50.
///     4. The seller transfers the item, Buyer-A confirms that they have received the asset.
///     5. The Escrow account for the Seller is debited $50.
///     6. The seller's Wallet is credited $50. (If the transaction fails, step 5/6 is debiting Escrow and crediting the Buyer's Wallet).
///
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub enum AccountType {
    Normal,
    Wallet,
    Escrow,
    SystemFee,
}

impl FromStr for AccountType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Normal" | "normal" => Ok(AccountType::Normal),
            "Wallet" | "wallet" => Ok(AccountType::Wallet),
            "Escrow" | "escrow" => Ok(AccountType::Escrow),
            "SystemFee" | "system_fee" => Ok(AccountType::SystemFee),
            _ => Err(DomainError::ParseError(
                "unrecognized account type".to_string(),
            )),
        }
    }
}

impl Display for AccountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Normal => {
                write!(f, "Normal Acct")
            }
            AccountType::Wallet => {
                write!(f, "Wallet Acct")
            }
            AccountType::Escrow => {
                write!(f, "Escrow Acct")
            }
            AccountType::SystemFee => {
                write!(f, "SystemFee Acct")
            }
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Account {
    pub id: String,
    pub locked: bool,
    pub user_fp: String,
    pub timezone: String,
    pub currency: Currency,
    pub status: AccountStatus,
    pub account_type: AccountType,
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
}

impl Account {
    pub fn new(
        user_fp: String,
        timezone: String,
        currency: Currency,
        account_type: AccountType,
    ) -> Self {
        let now = Utc::now();
        Account {
            user_fp,
            timezone,
            currency,
            account_type,
            locked: false,
            creation_time: now,
            id: generate_str_id(),
            modification_time: now,
            status: AccountStatus::Active,
        }
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "Acct id={}, timezone={}, acctType={}",
                self.id, self.timezone, self.account_type
            ),
        )
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct WalletHolding {
    pub balance: Decimal,
    pub account_id: String, // there should be a 1:1 (account_type x account_id) entry for this
    pub last_entry_id: Option<String>,
    pub modification_time: DateTime<Utc>,
}

impl WalletHolding {
    pub fn new(account_id: String) -> Self {
        WalletHolding {
            account_id,
            last_entry_id: None,
            balance: Decimal::zero(),
            modification_time: Utc::now(),
        }
    }
}

impl Display for WalletHolding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Acct Bal for = {} | [REDACTED]", self.account_id)
    }
}
