use crate::RedisConfig;

pub fn get_redis_client(config: RedisConfig) -> Result<redis::Client, String> {
    let redis_address = format!(
        "{}://:{}@{}",
        &config.uri_scheme, &config.password, &config.host_name
    );
    let client = redis::Client::open(redis_address).map_err(|err| return err.to_string());
    let client = client.map_err(|e| {
        return format!("failed to connect to redis: {}", e.to_string());
    })?;

    Ok(client)
}
