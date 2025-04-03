use uuid::Uuid;

pub struct RequestId(String);

pub fn generate_request_id() -> String {
    // Replace with a more robust generator like UUID if preferred
    format!("xrf_ilz_q3_{}", Uuid::new_v4().to_string())
}
