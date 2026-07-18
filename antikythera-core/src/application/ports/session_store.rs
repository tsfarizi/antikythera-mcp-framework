//! Port: Session Store
//!
//! Application defines this interface. Infrastructure provides the
//! in-memory LRU implementation backed by antikythera-session.

use crate::domain::types::ChatMessage;

/// Port trait for session message storage.
/// Application code depends only on this trait.
pub trait SessionStore: Send + Sync {
    /// Get chat history for a session.
    fn get(&self, session_id: &str) -> Option<Vec<ChatMessage>>;

    /// Ensure a session exists.
    fn touch_or_create(&mut self, session_id: &str);

    /// Replace full history for a session.
    fn replace_history(&mut self, session_id: &str, messages: Vec<ChatMessage>);

    /// Append messages to a session.
    fn push_messages(
        &mut self,
        session_id: &str,
        messages: impl IntoIterator<Item = ChatMessage>,
    );
}
