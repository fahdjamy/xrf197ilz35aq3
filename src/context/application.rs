use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ApplicationContext {
    pub app_id: u64,
    pub name: String,
    pub region: String,
    pub timestamp: DateTime<Utc>,
}
