use crate::core::WalletHolding;
use crate::PgDatabaseError;
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::info;

#[tracing::instrument(skip(pg_pool, holding))]
pub async fn create_wallet(
    pg_pool: &PgPool,
    holding: &WalletHolding,
) -> Result<bool, PgDatabaseError> {
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
