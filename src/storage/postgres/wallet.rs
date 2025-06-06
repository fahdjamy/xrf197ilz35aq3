use crate::core::WalletHolding;
use crate::PgDatabaseError;
use rust_decimal::Decimal;
use sqlx::{Executor, Postgres};
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
                    account_id,
                    last_entry_id,
                    modification_time
                    )
VALUES ($1, $2, $3, $4)",
        holding.balance as Decimal,
        holding.account_id,
        holding.last_entry_id,
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
pub async fn fetch_wallet<'a, E>(
    pg_pool: E,
    account_id: &str,
) -> Result<WalletHolding, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = sqlx::query_as!(
        WalletHolding,
        "SELECT balance, account_id, last_entry_id, modification_time FROM wallet WHERE account_id = $1",
        account_id
    )
    .fetch_one(pg_pool)
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
        r#"UPDATE wallet SET balance = $1, last_entry_id = $2, modification_time = $3
              WHERE account_id = $4
              RETURNING balance, last_entry_id, modification_time, account_id"#,
        holding.balance as Decimal,
        holding.last_entry_id,
        holding.modification_time,
        holding.account_id,
    )
    .fetch_one(pg_pool)
    .await?;

    Ok(result)
}
