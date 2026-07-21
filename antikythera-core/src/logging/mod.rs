//! Core Logging Module
//!
//! Centralized logging for antikythera-core.
//! All log entries automatically include the source module.
//!
//! ## Architecture
//!
//! This module provides **typed module loggers** that wrap the underlying
//! `antikythera_log::Logger` with automatic source tagging. Each subsystem
//! (agent, transport, provider, etc.) has its own logger type that ensures
//! every log entry is annotated with its origin.
//!
//! The global `LOGGERS` registry is owned by `antikythera-log` (in
//! `antikythera_log::session_logger`) so that any crate depending on
//! `antikythera-log` can share the same per-session buffers. This module
//! re-exports the registry helpers for backward compatibility and layers on
//! the typed module loggers below.

mod module_loggers;
mod registry;

pub use registry::*;
pub use module_loggers::*;
