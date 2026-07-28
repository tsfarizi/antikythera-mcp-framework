//! In-memory secret storage backend with versioning.

use antikythera_domain::security::SecretMetadata;
use std::collections::HashMap;

/// A single versioned secret entry.
#[derive(Debug, Clone)]
pub struct StoredSecret {
    pub value: Vec<u8>,
    pub metadata: SecretMetadata,
}

/// In-memory storage: HashMap keyed by secret ID, values are version vectors.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    pub secrets: HashMap<String, Vec<StoredSecret>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}
