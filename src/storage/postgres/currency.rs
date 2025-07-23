use crate::core::{Currency, CurrencyRate};
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres};
use tracing::info;

#[tracing::instrument(
    level = "debug",
    skip(pool, currencies_hash),
    name = "Find currencies rate"
)]
pub async fn fetch_currency_rate<'a, E>(
    pool: E,
    currencies_hash: &str,
) -> Result<Option<CurrencyRate>, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("fetching currency rate");
    let result = sqlx::query_as!(
        CurrencyRate,
        r#"
    SELECT
        app_id,
        currencies_hash as hash,
        base_currency as "base_currency: Currency",
        quote_currency as "quote_currency: Currency",
        recorded_at

    FROM currency_rates
    WHERE currencies_hash = $1"#,
        currencies_hash
    )
    .fetch_optional(pool)
    .await?;
    Ok(result)
}
