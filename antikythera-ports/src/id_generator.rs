//! Port: ID Generator
//!
//! Application defines this interface. Infrastructure provides
//! UUID-based or other implementations.

/// Port trait for generating unique IDs.
/// Decouples application from specific ID generation libraries.
pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> String;
}

/// Default UUID v4 implementation.
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn new_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
