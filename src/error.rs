use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Error)]
pub enum DomainError {
    #[error("{0}")]
    ParseError(String),
    #[error("{0}")]
    InvalidArgument(String),
    #[error("{0}")]
    InvalidState(String),
}

#[derive(Debug, Error)]
pub enum PgDatabaseError {
    #[error("row not found")]
    NotFound,
    #[error("row with id already exists")]
    UniqueViolation,
    #[error("foreign key violation")]
    ForeignKeyViolation,
    #[error("`{0}`")]
    RecordExists(String),
    #[error("`{0}`")]
    InvalidRecordState(String),
    #[error("`{0}`")]
    TransactionStepError(String),
    // ... other specific database errors
    #[error("`{0}`")]
    Configuration(String), // To capture configuration errors
    #[error("`{0}`")]
    Tls(String), // To capture TLS errors
    #[error("`{0}`")]
    Protocol(String), // To capture protocol errors
    #[error("`{0}`")]
    Encode(String), // To capture encoding errors
    #[error("`{0}`")]
    Decode(String), // To capture decoding errors
    #[error("DB pool timeout")]
    PoolTimedOut,
    #[error("DB pool closed")]
    PoolClosed,
    #[error("DB worker crashed")]
    WorkerCrashed,
    #[error("`{0}`")]
    InvalidArgument(String),
    #[error("`{0}`")]
    Unknown(String), // Catch-all for other errors with the error message
}

impl From<sqlx::Error> for PgDatabaseError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => PgDatabaseError::NotFound,
            sqlx::Error::Database(e) => {
                if let Some(code) = e.code() {
                    match code.as_ref() {
                        "23505" => PgDatabaseError::UniqueViolation,
                        "23503" => PgDatabaseError::ForeignKeyViolation,
                        // ... other specific database error code mappings
                        _ => PgDatabaseError::Unknown(e.to_string()), // Capture the error message
                    }
                } else {
                    PgDatabaseError::Unknown(e.to_string()) // Capture the error message
                }
            }
            sqlx::Error::Configuration(e) => PgDatabaseError::Configuration(e.to_string()),
            sqlx::Error::Tls(e) => PgDatabaseError::Tls(e.to_string()),
            sqlx::Error::Protocol(e) => PgDatabaseError::Protocol(e),
            sqlx::Error::Encode(e) => PgDatabaseError::Encode(e.to_string()),
            sqlx::Error::Decode(e) => PgDatabaseError::Decode(e.to_string()),
            sqlx::Error::PoolTimedOut => PgDatabaseError::PoolTimedOut,
            sqlx::Error::PoolClosed => PgDatabaseError::PoolClosed,
            sqlx::Error::WorkerCrashed => PgDatabaseError::WorkerCrashed,
            // ... other SqlxError variants you want to handle
            _ => PgDatabaseError::Unknown(e.to_string()), // Catch-all for other errors
        }
    }
}

#[derive(Debug, Error)]
pub enum OrchestrateError {
    #[error("`{0}`")]
    ServerError(String),
    #[error("`{0}`")]
    NotFoundError(String),
    #[error("`{0}`")]
    InvalidArgument(String),
    #[error("`{0}`")]
    DatabaseError(#[from] PgDatabaseError),
    #[error("`{0}`")]
    InvalidRecordState(String),
    #[error("`{0}`")]
    RowConstraintViolation(String),
}

impl OrchestrateError {
    pub fn error_code(&self) -> u16 {
        match self {
            OrchestrateError::ServerError(_) => 500,
            OrchestrateError::DatabaseError(pg_err) => match pg_err {
                PgDatabaseError::NotFound => 404,
                PgDatabaseError::UniqueViolation => 409,
                PgDatabaseError::RecordExists(_) => 409,
                PgDatabaseError::ForeignKeyViolation => 409,
                _ => 500,
            },
            OrchestrateError::NotFoundError(_) => 404,
            OrchestrateError::InvalidArgument(_) => 400,
            OrchestrateError::InvalidRecordState(_) => 400,
            OrchestrateError::RowConstraintViolation(_) => 400,
        }
    }
}

#[derive(Debug, Error)]
pub enum CassandraDBError {
    #[error("`{0}`")]
    Unknown(String),
    #[error("`{0}`")]
    InvalidArgument(String),
    #[error("`{0}`")]
    NotFound(String),
    #[error("`{0}`")]
    ServerError(String),
    #[error("`{0}`")]
    ExecutionError(String),
    #[error("`{0}`")]
    SetValueError(String),
}
