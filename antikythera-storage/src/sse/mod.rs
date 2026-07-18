//! SSE backup service for independent backup processing.
//!
//! Runs as a separate microservice that receives backup events via
//! HTTP and persists them to the configured storage backend.

/// HTTP handlers for backup events.
pub mod handler;

use std::net::SocketAddr;

/// Start the SSE backup service as an independent process.
pub async fn start_sse_server(
    bind: &str,
    backup_dir: std::path::PathBuf,
) -> Result<(), crate::error::StorageError> {
    let addr: SocketAddr = bind
        .parse()
        .map_err(|e| crate::error::StorageError::Config(format!("invalid bind address: {e}")))?;

    let app = handler::build_sse_router(backup_dir);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::error::StorageError::Connection(format!("failed to bind: {e}")))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::StorageError::Connection(format!("server error: {e}")))?;

    Ok(())
}
