pub fn get_redis_client(address: String) -> Result<redis::Client, String> {
    let client = redis::Client::open(address).map_err(|err| return err.to_string());
    let client = client.map_err(|e| {
        return format!("failed to connect to redis: {}", e.to_string());
    })?;

    Ok(client)
}
