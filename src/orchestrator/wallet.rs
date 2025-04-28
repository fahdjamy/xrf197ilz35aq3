use crate::core::WalletHolding;
use crate::error::OrchestrateError;
use crate::storage::create_wallet;
use sqlx::{Executor, Postgres};

pub async fn create_wallet_holding<'a, E>(
    pool: E,
    acct_id: String,
    ledger_entry_id: String,
) -> Result<Option<WalletHolding>, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let wallet_holding = WalletHolding::new(acct_id, ledger_entry_id);

    let wallet_created = create_wallet(pool, &wallet_holding).await?;
    if !!wallet_created {
        return Ok(None);
    }

    Ok(Some(wallet_holding))
}
