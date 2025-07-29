use crate::context::ApplicationContext;
use crate::core::{get_currency_hash, Currency, CurrencyRate};
use crate::error::OrchestrateError;
use crate::storage::{
    fetch_currency_rate, get_exchange_rate, save_currency_rate_record, save_exchange_rate,
};
use redis::aio::ConnectionManager;
use rust_decimal::Decimal;
use sqlx::{Executor, Postgres};
use tracing::warn;

pub async fn save_currencies_rate<'a, E>(
    pg_pool: E,
    rate: Decimal,
    base_currency: Currency,
    quote_currency: Currency,
    app_cxt: ApplicationContext,
) -> Result<(), OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    if base_currency == quote_currency {
        return Err(OrchestrateError::InvalidArgument(
            "same currencies".to_string(),
        ));
    }
    let currencies_hash =
        get_currency_hash(&base_currency.to_string(), &quote_currency.to_string());
    ////// get saved rate in redis if there's any
    let saved_redis_currencies_rate =
        get_exchange_rate(&currencies_hash, &mut app_cxt.redis_conn.clone()).await;

    let currencies_rate = match saved_redis_currencies_rate {
        None => {
            warn!(
                "No currencies rate found for base_currency={}, quote_currency={}",
                base_currency, quote_currency
            );
            CurrencyRate {
                rate,
                base_currency,
                quote_currency,
                hash: currencies_hash,
                recorded_at: Default::default(),
                app_id: app_cxt.app_id.clone().to_string(),
            }
        }
        Some(currencies_rate) => {
            ////// Save rate to DB first
            save_currency_rate_record(pg_pool, &currencies_rate).await?;
            ///// return fetched currency
            currencies_rate
        }
    };

    save_exchange_rate(&currencies_rate, &mut app_cxt.redis_conn.clone())
        .await
        .map_err(|err| {
            OrchestrateError::ServerError(format!(
                "Failed to save exchange rate to redis, err={}",
                err
            ))
        })?;
    Ok(())
}

pub async fn convert_amount<'a, E>(
    pg_pool: E,
    amount: Decimal,
    from_curr: Currency,
    to_curr: Currency,
    conn: &mut ConnectionManager,
) -> Result<Decimal, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    if to_curr == from_curr {
        return Ok(amount);
    }
    let rate = get_rate(pg_pool, &to_curr, &from_curr, conn)
        .await
        .map_err(|err| {
            OrchestrateError::InvalidRecordState(format!("invalid/unknown currency rate: {}", err))
        })?;

    Ok(amount * rate)
}

async fn get_rate<'a, E>(
    pg_pool: E,
    to_currency: &Currency,
    from_currency: &Currency,
    conn: &mut ConnectionManager,
) -> Result<Decimal, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let hash_code = get_currency_hash(&from_currency.to_string(), &to_currency.to_string());
    let rate = match get_exchange_rate(&hash_code, conn).await {
        Some(redis_currency_rate) => redis_currency_rate.rate,
        None => match fetch_currency_rate(pg_pool, &hash_code).await? {
            None => {
                return Err(OrchestrateError::InvalidRecordState(
                    "invalid/unknown currency rate".to_string(),
                ))
            }
            Some(pg_currency_rate) => pg_currency_rate.rate,
        },
    };

    Ok(rate)
}
