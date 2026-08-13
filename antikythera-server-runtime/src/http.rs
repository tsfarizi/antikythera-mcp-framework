//! Axum HTTP server implementing the wire protocol endpoints exactly:
//!
//! - `POST /antikythera/v1/llm/call`
//! - `POST /antikythera/v1/tools/execute`
//! - `GET  /antikythera/v1/tools`
//! - `GET  /antikythera/v1/events?client_id=...&session_id=...` (SSE)
//! - `POST /antikythera/v1/events/{correlation-id}/response` (POST-back)
//!
//! The state endpoints are NOT implemented (decision d, D2 — reserved
//! future). Gate denials return HTTP 403 with `{"error": "permission: ..."}`.

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::CorsLayer;

use crate::wire::{LlmRequest, PostbackBody, ToolCallEvent, ToolDefinition};
use crate::wit::SharedState;

/// Build the wire-protocol router.
pub fn router(shared: Arc<SharedState>) -> Router {
    Router::new()
        .route("/antikythera/v1/llm/call", post(llm_call))
        .route("/antikythera/v1/tools/execute", post(tools_execute))
        .route("/antikythera/v1/tools", get(tools_list))
        .route("/antikythera/v1/events", get(events_sse))
        .route(
            "/antikythera/v1/events/{correlation_id}/response",
            post(postback),
        )
        .layer(CorsLayer::permissive())
        .with_state(shared)
}

fn permission_response(message: String) -> Response {
    (StatusCode::FORBIDDEN, Json(json!({"error": message}))).into_response()
}

fn bad_request(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": message.into()})),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// POST /antikythera/v1/llm/call
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct LlmCallQuery {
    #[serde(default)]
    pub stream: bool,
}

async fn llm_call(
    State(shared): State<Arc<SharedState>>,
    Query(query): Query<LlmCallQuery>,
    Json(request): Json<LlmRequest>,
) -> Response {
    if let Err(e) = shared.check_llm_gate(request.session_id.as_deref()) {
        return permission_response(e);
    }
    let provider = match shared.resolve_provider(request.provider.as_deref()) {
        Ok(provider) => provider,
        Err(e) => return bad_request(e),
    };

    if query.stream {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let control = shared.control.clone();
        let client_id = shared.client_id.clone();
        let session_id = request.session_id.clone();
        tokio::spawn(async move {
            while let Some(chunk) = rx.recv().await {
                control.push_llm_token(&client_id, session_id.as_deref(), &chunk);
            }
        });
        match provider.call_stream(request, tx).await {
            Ok(response) => Json(response).into_response(),
            Err(e) => bad_request(e.to_string()),
        }
    } else {
        match provider.call(request).await {
            Ok(response) => Json(response).into_response(),
            Err(e) => bad_request(e.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// POST /antikythera/v1/tools/execute
// ---------------------------------------------------------------------------

async fn tools_execute(
    State(shared): State<Arc<SharedState>>,
    Json(event): Json<ToolCallEvent>,
) -> Response {
    match shared.router.execute_server_owned(&event).await {
        Ok(result) => Json(result).into_response(),
        Err(e) if e.starts_with("permission:") => permission_response(e),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e}))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// GET /antikythera/v1/tools
// ---------------------------------------------------------------------------

async fn tools_list(State(shared): State<Arc<SharedState>>) -> Json<Vec<ToolDefinition>> {
    Json(shared.router.peer_definitions())
}

// ---------------------------------------------------------------------------
// GET /antikythera/v1/events  (SSE control channel)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub client_id: String,
    pub session_id: Option<String>,
}

async fn events_sse(
    State(shared): State<Arc<SharedState>>,
    Query(query): Query<EventsQuery>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    if query.client_id.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let rx = shared.control.register_client(&query.client_id);
    let _ = shared.control.push_lifecycle(&query.client_id, "connected");

    let seq = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let stream = BroadcastStream::new(rx).map(move |item| {
        let n = seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let json = match item {
            Ok(json) => json,
            Err(_) => "{}".to_string(),
        };
        Ok::<_, Infallible>(Event::default().id(n.to_string()).data(json))
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    ))
}

// ---------------------------------------------------------------------------
// POST /antikythera/v1/events/{correlation-id}/response
// ---------------------------------------------------------------------------

async fn postback(
    State(shared): State<Arc<SharedState>>,
    Path(correlation_id): Path<String>,
    Json(body): Json<PostbackBody>,
) -> StatusCode {
    if body.correlation_id != correlation_id {
        tracing::warn!(
            correlation_id = %correlation_id,
            "postback correlation id does not match path; ignoring"
        );
        return StatusCode::NO_CONTENT;
    }
    let completed = shared.control.complete_postback(body);
    if !completed {
        tracing::warn!(
            correlation_id = %correlation_id,
            "postback for unknown or expired correlation id; ignoring"
        );
    }
    StatusCode::NO_CONTENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_query_requires_client_id() {
        // `client_id` has no default: missing it is a deserialization error.
        let query: Result<EventsQuery, _> = serde_urlencoded::from_str("session_id=s1");
        assert!(query.is_err());
        let query: EventsQuery = serde_urlencoded::from_str("client_id=c1&session_id=s1").unwrap();
        assert_eq!(query.client_id, "c1");
        assert_eq!(query.session_id.as_deref(), Some("s1"));
    }

    #[test]
    fn llm_call_query_stream_defaults_false() {
        let query: LlmCallQuery = serde_urlencoded::from_str("").unwrap();
        assert!(!query.stream);
        let query: LlmCallQuery = serde_urlencoded::from_str("stream=true").unwrap();
        assert!(query.stream);
    }
}
