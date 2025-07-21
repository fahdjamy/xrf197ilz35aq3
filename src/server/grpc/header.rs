use tonic::metadata::{MetadataKey, MetadataMap};
use tonic::Status;
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

pub fn get_xrf_user_auth_header(
    metadata_map: &MetadataMap,
    header_name: &str,
) -> Result<String, Status> {
    let response = get_header_value(metadata_map, header_name);
    if response.is_none() {
        Err(Status::invalid_argument("Missing xrf-user-fp".to_string()))
    } else {
        let xrf_user_auth_value = response.unwrap();
        if xrf_user_auth_value.is_empty()
            || xrf_user_auth_value.len() < 55
            || xrf_user_auth_value.len() > 125
        {
            return Err(Status::invalid_argument(
                "Invalid 'xrf-user-fp' header".to_string(),
            ));
        }
        Ok(xrf_user_auth_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::XRF_USER_FINGERPRINT;

    #[test]
    fn test_get_header_value_exists() {
        let mut metadata_map = MetadataMap::new();
        metadata_map.insert("header2", "value2".parse().unwrap());
        metadata_map.insert(XRF_USER_FINGERPRINT, "my-value".parse().unwrap());
        let result_one = get_header_value(&metadata_map, XRF_USER_FINGERPRINT);
        assert!(result_one.is_some());
        let result_two = get_header_value(&metadata_map, "header2");
        assert!(result_two.is_some());
    }

    #[test]
    fn test_get_header_value_not_exists() {
        let metadata_map = MetadataMap::new();
        let result = get_header_value(&metadata_map, XRF_USER_FINGERPRINT);
        assert!(result.is_none());
    }
}
