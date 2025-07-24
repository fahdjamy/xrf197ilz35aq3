use crate::RedisConfig;
use redis::aio::ConnectionManager;

pub async fn get_redis_client(config: RedisConfig) -> Result<ConnectionManager, String> {
    let redis_address = format!(
        "{}://:{}@{}",
        &config.uri_scheme, &config.password, &config.host_name
    );
    let client = redis::Client::open(redis_address)
        .map_err(|err| return err.to_string())
        .map_err(|e| {
            return format!("failed to connect to redis: {}", e.to_string());
        })?;

    let result = client.get_connection_manager().await.map_err(|e| {
        return format!("failed to connect to redis: {}", e.to_string());
    })?;

    Ok(result)
}
