//! Port: Logging
//!
//! Application code depends on this trait for logging.
//! Concrete implementations are provided by the logging framework.
//!
//! This follows the Dependency Inversion Principle: application defines
//! the interface, and infrastructure (logging module) implements it.

/// Minimal logging interface that application code can depend on.
/// Decouples application from specific logger implementations.
pub trait AppLogger: Send + Sync {
    fn log_info(&self, message: impl Into<String>);
    fn log_warn(&self, message: impl Into<String>);
    fn log_error(&self, message: impl Into<String>);
    fn log_debug(&self, message: impl Into<String>);
}