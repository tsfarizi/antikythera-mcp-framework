use antikythera_log::SessionLogger;
use antikythera_ports::session_store::SessionStore as SessionStoreTrait;

use crate::domain::types::ChatMessage;

/// Decorator that wraps any [`SessionStore`] implementation and adds
/// structured logging via [`SessionLogger`] for every mutation and read.
///
/// The inner store handles persistence; this layer only observes and records.
///
/// This is a public API decorator for consumers who want observability
/// on session store operations without modifying the inner implementation.
#[allow(dead_code)]
pub struct LoggedSessionStore<S: SessionStoreTrait> {
    inner: S,
}

impl<S: SessionStoreTrait> LoggedSessionStore<S> {
    /// Wrap `inner` with structured logging for all store operations.
    #[allow(dead_code)]
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S: SessionStoreTrait> SessionStoreTrait for LoggedSessionStore<S> {
    fn get(&self, session_id: &str) -> Option<Vec<ChatMessage>> {
        let result = self.inner.get(session_id);
        if let Some(ref messages) = result {
            SessionLogger::new(session_id).debug(format!(
                "Session history retrieved | messages={}",
                messages.len()
            ));
        }
        result
    }

    fn touch_or_create(&mut self, session_id: &str) {
        self.inner.touch_or_create(session_id);
        SessionLogger::new(session_id).info("Session touched/created");
    }

    fn replace_history(&mut self, session_id: &str, messages: Vec<ChatMessage>) {
        let count = messages.len();
        self.inner.replace_history(session_id, messages);
        SessionLogger::new(session_id)
            .info(format!("Session history replaced | messages={}", count));
    }

    fn push_messages(&mut self, session_id: &str, messages: impl IntoIterator<Item = ChatMessage>) {
        let messages: Vec<ChatMessage> = messages.into_iter().collect();
        let count = messages.len();
        self.inner.push_messages(session_id, messages);
        SessionLogger::new(session_id)
            .debug(format!("Messages pushed to session | count={}", count));
    }
}
