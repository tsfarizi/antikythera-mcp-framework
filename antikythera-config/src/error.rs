use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when loading or validating configuration
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration file not found at {path:?}")]
    NotFound { path: PathBuf },

    #[error("failed to read config from {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse config from {path:?}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("missing required field 'model' in configuration")]
    MissingModel,

    #[error("missing required field 'default_provider' in configuration")]
    MissingDefaultProvider,

    #[error("missing required field 'prompt_template' in configuration")]
    MissingPromptTemplate,

    #[error("configuration cache error: {0}")]
    CacheError(String),

    #[error(
        "config schema changed; existing file at {path:?} is unreadable: {reason}. A backup was saved to {backup_path:?}"
    )]
    SchemaChanged {
        path: PathBuf,
        backup_path: PathBuf,
        reason: String,
    },
}
