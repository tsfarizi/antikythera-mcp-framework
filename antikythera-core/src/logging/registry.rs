use antikythera_log::{LogBatch, LogEntry, LogFilter};

pub use antikythera_log::session_logger::{
    SessionLogger, clear_all_loggers, get_logger, logger_count,
};

static ACTIVE_SESSION: LazyLock<std::sync::Mutex<String>> =
    LazyLock::new(|| std::sync::Mutex::new("tui".to_string()));

pub fn set_active_session(session_id: &str) {
    let mut s = ACTIVE_SESSION
        .lock()
        .expect("ACTIVE_SESSION lock poisoned in set_active_session");
    *s = session_id.to_string();
}

pub fn get_active_session() -> String {
    ACTIVE_SESSION
        .lock()
        .expect("ACTIVE_SESSION lock poisoned in get_active_session")
        .clone()
}

pub fn query_logs(session_id: &str, filter: &LogFilter) -> LogBatch {
    get_logger(session_id).get_logs(filter)
}

pub fn get_latest_logs(session_id: &str, count: usize) -> Vec<LogEntry> {
    get_logger(session_id).get_latest(count)
}

pub fn get_logs_json(session_id: &str, filter: &LogFilter) -> Result<String, String> {
    get_logger(session_id).get_logs_json(filter)
}

pub fn subscribe_logs(session_id: &str) -> Option<antikythera_log::LogSubscriber> {
    Some(get_logger(session_id).subscribe())
}

pub fn clear_logs(session_id: &str) {
    get_logger(session_id).clear();
}

use std::sync::LazyLock;
