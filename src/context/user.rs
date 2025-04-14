use crate::core::{generate_str_id, WalletHolding};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_fp: String,
    pub timezone: String,
    pub is_test_ctx: bool,
    pub account_id: Option<String>,
    pub wallet_holding: Option<WalletHolding>,
}

impl UserContext {
    pub fn load_test_ctx() -> Self {
        UserContext {
            account_id: None,
            is_test_ctx: true,
            wallet_holding: None,
            user_fp: generate_str_id(),
            timezone: "UTC".to_string(),
        }
    }
}

impl Display for UserContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "testCtx={}", self.is_test_ctx)
    }
}
