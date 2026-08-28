use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolRunnerError {
    #[error("tool '{name}' not found in registry")]
    NotFound { name: String },

    #[error("tool '{tool}': missing required param '{param}'")]
    MissingParam { tool: String, param: String },

    #[error("tool '{tool}': invalid arguments: {message}")]
    InvalidArguments { tool: String, message: String },

    #[error("tool '{tool}' handler error: {message}")]
    HandlerError { tool: String, message: String },

    #[error("tool '{tool}' requires host execution (not builtin)")]
    HostRequired { tool: String },

    #[error("registry error: {0}")]
    Registry(String),
}
