#[derive(Debug, Clone, thiserror::Error)]
pub enum ObservabilityError {
    #[error("Metrics error: {0}")]
    Metrics(String),
    #[error("Tracing error: {0}")]
    Tracing(String),
    #[error("Audit error: {0}")]
    Audit(String),
    #[error("Configuration error: {0}")]
    Config(String),
}
