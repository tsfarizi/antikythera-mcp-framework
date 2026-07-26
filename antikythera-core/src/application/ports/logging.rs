//! Port: Logging
//!
//! Defines the port traits that application code depends on for logging.
//! Concrete implementations are provided by the infrastructure logging
//! module (`antikythera_core::logging`).
//!
//! This follows the **Dependency Inversion Principle**: the application
//! defines the interface, and infrastructure (the logging module) supplies
//! the concrete [`LogProvider`](crate::logging::provider::AntikytheraLogProvider).
//!
//! ## Traits
//!
//! - [`AppLogger`] -- minimal per-session logger interface.
//! - [`LogProvider`] -- factory for obtaining an [`AppLogger`] by session ID.
//! - [`LogQueryPort`] -- read-only access to historical log entries.

use antikythera_log::{LogEntry, LogFilter};

/// Minimal logging interface that application code can depend on.
/// Decouples application from specific logger implementations.
pub trait AppLogger: Send + Sync {
    fn log_info(&self, message: String);
    fn log_warn(&self, message: String);
    fn log_error(&self, message: String);
    fn log_debug(&self, message: String);
}

/// Port for obtaining a Logger instance by session ID.
/// Implementations live in the concrete logging crate.
pub trait LogProvider: Send + Sync {
    fn get_logger(&self, session_id: &str) -> Box<dyn AppLogger>;
    fn logger_count(&self) -> usize;
    fn clear_all_loggers(&self);
}

/// Port for querying log entries.
pub trait LogQueryPort: Send + Sync {
    fn query_logs(&self, session_id: &str, filter: &LogFilter) -> Vec<LogEntry>;
    fn get_latest_logs(&self, session_id: &str, count: usize) -> Vec<LogEntry>;
    fn get_logs_json(&self, session_id: &str, filter: &LogFilter) -> Result<String, String>;
}