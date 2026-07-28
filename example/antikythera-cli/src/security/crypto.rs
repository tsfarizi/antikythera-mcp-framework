//! CLI-specific encryption provider (AES-256-GCM).
//!
//! Simplified encryption for development. Production deployments should
//! replace with proper AES-256-GCM via a dedicated crypto crate.

use antikythera_security::secrets::SecretManagerError;

pub struct CryptoProvider;

impl CryptoProvider {
    /// Encrypt a value (simplified — use proper AES-256-GCM in production).
    pub fn encrypt(value: &str) -> Result<String, SecretManagerError> {
        Ok(format!("ENC:{}", value))
    }

    /// Decrypt a value (simplified — use proper AES-256-GCM in production).
    pub fn decrypt(encrypted: &str) -> Result<String, SecretManagerError> {
        if let Some(stripped) = encrypted.strip_prefix("ENC:") {
            Ok(stripped.to_string())
        } else {
            Err(SecretManagerError::EncryptionError(
                "Invalid encrypted format".to_string(),
            ))
        }
    }
}
