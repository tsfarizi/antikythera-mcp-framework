//! Session lifecycle management for WASM agent runner.
//!
//! Handles session CRUD, archival, idle sweeping, and capacity enforcement
//! for the in-memory session store on [`AgentRunnerRuntime`].

use antikythera_log::LogLevel;

use super::runner_types::*;
use super::{now_unix_ms, wasm_log, AgentRunnerError, AgentRunnerRuntime, SessionRuntime};
use crate::wasm_agent::types::StreamEventKind;

/// Session lifecycle operations: archive, sweep, enforce capacity, and ensure.
impl AgentRunnerRuntime {
    /// Archives a single session, removing it from the active store.
    ///
    /// Moves the session's state to the archived map, emits a
    /// [`StreamEventKind::SessionArchived`] event, and returns `Ok(true)` if
    /// the session existed, or `Ok(false)` if it was already absent.
    pub(super) fn archive_session(
        &mut self,
        session_id: &str,
        reason: &str,
        correlation_id: Option<String>,
    ) -> Result<bool, AgentRunnerError> {
        wasm_log(
            session_id,
            LogLevel::Info,
            &format!("Session archived: {reason}"),
        );
        let Some(runtime) = self.sessions.remove(session_id) else {
            return Ok(false);
        };

        let archived_at_ms = now_unix_ms();
        let state_json = runtime.state.to_json()?;

        wasm_log(
            session_id,
            LogLevel::Info,
            &format!(
                "Archiving session in FSM state: {}",
                runtime.state.fsm_state
            ),
        );

        self.archived_sessions.insert(
            session_id.to_string(),
            ArchivedSessionRecord {
                archived_at_ms,
                reason: reason.to_string(),
            },
        );

        self.emit_pending_event(
            session_id,
            StreamEventKind::SessionArchived,
            correlation_id,
            serde_json::json!({
                "reason": reason,
                "archived_at_ms": archived_at_ms,
                "last_touched_ms": runtime.last_touched_ms,
                "state_json": state_json,
                "message_count": runtime.state.message_history.len(),
                "step": runtime.state.current_step,
            }),
        );

        Ok(true)
    }

    /// Archives all sessions that have exceeded their idle timeout.
    ///
    /// Sessions with pending LLM chunks are skipped. Returns the count of
    /// sessions that were archived.
    pub(super) fn sweep_idle_sessions(&mut self, now_ms: i64) -> Result<u32, AgentRunnerError> {
        let candidates: Vec<String> = self
            .sessions
            .iter()
            .filter_map(|(id, session)| {
                if !session.pending_llm_chunks.is_empty() {
                    return None;
                }
                let timeout_ms = i64::from(session.state.config.session_timeout_secs) * 1_000;
                if timeout_ms <= 0 {
                    return None;
                }
                if now_ms.saturating_sub(session.last_touched_ms) > timeout_ms {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut archived = 0_u32;
        for session_id in candidates {
            if self.archive_session(&session_id, "idle_timeout", None)? {
                archived += 1;
            }
        }
        if archived > 0 {
            wasm_log(
                "runtime",
                LogLevel::Info,
                &format!("Idle sweep archived {archived} sessions"),
            );
        }
        Ok(archived)
    }

    /// Enforces the maximum in-memory session capacity by archiving the
    /// least-recently-used sessions.
    ///
    /// The session identified by `protected_session_id` (if any) is never
    /// evicted. Sessions with pending LLM chunks are also skipped. Returns
    /// the count of sessions archived.
    pub(super) fn enforce_capacity(
        &mut self,
        protected_session_id: Option<&str>,
        correlation_id: Option<String>,
    ) -> Result<u32, AgentRunnerError> {
        if self.max_in_memory_sessions == 0 {
            return Ok(0);
        }

        let mut archived = 0_u32;
        while self.sessions.len() > self.max_in_memory_sessions {
            let candidate = self
                .sessions
                .iter()
                .filter(|(id, session)| {
                    if let Some(protected) = protected_session_id
                        && id.as_str() == protected
                    {
                        return false;
                    }
                    session.pending_llm_chunks.is_empty()
                })
                .min_by_key(|(_, session)| session.last_touched_ms)
                .map(|(id, _)| id.clone());

            let Some(candidate_id) = candidate else {
                break;
            };

            if self.archive_session(&candidate_id, "capacity_pressure", correlation_id.clone())? {
                archived += 1;
            } else {
                break;
            }
        }

        if archived > 0 {
            wasm_log(
                "runtime",
                LogLevel::Info,
                &format!("Capacity pressure archived {archived} sessions"),
            );
        }
        Ok(archived)
    }

    /// Returns a mutable reference to the session, creating it from the
    /// default config if it does not yet exist.
    pub(super) fn ensure_session(&mut self, session_id: &str) -> &mut SessionRuntime {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| {
                let mut config = self.default_config.clone();
                config.session_id = session_id.to_string();
                SessionRuntime::new(config)
            })
    }

    /// Convenience wrapper that sweeps idle sessions using the current time
    /// or an explicitly provided timestamp.
    pub(super) fn sweep_sessions(&mut self, now_ms: Option<i64>) -> Result<u32, AgentRunnerError> {
        let now = now_ms.unwrap_or_else(now_unix_ms);
        self.sweep_idle_sessions(now)
    }
}
