//! Port: Model Provider
//!
//! Application defines this port. Infrastructure implements it.
//! The core agent loop depends only on this trait, not on any concrete provider.

// Re-export the existing trait — it's already correctly placed in infrastructure.
// This module exists to make the port ownership explicit in the application layer.
pub use crate::infrastructure::model::traits::{ModelClient, ModelProvider};
pub use crate::infrastructure::model::types::{ModelError, ModelRequest, ModelResponse};
