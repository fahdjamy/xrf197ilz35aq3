use crate::context::{ApplicationContext, UserContext};
use crate::core::{EntryType, MonetaryTransaction, TransactionType};
use crate::error::OrchestrateError;
use crate::storage::save_monetary_tx;
use crate::{
    commit_db_transaction, create_chained_block_chain, debit_wallet, rollback_db_transaction,
    start_db_transaction,
};
use cassandra_cpp::Session;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use tracing::{error, info};

pub async fn start_debit_transaction(
    pool: &PgPool,
    amount: String,
    tx_type: String,
    account_id: String,
    user_ctx: UserContext,
    cassandra_session: Session,
    app_cxt: ApplicationContext,
) -> Result<MonetaryTransaction, OrchestrateError> {
    let event = "debitTransaction";
    let mut db_tx = start_db_transaction(pool, event).await?;

    ////// 0. Validate user request
    let decimal_amount = Decimal::from_str(&amount).map_err(|_e| {
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

    ////// 1. Debit user wallet
    let debit_tx = MonetaryTransaction::payment(decimal_amount, account_id.clone());
    debit_wallet(
        &mut db_tx,
        decimal_amount,
        account_id.clone(),
        "todo".to_string(),
    )
    .await?;

    let account_debited = save_monetary_tx(&mut *db_tx, &debit_tx).await?;
    if !account_debited {
        rollback_db_transaction(db_tx, event).await?;
        return Err(OrchestrateError::ServerError(
            "could not debit user account, try again later".to_string(),
        ));
    }

    let mut ledger_desc = Vec::new();
    ledger_desc.push("debit user account".to_string());

    ///// 2. Create blockchain
    let block = match create_chained_block_chain(
        account_id.clone(),
        EntryType::Credit,
        user_ctx,
        cassandra_session,
        app_cxt,
        ledger_desc,
        &mut db_tx,
    )
    .await
    {
        Ok(block) => {
            commit_db_transaction(db_tx, event).await?;
            block
        }
        Err(err) => {
            error!("failed to create blockchain for debit account: {}", err);
            rollback_db_transaction(db_tx, event).await?;
            return Err(err.into());
        }
    };
    info!(
        "successfully debited account {} to block {}",
        account_id, block.id
    );

    Ok(debit_tx)
}
