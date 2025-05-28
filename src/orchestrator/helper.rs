use crate::error::OrchestrateError;
use crate::PgDatabaseError;
use sqlx::{Postgres, Transaction};
use tracing::log;

pub async fn rollback_db_transaction(
    tx: Transaction<'_, Postgres>,
) -> Result<(), OrchestrateError> {
    tx.rollback().await.map_err(|err| {
        log::error!(
            "failed to rollback transaction for creating a new account: {}",
            err
        );
        OrchestrateError::DatabaseError(PgDatabaseError::TransactionStepError(format!(
            "failed to rollback transaction for creating a new account: {}",
            err
        )))
    })?;
    Ok(())
}
