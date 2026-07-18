//! HTTP request/response types for the SSE backup service.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request body for backup events.
#[derive(Debug, Deserialize)]
pub struct BackupEvent {
    /// Session identifier.
    pub session_id: String,
    /// Serialized session data.
    pub data: Vec<u8>,
}

/// Response for backup operations.
#[derive(Debug, Serialize)]
pub struct BackupResponse {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Human-readable status message.
    pub message: String,
}

/// Build the SSE backup router with backup and health endpoints.
pub fn build_sse_router(backup_dir: PathBuf) -> Router {
    Router::new()
        .route("/backup/:id", axum::routing::post(handle_backup))
        .route("/health", axum::routing::get(health_check))
        .with_state(backup_dir)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "sse-backup" }))
}

async fn handle_backup(
    State(backup_dir): State<PathBuf>,
    Path(id): Path<String>,
    Json(event): Json<BackupEvent>,
) -> Result<Json<BackupResponse>, StatusCode> {
    let backup_path = backup_dir.join(format!("{id}.backup.json"));
    let data = event.data.clone();

    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(&backup_dir)?;
        std::fs::write(&backup_path, data)?;
        Ok::<_, std::io::Error>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(BackupResponse {
        success: true,
        message: format!("backup saved for session {id}"),
    }))
}
