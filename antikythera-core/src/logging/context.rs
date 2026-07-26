//! Propagates session ID through execution context instead of a global static.
//!
//! This module provides [`SessionContext`], a lightweight wrapper that carries
//! a session identifier through the call stack so that logging and telemetry
//! can attribute entries to the correct session without relying on
//! thread-local or global state.

/// Propagates session ID through execution context instead of global static.
///
/// Wraps a session identifier and provides ergonomic methods for resolving it,
/// including a fallback to the global active session when the context is empty.
///
/// # Examples
///
/// ```
/// use antikythera_core::logging::context::SessionContext;
///
/// let ctx = SessionContext::new("sess-123");
/// assert_eq!(ctx.session_id(), "sess-123");
///
/// let owned: String = ctx.into_session_id();
/// assert_eq!(owned, "sess-123");
/// ```
#[derive(Debug, Clone)]
pub struct SessionContext {
    session_id: String,
}

impl SessionContext {
    /// Creates a new `SessionContext` from any string-like value.
    ///
    /// The `session_id` is converted into an owned `String`.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
        }
    }

    /// Fallback to global active session if no context available.
    ///
    /// If the stored session ID is empty, delegates to
    /// [`super::get_active_session`] to obtain the currently active session.
    pub fn or_active(self) -> String {
        if self.session_id.is_empty() {
            super::get_active_session()
        } else {
            self.session_id
        }
    }

    /// Returns a borrowed reference to the session ID string.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Consumes the context and returns the owned session ID string.
    pub fn into_session_id(self) -> String {
        self.session_id
    }
}

impl Default for SessionContext {
    /// Creates a default context by reading the global active session.
    fn default() -> Self {
        Self::new(super::get_active_session())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_string_literal() {
        let ctx = SessionContext::new("test-session");
        assert_eq!(ctx.session_id(), "test-session");
    }

    #[test]
    fn test_new_with_string() {
        let ctx = SessionContext::new(String::from("dynamic-session"));
        assert_eq!(ctx.session_id(), "dynamic-session");
    }

    #[test]
    fn test_default_uses_global_fallback() {
        let ctx = SessionContext::default();
        // Default should produce a non-empty session ID (from global fallback)
        assert!(!ctx.session_id().is_empty());
    }

    #[test]
    fn test_or_active_returns_stored_id() {
        let ctx = SessionContext::new("my-session");
        let result = ctx.or_active();
        assert_eq!(result, "my-session");
    }

    #[test]
    fn test_or_active_empty_falls_back() {
        let ctx = SessionContext::new("");
        let result = ctx.or_active();
        // Empty string should trigger fallback to global
        assert!(!result.is_empty());
    }

    #[test]
    fn test_into_session_id() {
        let ctx = SessionContext::new("consume-me");
        let owned = ctx.into_session_id();
        assert_eq!(owned, "consume-me");
    }

    #[test]
    fn test_clone_independence() {
        let original = SessionContext::new("original");
        let cloned = original.clone();
        assert_eq!(original.session_id(), cloned.session_id());
    }
}