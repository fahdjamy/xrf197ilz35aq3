use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Error)]
pub enum DomainError {
    #[error("{0}")]
    ParseError(String),
    #[error("{0}")]
    InvalidArgument(String),
}
