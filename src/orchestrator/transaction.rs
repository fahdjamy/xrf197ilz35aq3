use crate::context::{ApplicationContext, UserContext};
use crate::core::{AccountStatus, Currency, EntryType, MonetaryTransaction, TransactionType};
use crate::error::OrchestrateError;
use crate::storage::{find_account_by_id, save_monetary_tx};
use crate::{
    commit_db_transaction, convert_amount, create_chained_block_chain, credit_wallet_holding,
    debit_wallet, rollback_db_transaction, start_db_transaction,
};
use cassandra_cpp::Session;
use redis::aio::ConnectionManager;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use std::str::FromStr;
use tracing::{error, info};

pub async fn debit_wallet_transaction(
    pool: &PgPool,
    amount: String,
    tx_type: String,
    account_id: String,
    user_ctx: &UserContext,
    cassandra_session: &Session,
    app_cxt: &ApplicationContext,
) -> Result<MonetaryTransaction, OrchestrateError> {
    let event = "debitTransaction";
    let decimal_amount = Decimal::from_str(&amount).map_err(|_e| {
        return OrchestrateError::InvalidArgument("cannot parse amount".to_string());
    })?;
    let transaction_type = TransactionType::from_str(&tx_type).map_err(|err| {
        return OrchestrateError::InvalidArgument(err.to_string());
    })?;

    ////// 0. Validate user request
    if transaction_type.must_be_positive()
        && (decimal_amount.is_sign_negative() || decimal_amount.is_zero())
    {
        return Err(OrchestrateError::InvalidArgument(
            "amount must be greater than zero".to_string(),
        ));
    }

    let mut ledger_desc = Vec::new();
    ledger_desc.push("debit user account".to_string());

    perform_wallet_transaction(
        event,
        pool,
        decimal_amount,
        account_id,
        user_ctx,
        EntryType::Debit,
        cassandra_session,
        app_cxt,
        ledger_desc,
    )
    .await
}

pub async fn perform_wallet_transaction(
    event: &str,
    pool: &PgPool,
    amount: Decimal,
    account_id: String,
    user_ctx: &UserContext,
    tx_entry_type: EntryType,
    cassandra_session: &Session,
    app_cxt: &ApplicationContext,
    mut ledger_desc: Vec<String>,
) -> Result<MonetaryTransaction, OrchestrateError> {
    let mut db_tx = start_db_transaction(pool, event).await?;
    let user_acct = find_account_by_id(&mut *db_tx, &account_id).await?;
    if user_acct.locked
        || user_acct.status == AccountStatus::Frozen
        || user_acct.status == AccountStatus::Inactive
    {
        return Err(OrchestrateError::InvalidRecordState(
            "the user's account is locked/frozen/inactive".to_string(),
        ));
    }

    // 1. Charge the user account with commission
    let commission = amount * Decimal::from_str("0.001").unwrap();
    charge_commission(
        &account_id,
        commission,
        "TODO",
        &mut app_cxt.redis_conn.clone(),
        &mut db_tx,
    )
    .await?;

    ////// 2. Debit user wallet
    let amount = amount - commission; // subtract commission from final amount
    let debit_tx = MonetaryTransaction::payment(amount, account_id.clone());
    debit_wallet(&mut db_tx, amount, account_id.clone()).await?;

    let account_debited = save_monetary_tx(&mut *db_tx, &debit_tx).await?;
    if !account_debited {
        rollback_db_transaction(db_tx, event).await?;
        return Err(OrchestrateError::ServerError(
            "could not perform wallet transaction".to_string(),
        ));
    }

    ledger_desc.push("charge user wallet with commission".to_string());

    ///// 3. Create blockchain
    let block = match create_chained_block_chain(
        account_id.clone(),
        tx_entry_type,
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

pub async fn credit_wallet(
    pool: &PgPool,
    amount: String,
    tx_type: String,
    account_id: String,
    user_ctx: &UserContext,
    cassandra_session: &Session,
    app_cxt: &ApplicationContext,
) -> Result<MonetaryTransaction, OrchestrateError> {
    let event = "debitTransaction";
    let decimal_amount = Decimal::from_str(&amount).map_err(|_e| {
        return OrchestrateError::InvalidArgument("cannot parse amount".to_string());
    })?;
    let transaction_type = TransactionType::from_str(&tx_type).map_err(|err| {
        return OrchestrateError::InvalidArgument(err.to_string());
    })?;

    ////// 0. Validate user request
    if transaction_type.must_be_positive()
        && (decimal_amount.is_sign_negative() || decimal_amount.is_zero())
    {
        return Err(OrchestrateError::InvalidArgument(
            "amount must be greater than zero".to_string(),
        ));
    }

    if !transaction_type.is_credit_transaction() {
        return Err(OrchestrateError::InvalidArgument(
            "invalid transaction type for crediting user wallets".to_string(),
        ));
    }
    let mut ledger_desc = Vec::new();
    ledger_desc.push("credit user account".to_string());

    perform_wallet_transaction(
        event,
        pool,
        decimal_amount,
        account_id,
        user_ctx,
        EntryType::Credit,
        cassandra_session,
        app_cxt,
        ledger_desc,
    )
    .await
}

async fn charge_commission(
    acct_id: &str,
    amount: Decimal,
    beneficiary_account_id: &str,
    redis_conn: &mut ConnectionManager,
    db_tx: &mut Transaction<'_, Postgres>,
) -> Result<(), OrchestrateError> {
    if amount.is_sign_negative() || amount.is_sign_positive() {
        return Err(OrchestrateError::InvalidArgument(
            "amount must be positive".to_string(),
        ));
    }

    let user_acct = find_account_by_id(&mut **db_tx, &acct_id).await?;
    if user_acct.locked
        || user_acct.status == AccountStatus::Frozen
        || user_acct.status == AccountStatus::Inactive
    {
        return Err(OrchestrateError::InvalidRecordState(
            "the user's account is locked/frozen/inactive".to_string(),
        ));
    }

    let system_acct = find_account_by_id(&mut **db_tx, &beneficiary_account_id).await?;
    let amount_to_save = convert_amount(
        &mut **db_tx,
        amount,
        Currency::ADA,
        system_acct.currency,
        redis_conn,
    )
    .await?;

    credit_wallet_holding(&mut *db_tx, amount_to_save, system_acct.id).await?;
    Ok(())
}
