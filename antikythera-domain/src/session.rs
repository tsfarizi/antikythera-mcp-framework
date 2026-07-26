//! Session and Message Types
//!
//! Canonical session domain types living in core so that the session crate
//! can re-export them without creating a cyclic dependency.

use super::message_types::Message;
use serde::{Deserialize, Serialize};

// ============================================================================
// Session Entity
// ============================================================================

/// Chat session with full history and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID (UUID v4)
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Model used
    pub model: String,
    /// Session title (auto-generated or user-set)
    pub title: Option<String>,
    /// Message history
    pub messages: Vec<Message>,
    /// Session metadata (JSON encoded)
    pub metadata: Option<String>,
    /// Created timestamp
    pub created_at: String,
    /// Last updated timestamp
    pub updated_at: String,
    /// Total token usage
    pub tokens_used: u64,
    /// Total steps in agent flow
    pub total_steps: u32,
    /// Tools used in this session
    pub tools_used: Vec<String>,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: impl Into<String>, model: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            user_id: user_id.into(),
            model: model.into(),
            title: None,
            messages: Vec::new(),
            metadata: None,
            created_at: now.clone(),
            updated_at: now,
            tokens_used: 0,
            total_steps: 0,
            tools_used: Vec::new(),
        }
    }

    /// Add a message
    pub fn add_message(&mut self, message: Message) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
        self.messages.push(message);
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get latest message
    pub fn latest_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Set title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Add token usage
    pub fn add_tokens(&mut self, tokens: u64) {
        self.tokens_used += tokens;
    }

    /// Record tool usage
    pub fn record_tool(&mut self, tool_name: &str, step: u32) {
        self.total_steps = step;
        if !self.tools_used.contains(&tool_name.to_string()) {
            self.tools_used.push(tool_name.to_string());
        }
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.total_steps = 0;
        self.tools_used.clear();
    }
}

// ============================================================================
// Session Summary
// ============================================================================

/// Lightweight session summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub user_id: String,
    pub model: String,
    pub title: Option<String>,
    pub message_count: usize,
    pub total_steps: u32,
    pub tools_used: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Session> for SessionSummary {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id.clone(),
            user_id: session.user_id.clone(),
            model: session.model.clone(),
            title: session.title.clone(),
            message_count: session.messages.len(),
            total_steps: session.total_steps,
            tools_used: session.tools_used.clone(),
            created_at: session.created_at.clone(),
            updated_at: session.updated_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_types::Message;

    #[test]
    fn session_new_initializes_correctly() {
        let s = Session::new("user-1", "gpt-4");
        assert!(!s.id.is_empty());
        assert_eq!(s.user_id, "user-1");
        assert_eq!(s.model, "gpt-4");
        assert!(s.title.is_none());
        assert!(s.messages.is_empty());
        assert_eq!(s.tokens_used, 0);
        assert_eq!(s.total_steps, 0);
        assert!(s.tools_used.is_empty());
    }

    #[test]
    fn session_add_message_increments_count_and_updates_timestamp() {
        let mut s = Session::new("u", "m");
        let before = s.updated_at.clone();
        s.add_message(Message::user("hello"));
        assert_eq!(s.messages.len(), 1);
        assert!(s.updated_at >= before);
    }

    #[test]
    fn session_clear_messages_resets_state() {
        let mut s = Session::new("u", "m");
        s.add_message(Message::user("a"));
        s.add_message(Message::assistant("b"));
        s.record_tool("search", 1);
        s.clear_messages();
        assert!(s.messages.is_empty());
        assert_eq!(s.total_steps, 0);
        assert!(s.tools_used.is_empty());
    }

    #[test]
    fn session_record_tool_deduplicates() {
        let mut s = Session::new("u", "m");
        s.record_tool("search", 1);
        s.record_tool("search", 2);
        assert_eq!(s.tools_used.len(), 1);
        assert_eq!(s.total_steps, 2);
    }

    #[test]
    fn session_summary_from_ref() {
        let mut s = Session::new("u", "m");
        s.set_title("My Session");
        s.add_message(Message::user("hi"));
        let summary = SessionSummary::from(&s);
        assert_eq!(summary.id, s.id);
        assert_eq!(summary.message_count, 1);
        assert_eq!(summary.title.as_deref(), Some("My Session"));
    }

    #[test]
    fn session_serialization_roundtrip() {
        let mut s = Session::new("u", "m");
        s.add_message(Message::user("hello"));
        let json = serde_json::to_string(&s).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, s.id);
        assert_eq!(restored.messages.len(), 1);
    }
}
