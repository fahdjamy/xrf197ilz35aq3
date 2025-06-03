use crate::core::{MonetaryTransaction, TransactionType};
use crate::error::OrchestrateError;
use crate::{debit_wallet, find_last_user_activity, start_db_transaction};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use tracing::error;

pub fn create_payment_transaction(
    amount: String,
    tx_type: String,
    account_id: String,
) -> Result<MonetaryTransaction, OrchestrateError> {
    let decimal_amount = Decimal::from_str(&amount).map_err(|e| {
        error!("invalid decimal_amount: {}", e);
        return OrchestrateError::InvalidArgument("cannot parse amount".to_string());
    })?;

    let transaction_type = TransactionType::from_str(&tx_type).map_err(|err| {
        return OrchestrateError::InvalidArgument(err.to_string());
    })?;

    if transaction_type.must_be_positive()
        && (decimal_amount.is_sign_negative() || decimal_amount.is_zero())
    {
        return Err(OrchestrateError::InvalidArgument(
            "amount cannot be less than or equal to 0 for tx".to_string(),
        ));
    }

    let tx = MonetaryTransaction::payment(decimal_amount, account_id, transaction_type);
    Ok(tx)
}

pub async fn start_debit_transaction(
    pool: &PgPool,
    user_fp: &str,
    amount: String,
    tx_type: String,
    account_id: String,
) -> Result<MonetaryTransaction, OrchestrateError> {
    let event = "debitTransaction";
    let mut db_tx = start_db_transaction(pool, event).await?;

    let last_user_activity = match find_last_user_activity(&mut *db_tx, user_fp).await? {
        Some(last_user_activity) => last_user_activity,
        None => {
            // if no activity is found, then the wallet is not valid
            // at least the creation account activity should have been created.
            return Err(OrchestrateError::InvalidRecordState(
                "No user activity found".to_string(),
            ));
        }
    };

    let decimal_amount = Decimal::from_str(&amount).map_err(|e| {
        return OrchestrateError::InvalidArgument("cannot parse amount".to_string());
    })?;

    let transaction_type = TransactionType::from_str(&tx_type).map_err(|err| {
        return OrchestrateError::InvalidArgument(err.to_string());
    })?;

    if transaction_type.must_be_positive()
        && (decimal_amount.is_sign_negative() || decimal_amount.is_zero())
    {
        return Err(OrchestrateError::InvalidArgument(
            "amount cannot be less than or equal to 0 for tx".to_string(),
        ));
    }

    let tx =
        MonetaryTransaction::payment(decimal_amount, account_id.clone(), TransactionType::Payment);

    debit_wallet(&mut db_tx, decimal_amount, account_id, "todo".to_string()).await?;
    unimplemented!()
}
