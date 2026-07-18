//! Core Logging Module
//!
//! Centralized logging for antikythera-core.
//! All log entries automatically include the source module.
//!
//! ## Architecture
//!
//! This module provides **typed module loggers** that wrap the underlying
//! `antikythera_log::Logger` with automatic source tagging. Each subsystem
//! (agent, transport, provider, etc.) has its own logger type that ensures
//! every log entry is annotated with its origin.
//!
//! The global `LOGGERS` registry is owned by `antikythera-log` (in
//! `antikythera_log::session_logger`) so that any crate depending on
//! `antikythera-log` can share the same per-session buffers. This module
//! re-exports the registry helpers for backward compatibility and layers on
//! the typed module loggers below.

use antikythera_log::{LogBatch, LogEntry, LogFilter, LogLevel, Logger};
use std::sync::{Arc, LazyLock};

// ============================================================================
// Global Logger Registry
// ============================================================================
//
// `get_logger` / `clear_all_loggers` / `logger_count` are re-exported from
// `antikythera_log::session_logger` below. The actual `LOGGERS` static
// lives in `antikythera-log` so any downstream crate (notably
// `antikythera-session`) can share the same buffers without a cyclic
// dependency through this crate.

pub use antikythera_log::session_logger::{
    SessionLogger, clear_all_loggers, get_logger, logger_count,
};

/// Active session key used by the logging bridge.
/// Defaults to "tui" so events land in a predictable bucket.
static ACTIVE_SESSION: LazyLock<std::sync::Mutex<String>> =
    LazyLock::new(|| std::sync::Mutex::new("tui".to_string()));

/// Set the session key that the logging bridge writes events to.
/// Call this from the TUI when a session becomes active.
pub fn set_active_session(session_id: &str) {
    let mut s = ACTIVE_SESSION
        .lock()
        .expect("ACTIVE_SESSION lock poisoned in set_active_session");
    *s = session_id.to_string();
}

/// Get the current active session key.
pub fn get_active_session() -> String {
    ACTIVE_SESSION
        .lock()
        .expect("ACTIVE_SESSION lock poisoned in get_active_session")
        .clone()
}

// ============================================================================
// Module Logger Base Trait
// ============================================================================

/// Helper macro to define a module logger struct with standard inherent methods.
macro_rules! define_module_logger {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident => $source:literal
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug)]
        $vis struct $name {
            logger: Arc<Logger>,
        }

        impl $name {
            pub fn new(session_id: &str) -> Self {
                Self {
                    logger: get_logger(session_id),
                }
            }

            pub fn debug(&self, message: impl Into<String>) {
                self.logger.log_with_source(LogLevel::Debug, $source, message);
            }

            pub fn info(&self, message: impl Into<String>) {
                self.logger.log_with_source(LogLevel::Info, $source, message);
            }

            pub fn warn(&self, message: impl Into<String>) {
                self.logger.log_with_source(LogLevel::Warn, $source, message);
            }

            pub fn error(&self, message: impl Into<String>) {
                self.logger.log_with_source(LogLevel::Error, $source, message);
            }
        }
    };
}

// ============================================================================
// Module Loggers
// ============================================================================

define_module_logger! {
    /// Config module logger
    pub struct ConfigLogger => "config"
}

define_module_logger! {
    /// Agent module logger — covers FSM runner, agent runner, parser, context
    pub struct AgentLogger => "agent"
}

impl AgentLogger {
    /// Log a tool call with structured context
    pub fn tool_call(&self, tool: &str, args: &serde_json::Value) {
        let context = format!("{{\"tool\": \"{}\", \"args\": {}}}", tool, args);
        self.logger
            .log_with_context(LogLevel::Info, format!("Tool call: {}", tool), context)
    }

    /// Log a tool execution result
    pub fn tool_result(&self, tool: &str, success: bool, step: u32) {
        let level = if success {
            LogLevel::Info
        } else {
            LogLevel::Error
        };
        let context = format!(
            "{{\"tool\": \"{}\", \"success\": {}, \"step\": {}}}",
            tool, success, step
        );
        self.logger.log_with_context(
            level,
            format!("Tool result: {} (step {})", tool, step),
            context,
        )
    }

    /// Log an agent step (debug level)
    pub fn agent_step(&self, step: u32, max_steps: u32) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "agent",
            format!("Agent step {}/{}", step, max_steps),
        )
    }

    /// Log agent completion
    pub fn agent_complete(&self, steps: u32) {
        self.logger.log_with_source(
            LogLevel::Info,
            "agent",
            format!("Agent completed in {} steps", steps),
        )
    }
}

define_module_logger! {
    /// Transport module logger — covers HTTP, SSE, RPC, process management
    pub struct TransportLogger => "transport"
}

impl TransportLogger {
    pub fn connect(&self, server: &str) {
        self.logger.log_with_source(
            LogLevel::Info,
            "transport",
            format!("Connecting to: {}", server),
        );
    }

    pub fn disconnect(&self, server: &str) {
        self.logger.log_with_source(
            LogLevel::Info,
            "transport",
            format!("Disconnected from: {}", server),
        );
    }

    pub fn tool_request(&self, server: &str, tool: &str) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "transport",
            format!("Tool request: {}.{} ", server, tool),
        );
    }

    pub fn tool_response(&self, server: &str, tool: &str, success: bool) {
        let level = if success {
            LogLevel::Debug
        } else {
            LogLevel::Error
        };
        self.logger.log_with_source(
            level,
            "transport",
            format!("Tool response: {}.{} (success: {})", server, tool, success),
        );
    }
}

define_module_logger! {
    /// Provider module logger — covers model provider API calls
    pub struct ProviderLogger => "provider"
}

impl ProviderLogger {
    pub fn api_call(&self, provider: &str, model: &str) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "provider",
            format!("API call: {} ({})", provider, model),
        );
    }

    pub fn api_response(&self, provider: &str, model: &str, tokens: Option<u32>) {
        let token_info = tokens
            .map(|t| format!(", {} tokens", t))
            .unwrap_or_default();
        self.logger.log_with_source(
            LogLevel::Debug,
            "provider",
            format!("API response: {} ({}){}", provider, model, token_info),
        );
    }

    pub fn api_error(&self, provider: &str, error: &str) {
        self.logger.log_with_source(
            LogLevel::Error,
            "provider",
            format!("API error: {} ({})", provider, error),
        );
    }
}

define_module_logger! {
    /// Discovery module logger — covers server discovery, scanning, loading
    pub struct DiscoveryLogger => "discovery"
}

define_module_logger! {
    /// STDIO module logger — covers STDIO command processing
    pub struct StdioLogger => "stdio"
}

define_module_logger! {
    /// Chat service module logger
    pub struct ChatLogger => "chat"
}

define_module_logger! {
    /// WASM runtime module logger
    pub struct WasmLogger => "wasm"
}

define_module_logger! {
    /// Resilience module logger — covers retry, circuit breaker, etc.
    pub struct ResilienceLogger => "resilience"
}

// `SessionLogger` is defined in `antikythera-log` (`session_logger` module)
// so that it can be reused by `antikythera-session` without a cyclic
// dependency. It is re-exported from `antikythera_log` and this module above.

define_module_logger! {
    /// Orchestrator module logger — covers multi-agent orchestration
    pub struct OrchestratorLogger => "orchestrator"
}

define_module_logger! {
    /// Streaming module logger — covers LLM streaming
    pub struct StreamingLogger => "streaming"
}

impl StreamingLogger {
    /// Log a streaming session start
    pub fn session_start(&self, mode: &str, session_id: &str) {
        self.logger.log_with_source(
            LogLevel::Info,
            "streaming",
            format!(
                "Streaming session started | mode={} session_id={}",
                mode, session_id
            ),
        );
    }

    /// Log a streaming session end
    pub fn session_end(&self, session_id: &str, total_events: usize) {
        self.logger.log_with_source(
            LogLevel::Info,
            "streaming",
            format!(
                "Streaming session ended | session_id={} total_events={}",
                session_id, total_events
            ),
        );
    }

    /// Log a token stream
    pub fn token_emitted(&self, session_id: &str, content_len: usize) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "streaming",
            format!(
                "Token emitted | session_id={} content_len={}",
                session_id, content_len
            ),
        );
    }

    /// Log buffer flush
    pub fn buffer_flushed(&self, session_id: &str, event_count: usize) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "streaming",
            format!(
                "Buffer flushed | session_id={} event_count={}",
                session_id, event_count
            ),
        );
    }

    /// Log buffer overflow
    pub fn buffer_overflow(&self, session_id: &str, dropped: usize) {
        self.logger.log_with_source(
            LogLevel::Warn,
            "streaming",
            format!(
                "Buffer overflow | session_id={} dropped={}",
                session_id, dropped
            ),
        );
    }

    /// Log a tool event
    pub fn tool_event(&self, session_id: &str, tool_name: &str, phase: &str) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "streaming",
            format!(
                "Tool event | session_id={} tool={} phase={}",
                session_id, tool_name, phase
            ),
        );
    }

    /// Log a streaming error
    pub fn stream_error(&self, session_id: &str, error: &str) {
        self.logger.log_with_source(
            LogLevel::Error,
            "streaming",
            format!("Stream error | session_id={} error={}", session_id, error),
        );
    }
}

define_module_logger! {
    /// Observability module logger — covers telemetry, metrics, audit, tracing
    pub struct ObservabilityLogger => "observability"
}

define_module_logger! {
    /// Security module logger — covers rate limiting, secrets, validation
    pub struct SecurityLogger => "security"
}

impl SecurityLogger {
    pub fn rate_limit_check(&self, session_id: &str, allowed: bool) {
        let level = if allowed {
            LogLevel::Debug
        } else {
            LogLevel::Warn
        };
        self.logger.log_with_source(
            level,
            "security",
            format!(
                "Rate limit check | session={} allowed={}",
                session_id, allowed
            ),
        );
    }

    pub fn rate_limit_exceeded(&self, session_id: &str, reason: &str) {
        self.logger.log_with_source(
            LogLevel::Warn,
            "security",
            format!(
                "Rate limit exceeded | session={} reason={}",
                session_id, reason
            ),
        );
    }

    pub fn secret_stored(&self, id: &str) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "security",
            format!("Secret stored | id={}", id),
        );
    }

    pub fn secret_retrieved(&self, id: &str) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "security",
            format!("Secret retrieved | id={}", id),
        );
    }

    pub fn secret_rotated(&self, id: &str) {
        self.logger.log_with_source(
            LogLevel::Info,
            "security",
            format!("Secret rotated | id={}", id),
        );
    }

    pub fn secret_deleted(&self, id: &str) {
        self.logger.log_with_source(
            LogLevel::Info,
            "security",
            format!("Secret deleted | id={}", id),
        );
    }

    pub fn secret_error(&self, id: &str, error: &str) {
        self.logger.log_with_source(
            LogLevel::Error,
            "security",
            format!("Secret error | id={} error={}", id, error),
        );
    }

    pub fn cleanup_task(&self, action: &str) {
        self.logger.log_with_source(
            LogLevel::Debug,
            "security",
            format!("Cleanup task | action={}", action),
        );
    }
}

// ============================================================================
// Log Query API
// ============================================================================
//
// The query helpers below all use the shared logger registry from
// `antikythera_log::session_logger` so that callers can read logs written
// by any crate in the workspace (including `antikythera-session`).

/// Query logs across all modules
pub fn query_logs(session_id: &str, filter: &LogFilter) -> LogBatch {
    get_logger(session_id).get_logs(filter)
}

/// Get latest logs across all modules
pub fn get_latest_logs(session_id: &str, count: usize) -> Vec<LogEntry> {
    get_logger(session_id).get_latest(count)
}

/// Get logs as JSON
pub fn get_logs_json(session_id: &str, filter: &LogFilter) -> Result<String, String> {
    get_logger(session_id).get_logs_json(filter)
}

/// Subscribe to real-time log stream
pub fn subscribe_logs(session_id: &str) -> Option<antikythera_log::LogSubscriber> {
    Some(get_logger(session_id).subscribe())
}

/// Clear logs for a session
pub fn clear_logs(session_id: &str) {
    get_logger(session_id).clear();
}

// ============================================================================
// AppLogger Port Implementations
// ============================================================================
//
// These implementations satisfy the `AppLogger` trait port, allowing
// application code to depend on the trait instead of concrete logger types.
// This follows the Dependency Inversion Principle.

macro_rules! impl_app_logger {
    ($ty:ty) => {
        impl crate::application::ports::logging::AppLogger for $ty {
            fn log_info(&self, message: impl Into<String>) { self.info(message); }
            fn log_warn(&self, message: impl Into<String>) { self.warn(message); }
            fn log_error(&self, message: impl Into<String>) { self.error(message); }
            fn log_debug(&self, message: impl Into<String>) { self.debug(message); }
        }
    };
}

impl_app_logger!(ConfigLogger);
impl_app_logger!(AgentLogger);
impl_app_logger!(TransportLogger);
impl_app_logger!(ProviderLogger);
impl_app_logger!(DiscoveryLogger);
impl_app_logger!(StdioLogger);
impl_app_logger!(ChatLogger);
impl_app_logger!(WasmLogger);
impl_app_logger!(ResilienceLogger);
impl_app_logger!(OrchestratorLogger);
impl_app_logger!(StreamingLogger);
impl_app_logger!(ObservabilityLogger);
impl_app_logger!(SecurityLogger);
impl_app_logger!(SessionLogger);
