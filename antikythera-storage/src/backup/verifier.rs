use crate::backend::StorageBackend;
use crate::error::StorageError;

/// Verify that a session was successfully synced to primary storage.
pub async fn verify_sync(
    backend: &dyn StorageBackend,
    session_id: &str,
) -> Result<bool, StorageError> {
    backend.verify_sync(session_id).await
}

/// Verify and delete backup if sync was successful.
pub async fn verify_and_delete(
    backend: &dyn StorageBackend,
    session_id: &str,
) -> Result<bool, StorageError> {
    if verify_sync(backend, session_id).await? {
        backend.delete_backup(session_id).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
