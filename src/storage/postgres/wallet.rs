use crate::core::{Account, WalletHolding};
use crate::PgDatabaseError;
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::info;

pub async fn create_wallet(
    pg_pool: &PgPool,
    account: &Account,
    wallet_holding: &WalletHolding,
) -> Result<bool, PgDatabaseError> {
    info!("creating wallet for acct: {}", account);
    let result = sqlx::query!(
        "
INSERT INTO wallet (
                    balance,
                    account_id,
                    last_entry_id,
                    modification_time
                    )
VALUES ($1, $2, $3, $4)",
        wallet_holding.balance as Decimal,
        wallet_holding.account_id,
        wallet_holding.last_entry_id,
        wallet_holding.modification_time,
    )
    .execute(pg_pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
