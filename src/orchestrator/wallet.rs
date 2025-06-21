use crate::core::{Currency, WalletHolding};
use crate::error::OrchestrateError;
use crate::storage::{create_wallet, fetch_wallet, update_wallet_balance};
use chrono::Utc;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use sqlx::{Executor, PgConnection, Postgres, Transaction};
use std::ops::Add;

pub async fn create_wallet_holding<'a, E>(
    pool: E,
    acct_id: String,
    currency: Currency,
) -> Result<Option<WalletHolding>, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let wallet_holding = WalletHolding::new(acct_id, currency);

    let wallet_created = create_wallet(pool, &wallet_holding).await?;
    if !wallet_created {
        return Ok(None);
    }

    Ok(Some(wallet_holding))
}

pub async fn get_wallet_holding<'a, E>(
    pool: E,
    acct_id: String,
) -> Result<Option<WalletHolding>, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let wallet_holding = fetch_wallet(pool, &acct_id).await?;

    Ok(Some(wallet_holding))
}

pub async fn credit_wallet_holding(
    db_tx: &mut Transaction<'_, Postgres>,
    amount: Decimal,
    acct_id: String,
) -> Result<bool, OrchestrateError> {
    if amount == Decimal::zero() {
        return Err(OrchestrateError::InvalidArgument(
            "Amount cannot be zero".to_string(),
        ));
    }
    let mut wallet_holding = match get_wallet_holding(&mut **db_tx, acct_id).await? {
        Some(wallet_holding) => wallet_holding,
        None => {
            return Err(OrchestrateError::NotFoundError(
                "No wallet for account found".to_string(),
            ))
        }
    };

    wallet_holding.modification_time = Utc::now();
    wallet_holding.balance = wallet_holding.balance.add(amount);

    let updated_wallet = update_wallet_balance(&mut **db_tx, &wallet_holding).await?;

    if updated_wallet.balance != wallet_holding.balance {
        return Ok(false);
    }
    Ok(true)
}

pub async fn debit_wallet(
    tx: &mut PgConnection,
    amount: Decimal,
    acct_id: String,
) -> Result<bool, OrchestrateError> {
    if amount == Decimal::zero() {
        return Err(OrchestrateError::InvalidArgument(
            "Amount cannot be zero".to_string(),
        ));
    };

    let mut wallet_holding = match get_wallet_holding(&mut *tx, acct_id).await? {
        None => {
            return Err(OrchestrateError::NotFoundError(
                "No wallet for account found".to_string(),
            ))
        }
        Some(wallet_h) => wallet_h,
    };

    if wallet_holding.balance < amount {
        return Err(OrchestrateError::InvalidArgument(
            "debit amount is higher than the balance".to_string(),
        ));
    }

    wallet_holding.modification_time = Utc::now();
    wallet_holding.balance = wallet_holding.balance - amount;

    let updated_wallet = update_wallet_balance(&mut *tx, &wallet_holding).await?;
    Ok(updated_wallet.balance == wallet_holding.balance)
}
