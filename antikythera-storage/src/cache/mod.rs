pub mod entry;

use std::collections::{HashMap, VecDeque};

use entry::CacheEntry;

/// Eviction policy for the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Evict least-recently-used entries when capacity is reached.
    Lru,
    /// Evict entries that have exceeded their TTL.
    Ttl,
    /// Apply both LRU capacity eviction and TTL expiration.
    Both,
}

impl From<&str> for EvictionPolicy {
    fn from(s: &str) -> Self {
        match s {
            "lru" => Self::Lru,
            "ttl" => Self::Ttl,
            "both" => Self::Both,
            _ => Self::Both,
        }
    }
}

/// In-memory session cache with TTL and LRU eviction.
///
/// Sessions are loaded on demand from the backend and held in RAM while
/// actively used. The cache evicts entries based on the configured policy
/// to bound memory consumption.
pub struct CacheManager {
    entries: HashMap<String, CacheEntry>,
    /// Access order deque: front = LRU, back = MRU.
    access_order: VecDeque<String>,
    max_sessions: usize,
    ttl_seconds: u64,
    eviction_policy: EvictionPolicy,
}

impl CacheManager {
    /// Create a new cache with the given capacity and TTL.
    pub fn new(max_sessions: usize, ttl_seconds: u64, eviction_policy: EvictionPolicy) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            max_sessions,
            ttl_seconds,
            eviction_policy,
        }
    }

    /// Get session data from cache, updating access time.
    pub fn get(&mut self, session_id: &str) -> Option<Vec<u8>> {
        let expired = self
            .entries
            .get(session_id)
            .map(|e| self.is_expired(e))
            .unwrap_or(false);

        if expired {
            self.evict_entry(session_id);
            return None;
        }

        if let Some(entry) = self.entries.get_mut(session_id) {
            entry.touch();
            let data = entry.data.clone();
            let _ = entry;
            self.move_to_back(session_id);
            Some(data)
        } else {
            None
        }
    }

    /// Insert or update a session in the cache.
    pub fn insert(&mut self, session_id: String, data: Vec<u8>) {
        if let Some(entry) = self.entries.get_mut(&session_id) {
            entry.data = data;
            entry.touch();
            entry.mark_dirty();
            self.move_to_back(&session_id);
            return;
        }

        // Evict if at capacity.
        while self.entries.len() >= self.max_sessions {
            self.evict_lru();
        }

        self.access_order.push_back(session_id.clone());
        self.entries
            .insert(session_id.clone(), CacheEntry::new(session_id, data));
    }

    /// Remove a session from the cache.
    pub fn remove(&mut self, session_id: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.entries.remove(session_id) {
            self.access_order.retain(|id| id != session_id);
            Some(entry.data)
        } else {
            None
        }
    }

    /// Check if a session exists in cache.
    pub fn contains(&self, session_id: &str) -> bool {
        self.entries.contains_key(session_id)
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all dirty session IDs that need backup.
    pub fn dirty_sessions(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.is_dirty)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Mark a session as clean (backup completed).
    pub fn mark_clean(&mut self, session_id: &str) {
        if let Some(entry) = self.entries.get_mut(session_id) {
            entry.mark_clean();
        }
    }

    /// Get the data for a dirty session without removing it.
    pub fn get_dirty_data(&self, session_id: &str) -> Option<Vec<u8>> {
        self.entries
            .get(session_id)
            .filter(|e| e.is_dirty)
            .map(|e| e.data.clone())
    }

    /// Run TTL expiration sweep, returning evicted session IDs.
    pub fn sweep_expired(&mut self) -> Vec<String> {
        let ttl_seconds = self.ttl_seconds;
        let policy = self.eviction_policy;
        let expired: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| {
                matches!(policy, EvictionPolicy::Ttl | EvictionPolicy::Both)
                    && entry.is_expired(ttl_seconds)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in &expired {
            self.evict_entry(id);
        }

        expired
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }

    // -- internal helpers --

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        matches!(
            self.eviction_policy,
            EvictionPolicy::Ttl | EvictionPolicy::Both
        ) && entry.is_expired(self.ttl_seconds)
    }

    fn move_to_back(&mut self, session_id: &str) {
        if let Some(pos) = self.access_order.iter().position(|id| id == session_id) {
            self.access_order.remove(pos);
        }
        self.access_order.push_back(session_id.to_string());
    }

    fn evict_lru(&mut self) {
        if let Some(lru_id) = self.access_order.pop_front() {
            self.entries.remove(&lru_id);
        }
    }

    fn evict_entry(&mut self, session_id: &str) {
        self.entries.remove(session_id);
        self.access_order.retain(|id| id != session_id);
    }
}
