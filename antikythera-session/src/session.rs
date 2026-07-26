//! Session and Message Types
//!
//! Canonical definitions now live in `antikythera_core::domain`.
//! This module re-exports them for backward compatibility.

pub use antikythera_domain::message_types::{Message, MessagePart, MessageRole};
pub use antikythera_domain::session::{Session, SessionSummary};
pub use antikythera_domain::session_manager::{SessionManager, SessionManagerError};
