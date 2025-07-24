use crate::core::{Currency, CurrencyRate};
use redis::aio::ConnectionManager;
use redis::AsyncTypedCommands;
use rust_decimal::Decimal;
use serde::Deserialize;
use tracing::warn;

#[derive(Clone, Deserialize)]
struct RedisCurrencyRate {
    pub rate: Decimal,
    pub app_id: String,
    pub base_currency: Currency,
    pub quote_currency: Currency,
}

pub async fn get_exchange_rate(
    currency_hash: String,
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
