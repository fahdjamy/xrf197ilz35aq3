use crate::core::{get_currency_hash, Currency};
use crate::error::OrchestrateError;
use crate::storage::{fetch_currency_rate, get_exchange_rate};
use rust_decimal::Decimal;
use sqlx::{Executor, Postgres};

pub async fn convert_amount<'a, E>(
    pg_pool: E,
    amount: Decimal,
    from_curr: Currency,
    to_curr: Currency,
) -> Result<Decimal, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    if to_curr == from_curr {
        return Ok(amount);
    }
    let rate = get_rate(pg_pool, &to_curr, &from_curr)
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
) -> Result<Decimal, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let hash_code = get_currency_hash(&from_currency.to_string(), &to_currency.to_string());
    let rate = match get_exchange_rate(hash_code.clone(), from_currency, to_currency) {
        Some(redis_rate) => redis_rate,
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
