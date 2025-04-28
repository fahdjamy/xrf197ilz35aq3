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
