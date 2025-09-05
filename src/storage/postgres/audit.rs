use crate::core::{AuditEventType, AuditLog};
use crate::PgDatabaseError;
use sqlx::{Executor, Postgres};
use tracing::info;

#[tracing::instrument(level = "debug", skip(pg_pool, audit_log), name = "Create audit log")]
pub async fn create_audit_log<'a, E>(
    pg_pool: E,
    audit_log: AuditLog,
) -> Result<bool, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    info!(
        "creating new audit log :: entryId={} :: entityType={}",
        &audit_log.entity_id, audit_log.entity_type
    );

    let result = sqlx::query!(
        "
INSERT INTO audit_log (
                       id,
                       changes,
                       user_fp,
                       entity_id,
                       entity_type,
                       audit_type,
                       request_ip,
                       request_id,
                       creation_time,
                       request_user_agent
                       )
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        audit_log.id,
        audit_log.changes,
        audit_log.user_fp,
        audit_log.entity_id,
        audit_log.entity_type.clone().to_string(),
        audit_log.audit_type as AuditEventType,
        audit_log.request_ip,
        audit_log.request_id,
        audit_log.creation_time,
        audit_log.request_user_agent
    )
    .execute(pg_pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(
    level = "debug",
    skip(pg_pool, entity_id, entity_type),
    name = "Find audit logs"
)]
pub async fn find_audit_logs<'a, E>(
    pg_pool: E,
    entity_id: &str,
    entity_type: &str,
) -> Result<Vec<AuditLog>, PgDatabaseError>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = sqlx::query_as!(
        AuditLog,
        r#"
SELECT id,
       changes,
       user_fp,
       entity_id,
       entity_type,
       request_ip,
       request_id,
       creation_time,
       request_user_agent,
       audit_type as "audit_type: _"
FROM audit_log
WHERE entity_id = $1
    AND entity_type = $2"#,
        entity_id,
        entity_type
    )
    .fetch_all(pg_pool)
    .await?;
    unimplemented!()
}
