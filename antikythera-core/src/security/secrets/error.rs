//! Secret Manager Errors

use thiserror::Error;

/// Secret manager error
#[derive(Debug, Clone, Error)]
pub enum SecretManagerError {
    #[error("Secret not found: {0}")]
    SecretNotFound(String),
    #[error("Secret expired: {0}")]
    SecretExpired(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}
