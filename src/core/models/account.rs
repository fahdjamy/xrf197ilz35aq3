use chrono::{DateTime, Utc};

enum AccountStatus {
    Frozen,
    Active,
    Inactive,
}

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
    pub account_id: u64,
    pub timezone: String,
    pub account_type: String,
    pub status: AccountStatus,
    pub creation_time: DateTime<Utc>,
    pub modification_time: DateTime<Utc>,
}

impl Account {
    pub fn new() -> Self {
        Account {
            account_id: 0,
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
