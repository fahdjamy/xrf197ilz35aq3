use crate::core::generate_timebase_str_id;
use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_event_type")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum AuditEventType {
    CREATE,
    DELETE,
    UPDATE,
}

impl Display for AuditEventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::CREATE => write!(f, "CREATE"),
            AuditEventType::DELETE => write!(f, "DELETE"),
            AuditEventType::UPDATE => write!(f, "UPDATE"),
        }
    }
}

// A generic struct to represent the data within the 'changes' JSONB column.
// Using generics here (`<T>`) to make this reusable for any entity type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeData<T> {
    // `Option` is used because 'old' is not present on 'create' events,
    // and 'new' is not present on 'delete' events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new: Option<T>,
}

#[derive(Serialize, Debug, Clone)]
pub enum EntityType {
    Account,
    Transaction,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Account => f.write_str("Account"),
            EntityType::Transaction => f.write_str("Transaction"),
        }
    }
}

impl From<String> for EntityType {
    fn from(s: String) -> Self {
        match s.as_ref() {
            "Account" => EntityType::Account,
            "Transaction" => EntityType::Transaction,
            _ => EntityType::Account,
        }
    }
}

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct AuditLog {
    pub id: String,
    pub user_fp: String,
    pub entity_id: String,
    pub entity_type: EntityType,

    pub changes: JsonValue,
    pub audit_type: AuditEventType,

    pub request_ip: Option<String>,
    pub request_id: Option<String>,
    pub creation_time: DateTime<Utc>,
    pub request_user_agent: Option<String>,
}

impl AuditLog {
    pub fn build<T: Serialize>(
        user_fp: String,
        entity_id: String,
        entity_type: EntityType,
        audit_type: AuditEventType,
        request_ip: Option<String>,
        request_id: Option<String>,
        req_user_agent: Option<String>,
        old: Option<T>,
        new: Option<T>,
    ) -> Result<Self, DomainError> {
        let change_data: ChangeData<T> = ChangeData { old, new };

        // 2. Serialize the change data into a generic JsonValue
        let changes_json = serde_json::to_value(change_data)
            .map_err(|err| DomainError::ParseError(err.to_string()))?;

        Ok(AuditLog {
            user_fp,
            entity_id,
            audit_type,
            entity_type,
            request_ip,
            request_id,
            changes: changes_json,
            creation_time: Utc::now(),
            id: generate_timebase_str_id(),
            request_user_agent: req_user_agent,
        })
    }
}

impl Display for AuditLog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "auditId={}, entityId={}, entryType={}, createdAt={}",
            self.id, self.entity_id, self.entity_type, self.creation_time
        )
    }
}
