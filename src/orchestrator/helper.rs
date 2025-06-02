use crate::error::OrchestrateError;
use crate::PgDatabaseError;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::log;

pub async fn rollback_db_transaction(
    tx: Transaction<'_, Postgres>,
    event: &str,
) -> Result<(), OrchestrateError> {
    tx.rollback().await.map_err(|err| {
        let err_msg = "failed to rollback transaction";
        log::error!("event={} :: msg='{}' :: err={}", event, err_msg, err);
        OrchestrateError::DatabaseError(PgDatabaseError::TransactionStepError(format!(
            "{} :: err={}",
            err_msg, err
        )))
    })?;
    Ok(())
}

pub async fn start_db_transaction(
    pool: &PgPool,
    event: &str,
) -> Result<Transaction<'static, Postgres>, OrchestrateError> {
    pool.begin().await.map_err(|err| {
        let err_msg = "failed to start a new transaction";
        log::error!("event={} :: msg='{}' :: err={}", event, err_msg, err);
        OrchestrateError::ServerError(format!("{} :: err={}", err_msg, err))
    })
}

pub async fn commit_db_transaction(
    tx: Transaction<'_, Postgres>,
    event: &str,
) -> Result<(), OrchestrateError> {
    tx.commit().await.map_err(|err| {
        let err_msg = "failed to commit transaction";
        log::error!("event={} :: msg='{}' :: err={}", event, err_msg, err);
        OrchestrateError::DatabaseError(PgDatabaseError::TransactionStepError(format!(
            "{} :: err={}",
            err_msg, err
        )))
    })
}
