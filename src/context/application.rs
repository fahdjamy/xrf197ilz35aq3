use crate::core::BlockRegion;
use chrono::Utc;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApplicationContext {
    pub app_id: u64,
    pub name: String,
    pub region: String,
    pub timestamp: u64,
    pub is_test_ctx: bool,
}

impl ApplicationContext {
    pub fn load(app_name: String, region: String) -> Self {
        ApplicationContext {
            region,
            app_id: 0,
            name: app_name,
            is_test_ctx: false,
            timestamp: Utc::now().timestamp() as u64,
        }
    }

    pub fn load_test_ctx(app_id: u64) -> Self {
        ApplicationContext {
            app_id,
            is_test_ctx: true,
            name: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp() as u64,
            region: BlockRegion::MexicoCentral.to_string(),
        }
    }
}

impl Display for ApplicationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "appId={} :: region={}", self.app_id, self.region)
    }
}
