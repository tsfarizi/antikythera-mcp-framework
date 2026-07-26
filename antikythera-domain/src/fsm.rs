//! Agent Finite State Machine
//!
//! Typed FSM that enforces valid state transitions in the agent loop.
//! Each variant corresponds to a concrete phase of execution;
//! illegal transitions are rejected at the call site rather than silently
//! corrupting runtime state.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Typed FSM states for the agent loop.
///
/// DESIGN NOTE: This enum is mirrored in antikythera-sdk/src/wasm_agent/types.rs
/// as `AgentFsmState`. The transition matrix MUST be kept in sync.
/// See: tests/sdk/wasm_agent/fsm_parity_tests.rs for parity verification.
///
/// Valid flow:
/// ```text
/// Idle → UserTurnPrepared → LlmStreaming → LlmCommitted
///   → ToolRequested → ToolResultProcessed → LlmStreaming (loop)
///   → ToolRequested → ToolResultProcessed → Final
///   → LlmCommitted → Final
///   → LlmCommitted → Idle (retry)
///   → ToolResultProcessed → Idle (retry)
///   → Final → Idle (new turn)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentFsmState {
    Idle,
    UserTurnPrepared,
    LlmStreaming,
    LlmCommitted,
    ToolRequested,
    ToolResultProcessed,
    Final,
}

impl AgentFsmState {
    /// Returns the initial state of a fresh agent loop.
    pub fn initial() -> Self {
        Self::Idle
    }

    /// Attempt a state transition. Returns `Err` if the transition is not
    /// permitted by the FSM definition.
    pub fn transition_to(&mut self, next: AgentFsmState) -> Result<(), FsmTransitionError> {
        if self.can_transition_to(&next) {
            *self = next;
            Ok(())
        } else {
            Err(FsmTransitionError {
                from: *self,
                to: next,
            })
        }
    }

    /// Check whether a transition from `self` to `next` is valid without
    /// mutating state.
    pub fn can_transition_to(&self, next: &AgentFsmState) -> bool {
        (self == &Self::Idle && next == &Self::UserTurnPrepared)
            || (self == &Self::UserTurnPrepared && next == &Self::LlmStreaming)
            || (self == &Self::LlmStreaming && next == &Self::LlmCommitted)
            || (self == &Self::LlmCommitted && next == &Self::ToolRequested)
            || (self == &Self::LlmCommitted && next == &Self::Final)
            || (self == &Self::LlmCommitted && next == &Self::Idle)
            || (self == &Self::ToolRequested && next == &Self::ToolResultProcessed)
            || (self == &Self::ToolResultProcessed && next == &Self::LlmStreaming)
            || (self == &Self::ToolResultProcessed && next == &Self::Final)
            || (self == &Self::ToolResultProcessed && next == &Self::Idle)
            || (self == &Self::Final && next == &Self::Idle)
    }
}

/// Display implementation: lowercase snake_case for log messages.
impl fmt::Display for AgentFsmState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => f.write_str("idle"),
            Self::UserTurnPrepared => f.write_str("user_turn_prepared"),
            Self::LlmStreaming => f.write_str("llm_streaming"),
            Self::LlmCommitted => f.write_str("llm_committed"),
            Self::ToolRequested => f.write_str("tool_requested"),
            Self::ToolResultProcessed => f.write_str("tool_result_processed"),
            Self::Final => f.write_str("final"),
        }
    }
}

/// Error returned when an invalid FSM transition is attempted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsmTransitionError {
    /// State the FSM was in when the transition was attempted.
    pub from: AgentFsmState,
    /// State the caller tried to transition to.
    pub to: AgentFsmState,
}

impl fmt::Display for FsmTransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid FSM transition: ")?;
        fmt::Display::fmt(&self.from, f)?;
        f.write_str(" → ")?;
        fmt::Display::fmt(&self.to, f)
    }
}

impl std::error::Error for FsmTransitionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_idle() {
        assert_eq!(AgentFsmState::initial(), AgentFsmState::Idle);
    }

    #[test]
    fn valid_happy_path_transitions() {
        let mut s = AgentFsmState::Idle;
        s.transition_to(AgentFsmState::UserTurnPrepared).unwrap();
        s.transition_to(AgentFsmState::LlmStreaming).unwrap();
        s.transition_to(AgentFsmState::LlmCommitted).unwrap();
        s.transition_to(AgentFsmState::ToolRequested).unwrap();
        s.transition_to(AgentFsmState::ToolResultProcessed).unwrap();
        s.transition_to(AgentFsmState::LlmStreaming).unwrap();
        s.transition_to(AgentFsmState::LlmCommitted).unwrap();
        s.transition_to(AgentFsmState::Final).unwrap();
        s.transition_to(AgentFsmState::Idle).unwrap();
    }

    #[test]
    fn invalid_transition_returns_error() {
        let mut s = AgentFsmState::Idle;
        let result = s.transition_to(AgentFsmState::LlmStreaming);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.from, AgentFsmState::Idle);
        assert_eq!(err.to, AgentFsmState::LlmStreaming);
    }

    #[test]
    fn can_transition_to_pure_check() {
        assert!(AgentFsmState::Idle.can_transition_to(&AgentFsmState::UserTurnPrepared));
        assert!(!AgentFsmState::Idle.can_transition_to(&AgentFsmState::Final));
    }

    #[test]
    fn committed_can_go_to_idle_or_final() {
        assert!(AgentFsmState::LlmCommitted.can_transition_to(&AgentFsmState::Idle));
        assert!(AgentFsmState::LlmCommitted.can_transition_to(&AgentFsmState::Final));
        assert!(AgentFsmState::LlmCommitted.can_transition_to(&AgentFsmState::ToolRequested));
    }

    #[test]
    fn display_produces_snake_case() {
        assert_eq!(AgentFsmState::Idle.to_string(), "idle");
        assert_eq!(
            AgentFsmState::ToolResultProcessed.to_string(),
            "tool_result_processed"
        );
    }

    #[test]
    fn transition_error_display() {
        let err = FsmTransitionError {
            from: AgentFsmState::Idle,
            to: AgentFsmState::Final,
        };
        let msg = err.to_string();
        assert!(msg.contains("idle"));
        assert!(msg.contains("final"));
    }
}
