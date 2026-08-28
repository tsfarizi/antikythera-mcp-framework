//! Unified entry point for all security subsystems.

use antikythera_domain::security::SecurityConfig;

#[cfg(feature = "validation")]
use crate::validation::InputValidator;

#[cfg(feature = "rate-limit")]
use crate::rate_limit::RateLimiter;

#[cfg(feature = "memory")]
use crate::secrets::SecretManager;

/// Facade providing access to all security components from a single config.
pub struct SecurityFacade {
    #[cfg(feature = "validation")]
    pub validator: InputValidator,

    #[cfg(feature = "rate-limit")]
    pub rate_limiter: RateLimiter,

    #[cfg(feature = "memory")]
    pub secret_store: SecretManager,
}

impl SecurityFacade {
    /// Build all subsystems from a single `SecurityConfig`.
    ///
    /// Each subsystem is only constructed when its feature flag is enabled.
    pub fn from_config(config: SecurityConfig) -> Result<Self, crate::error::SecurityError> {
        #[cfg(feature = "validation")]
        let validator = InputValidator::new(config.validation)
            .map_err(|e| crate::error::SecurityError::Validation(e.to_string()))?;

        #[cfg(feature = "rate-limit")]
        let rate_limiter = RateLimiter::new(config.rate_limit);

        #[cfg(feature = "memory")]
        let secret_store = SecretManager::new(config.secrets)
            .map_err(|e| crate::error::SecurityError::Secret(e.to_string()))?;

        Ok(Self {
            #[cfg(feature = "validation")]
            validator,
            #[cfg(feature = "rate-limit")]
            rate_limiter,
            #[cfg(feature = "memory")]
            secret_store,
        })
    }
}
