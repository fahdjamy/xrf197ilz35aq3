use crate::core::CurrencyRate;
use redis::aio::ConnectionManager;
use redis::AsyncTypedCommands;
use tracing::warn;

pub async fn get_exchange_rate(
    currency_hash: &str,
    conn: &mut ConnectionManager,
) -> Option<CurrencyRate> {
    let fetched_redis_rates = match conn.get(currency_hash).await {
        Ok(fetch_str) => match fetch_str {
            None => {
                return None;
            }
            Some(result_str) => result_str,
        },
        Err(err) => {
            warn!("failed to get exchange rate: {}", err);
            return None;
        }
    };

    // convert redis JSON to RedisCurrencyRate
    let converted_curr_rate: CurrencyRate = match serde_json::from_str(&fetched_redis_rates) {
        Ok(data) => data,
        Err(err) => {
            warn!("Failed to convert redis rate to CurrencyRate: {}", err);
            return None;
        }
    };
    Some(converted_curr_rate)
}

pub async fn save_exchange_rate(
    currency_rate: &CurrencyRate,
    conn: &mut ConnectionManager,
) -> Result<(), String> {
    let currency_rate_json = serde_json::to_string(&currency_rate).map_err(|err| {
        return format!("Failed to serialize exchange rate: {}", err);
    })?;

    // save currency rate to REDIS
    conn.set(&currency_rate.hash, currency_rate_json)
        .await
        .map_err(|err| {
            warn!("Failed to save exchange rate: {}", err);
            return format!("Failed to save exchange rate: {}", err);
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::{Currency, CurrencyRate};
    use crate::storage::redis::save_exchange_rate;
    use crate::storage::{get_exchange_rate, get_redis_client};
    use crate::RedisConfig;
    use redis::aio::ConnectionManager;
    use testcontainers::core::{IntoContainerPort, WaitFor};
    use testcontainers::runners::AsyncRunner;
    use testcontainers::{ContainerAsync, GenericImage};
    use testcontainers_modules::redis::REDIS_PORT;

    async fn setup_redis_test_container() -> GenericImage {
        let image = GenericImage::new("redis", "latest")
            .with_exposed_port(6379.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"));

        image
    }

    async fn start_redis_container(
        redis_container: &ContainerAsync<GenericImage>,
    ) -> ConnectionManager {
        let host = redis_container
            .get_host()
            .await
            .expect("Failed to get host");
        let host_port = redis_container
            .get_host_port_ipv4(REDIS_PORT)
            .await
            .expect("Failed to get host port");

        let config = RedisConfig::test_config(host_port.to_string(), host.to_string());

        get_redis_client(&config)
            .await
            .expect("Failed to create redis client")
    }

    #[tokio::test]
    pub async fn test_currency_exchange_rate() {
        let image = setup_redis_test_container().await;
        let redis_container = image.start().await.expect("Failed to start container");

        let mut conn_manager = start_redis_container(&redis_container).await;

        let response = get_exchange_rate("key", &mut conn_manager).await;
        assert!(response.is_none());
    }

    #[tokio::test]
    pub async fn test_set_current_exchange_rate() {
        let image = setup_redis_test_container().await;
        let redis_container = image.start().await.expect("Failed to start container");

        let host = redis_container
            .get_host()
            .await
            .expect("Failed to get host");
        let host_port = redis_container
            .get_host_port_ipv4(REDIS_PORT)
            .await
            .expect("Failed to get host port");

        let config = RedisConfig::test_config(host_port.to_string(), host.to_string());
        let mut conn_manager = get_redis_client(&config)
            .await
            .expect("Failed to create redis client");

        let currency_rate = CurrencyRate {
            rate: Default::default(),
            app_id: "app".to_string(),
            base_currency: Currency::USD,
            quote_currency: Currency::USD,
            hash: "test_hash".to_string(),
            recorded_at: Default::default(),
        };

        save_exchange_rate(&currency_rate, &mut conn_manager)
            .await
            .expect("Failed to save exchange rate in DB");

        let response = get_exchange_rate(&currency_rate.hash, &mut conn_manager).await;
        dbg!(&response);

        assert!(response.is_some());
    }
}
