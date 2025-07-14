use crate::core::BlockRegion;
use crate::storage::PreparedAppStatements;
use chrono::Utc;
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
pub struct ApplicationContext {
    pub app_id: u64,
    pub name: String,
    pub timestamp: u64,
    pub is_test_ctx: bool,
    pub block_region: BlockRegion,
    pub beneficiary_account_id: String,
    pub statements: PreparedAppStatements,
}

impl ApplicationContext {
    pub fn load(
        app_id: u64,
        region: String,
        app_name: String,
        beneficiary_account_id: String,
        statements: PreparedAppStatements,
    ) -> Result<Self, String> {
        let block_region = match BlockRegion::from_str(&region) {
            Ok(region) => region,
            Err(e) => return Err(e.to_string()),
        };
        Ok(ApplicationContext {
            app_id,
            statements,
            block_region,
            name: app_name,
            is_test_ctx: false,
            beneficiary_account_id: beneficiary_account_id,
            timestamp: Utc::now().timestamp() as u64,
        })
    }

    pub fn load_test_ctx(
        app_id: u64,
        region: String,
        statements: PreparedAppStatements,
    ) -> Result<Self, String> {
        let block_region =
            BlockRegion::from_str(&region).unwrap_or_else(|_e| BlockRegion::MexicoCentral);
        Ok(ApplicationContext {
            app_id,
            statements,
            block_region,
            is_test_ctx: true,
            name: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp() as u64,
            beneficiary_account_id: "testAccountId".to_string(),
        })
    }
}

impl Display for ApplicationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "appId={} :: region={}", self.app_id, self.block_region)
    }
}
