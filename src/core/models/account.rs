use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

enum AccountStatus {
    Frozen,
    Active,
    Inactive,
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
///     1. Buyer pays $50.
///     2. Buyer's UserWallet is debited $50.
///     3. An Escrow account (perhaps specific to this transaction or seller) is credited $50.
///     4. Seller ships the item, buyer confirms receipt.
///     5. The Escrow account is debited $50.
///     6. Seller's UserWallet is credited $50. (If the transaction fails, step 5/6 is debiting Escrow and crediting the Buyer's Wallet).
enum AccountType {
    Normal,
    Wallet,
    Escrow,
    SystemFee,
}

pub struct Account {
    pub id: String,
    pub freeze: bool,
    pub user_fp: String,
    pub timezone: String,
    pub account_type: String,
    pub status: AccountStatus,
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
}

impl Account {
    pub fn new() -> Self {
        Account {
            freeze: false,
            id: "".to_string(),
            user_fp: "".to_string(),
            timezone: "".to_string(),
            account_type: "".to_string(),
            status: AccountStatus::Active,
            creation_time: Default::default(),
            modification_time: Default::default(),
        }
    }
}

pub struct AccountBalance {
    pub version: u64,
    pub balance: Decimal,
    pub account_id: String, // there should be a 1:1 (account_type x account_id) entry for this
    pub account_type: AccountType,
    pub last_entry_id: Option<String>,
    pub modification_time: DateTime<Utc>,
}
