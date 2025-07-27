use crate::RedisConfig;
use redis::aio::ConnectionManager;

pub async fn get_redis_client(config: &RedisConfig) -> Result<ConnectionManager, String> {
    // redis://[username]:[password]@[hostname]:[port]/[dbNumber]
    // redis://: The scheme indicating a standard Redis connection. If using TLS/SSL, this would be rediss://.
    // [username]: (Optional, for Redis 6 and later with ACLs) The username for authentication.
    //          If no username is provided, and a password is used, the default user "default" is assumed.
    // [password]: The password for authentication.
    // [hostname]: The IP address or hostname of the Redis server.
    // [port]: The port number on which the Redis server is listening to (default is 6379).
    // [dbNumber]: (Optional) The specific database number to connect to (e.g., /0, /1).
    let redis_address = format!(
        "redis://{}:{}@{}:{}",
        &config.username, &config.password, &config.hostname, &config.port
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
