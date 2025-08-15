use crate::core::BlockRegion;
use crate::storage::{get_redis_client, PreparedAppStatements};
use crate::{Environment, RedisConfig};
use redis::aio::ConnectionManager;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApplicationContext {
    pub app_id: String,
    pub is_test_ctx: bool,
    pub app_env: Environment,
    pub block_region: BlockRegion,
    pub redis_conn: ConnectionManager,
    pub statements: Arc<PreparedAppStatements>,
}

impl Debug for ApplicationContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AppCxt, appId={}, appName=xrfQ3, is_test_ctx={}, region={}",
            self.app_id, self.is_test_ctx, self.block_region
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
        app_id: String,
        region: String,
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
            is_test_ctx: false,
            app_env: Environment::Dev, // TODO: Change this and load environment
        })
    }

    pub async fn load_test_ctx(
        app_id: String,
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
        })
    }
}
