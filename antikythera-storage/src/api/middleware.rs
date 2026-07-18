use axum::http::StatusCode;

/// Validate that the request has a valid session ID format.
pub fn validate_session_id(id: &str) -> Result<(), StatusCode> {
    if id.is_empty() || id.len() > 256 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}
