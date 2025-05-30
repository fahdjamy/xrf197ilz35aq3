use crate::core::{MonetaryTransaction, TransactionType};
use crate::error::OrchestrateError;
use rust_decimal::Decimal;
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
