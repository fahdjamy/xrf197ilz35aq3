use uuid::Uuid;

pub fn generate_timebase_str_id() -> String {
    Uuid::now_v7().to_string()
}

pub fn generate_str_id() -> String {
    Uuid::new_v4().to_string()
}
