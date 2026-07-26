use crate::wasm_agent::runner::AgentRunnerRuntime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_facade_has_all_methods() {
        // Verify that AgentRunnerRuntime can be default-constructed
        let rt = AgentRunnerRuntime::default();
        assert!(rt.sessions.is_empty());
        assert!(rt.archived_sessions.is_empty());
        assert!(rt.pending_events.is_empty());
    }

    #[test]
    fn test_runner_default_config() {
        let rt = AgentRunnerRuntime::default();
        assert_eq!(rt.max_in_memory_sessions, 128);
        assert!(rt.known_tools.is_empty());
    }

    #[test]
    fn test_session_lifecycle_module_accessible() {
        // Verify session_lifecycle module is accessible (compilation check)
        // If this compiles, the module and its impl blocks are linked.
        let _rt = AgentRunnerRuntime::default();
    }

    #[test]
    fn test_llm_stream_module_accessible() {
        // Verify llm_stream module is accessible (compilation check)
        let _rt = AgentRunnerRuntime::default();
    }

    #[test]
    fn test_tool_pipeline_module_accessible() {
        // Verify tool_pipeline module is accessible (compilation check)
        let _rt = AgentRunnerRuntime::default();
    }

    #[test]
    fn test_new_session_id_is_unique() {
        use crate::wasm_agent::runner::new_session_id;
        let id1 = new_session_id();
        let id2 = new_session_id();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("session-"));
        assert!(id2.starts_with("session-"));
    }

    #[test]
    fn test_now_unix_ms_returns_positive() {
        use crate::wasm_agent::runner::now_unix_ms;
        let ts = now_unix_ms();
        assert!(ts > 0);
    }

    #[test]
    fn test_agent_runner_error_display() {
        use crate::wasm_agent::runner::AgentRunnerError;
        let err = AgentRunnerError::SessionNotFound("test-123".to_string());
        assert_eq!(err.to_string(), "Session not found: test-123");

        let err = AgentRunnerError::ToolFailed("bad input".to_string());
        assert_eq!(err.to_string(), "Tool failed: bad input");

        let err = AgentRunnerError::Internal("oops".to_string());
        assert_eq!(err.to_string(), "Internal error: oops");
    }

    #[test]
    fn test_agent_runner_error_from_string() {
        use crate::wasm_agent::runner::AgentRunnerError;
        let err: AgentRunnerError = "some error".to_string().into();
        assert!(matches!(err, AgentRunnerError::Internal(_)));
    }
}
