use crate::core::BlockRegion;
use crate::storage::PreparedAppStatements;
use chrono::Utc;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApplicationContext {
    pub app_id: u64,
    pub name: String,
    pub timestamp: u64,
    pub is_test_ctx: bool,
    pub app_env: Environment,
    pub block_region: BlockRegion,
    pub beneficiary_account_id: String,
    pub statements: Arc<PreparedAppStatements>,
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
        let statements = Arc::new(statements);
        Ok(ApplicationContext {
            app_id,
            statements,
            block_region,
            name: app_name,
            is_test_ctx: false,
            beneficiary_account_id,
            app_env: Environment::Dev, // TODO: Change this and load environment
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
        let statements = Arc::new(statements);
        Ok(ApplicationContext {
            app_id,
            statements,
            block_region,
            is_test_ctx: true,
            app_env: Environment::Test,
            name: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp() as u64,
            beneficiary_account_id: "testAccountId".to_string(),
        })
    }
}

impl Display for ApplicationContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "appId={} :: region={}", self.app_id, self.block_region)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Live,
    Test,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Live => "live",
            Environment::Test => "test",
            Environment::Staging => "stg",
            Environment::Production => "prod",
        }
    }

    pub fn is_local(&self) -> bool {
        *self == Environment::Dev
    }

    pub fn is_not_local(&self) -> bool {
        !self.is_local()
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
