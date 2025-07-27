use crate::core::BlockRegion;
use crate::storage::{get_redis_client, PreparedAppStatements};
use crate::RedisConfig;
use chrono::Utc;
use redis::aio::ConnectionManager;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ApplicationContext {
    pub app_id: u64,
    pub name: String,
    pub timestamp: u64,
    pub is_test_ctx: bool,
    pub app_env: Environment,
    pub ben_account_id: String,
    pub block_region: BlockRegion,
    pub redis_conn: ConnectionManager,
    pub statements: Arc<PreparedAppStatements>,
}

impl Debug for ApplicationContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AppCxt, appId={}, name={}, is_test_ctx={}, region={}, ben_acct={}",
            self.app_id, self.name, self.is_test_ctx, self.block_region, self.ben_account_id
        )
    }
}

impl Display for ApplicationContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "appId={} :: region={}", self.app_id, self.block_region)
    }
}

impl ApplicationContext {
    pub async fn load(
        app_id: u64,
        region: String,
        app_name: String,
        ben_account_id: String,
        redis_config: &RedisConfig,
        statements: PreparedAppStatements,
    ) -> Result<Self, String> {
        let block_region = match BlockRegion::from_str(&region) {
            Ok(region) => region,
            Err(e) => return Err(e.to_string()),
        };
        let statements = Arc::new(statements);
        let redis_conn = get_redis_client(redis_config).await?;

        Ok(ApplicationContext {
            app_id,
            statements,
            redis_conn,
            block_region,
            name: app_name,
            ben_account_id,
            is_test_ctx: false,
            app_env: Environment::Dev, // TODO: Change this and load environment
            timestamp: Utc::now().timestamp() as u64,
        })
    }

    pub async fn load_test_ctx(
        app_id: u64,
        region: String,
        redis_config: &RedisConfig,
        statements: PreparedAppStatements,
    ) -> Result<Self, String> {
        let block_region =
            BlockRegion::from_str(&region).unwrap_or_else(|_e| BlockRegion::MexicoCentral);
        let statements = Arc::new(statements);
        let redis_conn = get_redis_client(redis_config).await?;
        Ok(ApplicationContext {
            app_id,
            statements,
            redis_conn,
            block_region,
            is_test_ctx: true,
            app_env: Environment::Test,
            name: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp() as u64,
            ben_account_id: "testAccountId".to_string(),
        })
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
