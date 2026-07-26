//! Concrete [`LogProvider`] implementation backed by `antikythera_log`.
//!
//! Bridges the port traits defined in [`crate::application::ports::logging`]
//! to the session-based logger registry maintained by `antikythera_log`.

use antikythera_log::session_logger;
use antikythera_log::{LogEntry, LogFilter, Logger};
use std::sync::Arc;

use crate::application::ports::logging::{AppLogger, LogProvider, LogQueryPort};

/// Adapter that wraps antikythera_log's Logger to implement the AppLogger port.
///
/// Delegates each log-level method to the inner [`Logger`] instance obtained
/// from the session logger registry.
struct LogProviderAdapter(Arc<Logger>);

/// Delegates info-level log calls to the underlying `antikythera_log::Logger`.
impl AppLogger for LogProviderAdapter {
    fn log_info(&self, message: String) {
        self.0.info(message);
    }

    fn log_warn(&self, message: String) {
        self.0.warn(message);
    }

    fn log_error(&self, message: String) {
        self.0.error(message);
    }

    fn log_debug(&self, message: String) {
        self.0.debug(message);
    }
}

/// Concrete LogProvider wrapping antikythera_log's session_logger.
///
/// Provides session-scoped logger instances and delegates log storage and
/// querying to the global `antikythera_log::session_logger` registry.
pub struct AntikytheraLogProvider;

/// Obtains or creates a per-session [`AppLogger`] via the session logger registry.
impl LogProvider for AntikytheraLogProvider {
    fn get_logger(&self, session_id: &str) -> Box<dyn AppLogger> {
        Box::new(LogProviderAdapter(session_logger::get_logger(session_id)))
    }

    fn logger_count(&self) -> usize {
        session_logger::logger_count()
    }

    fn clear_all_loggers(&self) {
        session_logger::clear_all_loggers();
    }
}

/// Queries historical log entries from the session logger registry.
impl LogQueryPort for AntikytheraLogProvider {
    fn query_logs(&self, session_id: &str, filter: &LogFilter) -> Vec<LogEntry> {
        let logger = session_logger::get_logger(session_id);
        let batch = logger.get_logs(filter);
        batch.entries
    }

    fn get_latest_logs(&self, session_id: &str, count: usize) -> Vec<LogEntry> {
        let logger = session_logger::get_logger(session_id);
        logger.get_latest(count)
    }

    fn get_logs_json(&self, session_id: &str, filter: &LogFilter) -> Result<String, String> {
        let logger = session_logger::get_logger(session_id);
        logger.get_logs_json(filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_instantiation() {
        let _provider = AntikytheraLogProvider;
    }

    #[test]
    fn test_logger_count_returns_number() {
        let provider = AntikytheraLogProvider;
        let count = provider.logger_count();
        assert!(count >= 0);
    }

    #[test]
    fn test_get_logger_returns_logger() {
        let provider = AntikytheraLogProvider;
        let logger = provider.get_logger("test-session");
        // Logger should be usable without panic
        logger.log_info("test message".to_string());
    }

    #[test]
    fn test_clear_all_loggers_no_panic() {
        let provider = AntikytheraLogProvider;
        provider.clear_all_loggers();
        // Should not panic
    }
}
