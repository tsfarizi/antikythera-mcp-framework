use std::time::Instant;

/// A single cached session entry with metadata for TTL and LRU tracking.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Session identifier.
    pub session_id: String,

    /// Serialized session data (JSON bytes).
    pub data: Vec<u8>,

    /// When this entry was created in cache.
    pub created_at: Instant,

    /// When this entry was last accessed.
    pub last_accessed: Instant,

    /// Whether this entry has unsaved changes that need backup.
    pub is_dirty: bool,
}

impl CacheEntry {
    /// Create a new cache entry.
    pub fn new(session_id: String, data: Vec<u8>) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            data,
            created_at: now,
            last_accessed: now,
            is_dirty: false,
        }
    }

    /// Mark entry as dirty (needs backup).
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Mark entry as clean (backup completed).
    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    /// Update last access time and return self for chaining.
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }

    /// Check if entry has expired given a TTL in seconds.
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        self.last_accessed.elapsed().as_secs() > ttl_seconds
    }

    /// Seconds since last access.
    pub fn idle_seconds(&self) -> u64 {
        self.last_accessed.elapsed().as_secs()
    }
}
