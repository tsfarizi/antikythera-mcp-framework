pub mod routes;
pub mod middleware;

use axum::Router;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::StorageEngine;

/// Build the REST API router for standalone storage service.
pub fn build_router(engine: Arc<Mutex<StorageEngine>>) -> Router {
    Router::new()
        .nest("/api", routes::api_routes(engine))
}
