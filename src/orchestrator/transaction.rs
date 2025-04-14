use crate::core::{Transaction, TransactionType};
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::error;

pub fn create_payment_transaction(
    amount: String,
    tx_type: String,
    account_id: String,
) -> Result<Transaction, String> {
    let decimal_amount = Decimal::from_str(&amount).map_err(|e| {
        error!("invalid decimal_amount: {}", e);
        return format!("invalid decimal_amount: {}", amount);
    })?;

    let transaction_type = TransactionType::from_str(&tx_type).map_err(|err| {
        return format!("invalid transaction type: {}", err);
    })?;

    if transaction_type.must_be_positive()
        && (decimal_amount.is_sign_negative() || decimal_amount.is_zero())
    {
        return Err("amount cannot be negative".to_string());
    }

    let tx = Transaction::payment(decimal_amount, account_id, transaction_type);
    Ok(tx)
}
