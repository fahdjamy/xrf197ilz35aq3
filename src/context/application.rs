use chrono::Utc;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct ApplicationContext {
    pub app_id: u64,
    pub name: String,
    pub region: String,
    pub timestamp: u64,
}

impl ApplicationContext {
    pub fn load(app_name: String, region: String) -> Self {
        ApplicationContext {
            region,
            app_id: 0,
            name: app_name,
            timestamp: Utc::now().timestamp() as u64,
        }
    }
}

impl Display for ApplicationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "appId={} :: region={}", self.app_id, self.region)
    }
}
