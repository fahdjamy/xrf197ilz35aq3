use tonic::metadata::{MetadataKey, MetadataMap};
use tracing::error;

pub fn get_header_value(metadata_map: &MetadataMap, header_name: &str) -> Option<String> {
    // For Case-Insensitivity: this creates keys that are treated case-insensitively during lookups.
    // i.e: "my-header", "My-Header", "MY-HEADER" are all considered the same
    // '::from_bytes' function (and others like from_static) will return an error if the provided
    // byte sequence doesn't represent a valid HTTP header name (i.e., it contains illegal characters)
    let header_key = match MetadataKey::from_bytes(header_name.as_bytes()) {
        Ok(key) => key,
        Err(_) => {
            error!("Invalid header name: {}", header_name);
            return None;
        }
    };

    match metadata_map.get(&header_key) {
        None => {
            error!("Header not found: {}", header_name);
            None
        }
        Some(header_value) => match header_value.to_str() {
            Ok(value) => Some(value.to_string()),
            Err(_) => {
                error!("Invalid header value");
                None
            }
        },
    }
}
