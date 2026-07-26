//! REST API for standalone storage service.
//!
//! Provides HTTP endpoints for session CRUD operations when running
//! as a separate process.

/// Route definitions for session endpoints.
pub mod routes;

/// Request validation middleware.
pub mod middleware;

use axum::Router;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::StorageEngine;

/// Build the REST API router for standalone storage service.
pub fn build_router(engine: Arc<Mutex<StorageEngine>>) -> Router {
    Router::new().nest("/api", routes::api_routes(engine))
}
