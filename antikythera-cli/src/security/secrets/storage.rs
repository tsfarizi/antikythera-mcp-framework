//! Secret Storage Logic

use antikythera_core::security::config::SecretMetadata;
use std::collections::HashMap;

/// Stored secret with metadata
#[derive(Debug, Clone)]
pub struct StoredSecret {
    pub value: String,
    pub metadata: SecretMetadata,
}

/// Secret storage backend
#[derive(Debug)]
pub enum SecretStorage {
    Memory {
        secrets: HashMap<String, Vec<StoredSecret>>,
    },
    File {
        secrets: HashMap<String, Vec<StoredSecret>>,
        path: String,
    },
}
