//! Application modules (CLI-owned).
//!
//! These are application-level orchestration modules that use core's
//! agent management system and port traits.

pub mod discovery;
pub mod prompt_composer;
#[cfg(feature = "native-transport")]
pub mod stdio;
pub mod session_store;
