use crate::core::{Currency, WalletHolding};
use crate::PgDatabaseError;
use rust_decimal::Decimal;
use sqlx::{Error, Executor, Postgres};
use tracing::info;

#[tracing::instrument(skip(pg_pool, holding))]
pub async fn create_wallet<'a, E>(
    pg_pool: E,
    holding: &WalletHolding,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!("creating wallet for acctId: {}", &holding.account_id);
    let result = sqlx::query!(
        "
INSERT INTO wallet (
                    balance,
                    currency,
                    account_id,
                    modification_time
                    )
VALUES ($1, $2, $3, $4)",
        holding.balance as Decimal,
        holding.currency.clone() as Currency,
        holding.account_id,
        holding.modification_time,
    )
    .execute(pg_pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(
    level = "debug",
    skip(pg_pool, account_id),
    name = "fetch account balance"
)]
pub async fn fetch_wallets<'a, E>(
    pg_pool: E,
    account_id: &str,
) -> Result<Option<Vec<WalletHolding>>, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = sqlx::query_as!(
        WalletHolding,
        r#"
SELECT balance,
       currency as "currency: _",
       account_id,
       modification_time
FROM wallet WHERE account_id = $1
       "#,
        account_id
    )
    .fetch_all(pg_pool)
    .await;

    match result {
        Ok(wallet) => Ok(Some(wallet)),
        Err(Error::RowNotFound) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[tracing::instrument(
    level = "debug",
    skip(pg_pool, account_ids),
    name = "fetch user wallet holdings"
)]
pub async fn fetch_user_wallets<'a, E>(
    pg_pool: E,
    account_ids: &[String],
) -> Result<Vec<WalletHolding>, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    let result: Vec<WalletHolding> = sqlx::query_as!(
        WalletHolding,
        r#"
SELECT balance,
       currency as "currency: _",
       account_id,
       modification_time
FROM wallet WHERE account_id = ANY($1)
"#,
        account_ids,
    )
    .fetch_all(pg_pool)
    .await?;

    Ok(result)
}

#[tracing::instrument(
    level = "debug",
    skip(pg_pool, holding),
    name = "update wallet holding information"
)]
pub async fn update_wallet_balance<'a, E>(
    pg_pool: E,
    holding: &WalletHolding,
) -> Result<WalletHolding, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = sqlx::query_as!(
        WalletHolding,
        r#"UPDATE wallet SET balance = $1, modification_time = $2
              WHERE account_id = $3
              RETURNING balance, currency as "currency: _", modification_time, account_id"#,
        holding.balance as Decimal,
        holding.modification_time,
        holding.account_id,
    )
    .fetch_one(pg_pool)
    .await?;

    Ok(result)
}
