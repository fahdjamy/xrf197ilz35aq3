use crate::core::{AuditLog, EntryType};
use crate::error::OrchestrateError;
use crate::storage::{find_audit_logs, save_audit_log};
use sqlx::{Executor, Postgres, Transaction};

pub async fn create_new_audit(
    audit_log: AuditLog,
    db_tx: &mut Transaction<'_, Postgres>,
) -> Result<bool, OrchestrateError> {
    let saved = save_audit_log(db_tx, audit_log).await?;
    Ok(saved)
}

pub async fn fetch_audit_history<'a, E>(
    pool: E,
    entity_id: &str,
    entry_type: EntryType,
) -> Result<Vec<AuditLog>, OrchestrateError>
where
    E: Executor<'a, Database = Postgres>,
{
    let saved_logs = find_audit_logs(pool, entity_id, &entry_type.to_string()).await?;
    Ok(saved_logs)
}
