use crate::core::{get_currency_hash, Currency};
use crate::error::OrchestrateError;
use crate::storage::{fetch_currency_rate, get_exchange_rate};
use redis::aio::ConnectionManager;
use rust_decimal::Decimal;
use sqlx::{Executor, Postgres};

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
