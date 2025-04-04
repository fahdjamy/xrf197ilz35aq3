use uuid::Uuid;
use chrono;

pub struct RequestId(String);

pub fn generate_request_id() -> String {
    // Replace with a more robust generator like UUID if preferred
    let time_ms = chrono::Utc::now().timestamp_millis();
    format!("xrf_ilz_q3_{}*{}", Uuid::new_v4().to_string(), time_ms)
}
