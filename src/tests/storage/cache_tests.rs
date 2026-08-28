use std::time::Duration;

use antikythera_storage::cache::CacheManager;
use antikythera_storage::cache::EvictionPolicy;

#[test]
fn test_cache_insert_and_get() {
    let mut cache = CacheManager::new(10, 3600, EvictionPolicy::Both);
    let data = b"session data".to_vec();

    cache.insert("session1".to_string(), data.clone());
    assert_eq!(cache.get("session1"), Some(data));
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_cache_lru_eviction() {
    let mut cache = CacheManager::new(2, 3600, EvictionPolicy::Lru);

    cache.insert("a".to_string(), b"1".to_vec());
    cache.insert("b".to_string(), b"2".to_vec());
    // Cache is at capacity (2)
    assert_eq!(cache.len(), 2);

    // Access "a" to make "b" the LRU
    cache.get("a");

    // Insert "c" — should evict "b" (least recently used)
    cache.insert("c".to_string(), b"3".to_vec());
    assert_eq!(cache.len(), 2);
    assert!(cache.contains("a"));
    assert!(!cache.contains("b"));
    assert!(cache.contains("c"));
}

#[test]
fn test_cache_ttl_expiration() {
    // TTL=0 means entries expire once 1+ second has elapsed since last access
    let mut cache = CacheManager::new(10, 0, EvictionPolicy::Ttl);

    cache.insert("session1".to_string(), b"data".to_vec());
    std::thread::sleep(Duration::from_secs(1));

    assert_eq!(cache.get("session1"), None);
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_cache_remove() {
    let mut cache = CacheManager::new(10, 3600, EvictionPolicy::Both);

    cache.insert("session1".to_string(), b"data".to_vec());
    assert_eq!(cache.len(), 1);

    let removed = cache.remove("session1");
    assert_eq!(removed, Some(b"data".to_vec()));
    assert_eq!(cache.len(), 0);
    assert!(!cache.contains("session1"));

    // Removing non-existent key returns None
    assert_eq!(cache.remove("session1"), None);
}

#[test]
fn test_cache_dirty_tracking() {
    let mut cache = CacheManager::new(10, 3600, EvictionPolicy::Both);

    // New entry should not be dirty
    cache.insert("s1".to_string(), b"d1".to_vec());
    assert!(cache.dirty_sessions().is_empty());

    // Inserting into existing entry marks it dirty
    cache.insert("s1".to_string(), b"d2".to_vec());
    assert_eq!(cache.dirty_sessions().len(), 1);
    assert!(cache.dirty_sessions().contains(&"s1".to_string()));
}

#[test]
fn test_cache_mark_clean() {
    let mut cache = CacheManager::new(10, 3600, EvictionPolicy::Both);

    cache.insert("s1".to_string(), b"d1".to_vec());
    cache.insert("s1".to_string(), b"d2".to_vec());
    assert_eq!(cache.dirty_sessions().len(), 1);

    cache.mark_clean("s1");
    assert!(cache.dirty_sessions().is_empty());
}

#[test]
fn test_cache_sweep_expired() {
    // TTL=0 means entries expire once 1+ second has elapsed
    let mut cache = CacheManager::new(10, 0, EvictionPolicy::Both);

    cache.insert("a".to_string(), b"1".to_vec());
    cache.insert("b".to_string(), b"2".to_vec());
    assert_eq!(cache.len(), 2);

    std::thread::sleep(Duration::from_secs(1));

    let evicted = cache.sweep_expired();
    assert_eq!(evicted.len(), 2);
    assert!(evicted.contains(&"a".to_string()));
    assert!(evicted.contains(&"b".to_string()));
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_cache_sweep_only_removes_expired() {
    // Use Lru-only policy — sweep_expired should remove nothing
    let mut cache = CacheManager::new(10, 0, EvictionPolicy::Lru);

    cache.insert("a".to_string(), b"1".to_vec());
    let evicted = cache.sweep_expired();
    assert!(evicted.is_empty());
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_cache_clear() {
    let mut cache = CacheManager::new(10, 3600, EvictionPolicy::Both);

    cache.insert("a".to_string(), b"1".to_vec());
    cache.insert("b".to_string(), b"2".to_vec());
    assert_eq!(cache.len(), 2);

    cache.clear();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
fn test_cache_get_dirty_data() {
    let mut cache = CacheManager::new(10, 3600, EvictionPolicy::Both);

    // Not dirty initially
    cache.insert("s1".to_string(), b"original".to_vec());
    assert!(cache.get_dirty_data("s1").is_none());

    // Update makes it dirty
    cache.insert("s1".to_string(), b"updated".to_vec());
    assert_eq!(cache.get_dirty_data("s1"), Some(b"updated".to_vec()));
}

#[test]
fn test_cache_eviction_policy_from_str() {
    assert_eq!(EvictionPolicy::from("lru"), EvictionPolicy::Lru);
    assert_eq!(EvictionPolicy::from("ttl"), EvictionPolicy::Ttl);
    assert_eq!(EvictionPolicy::from("both"), EvictionPolicy::Both);
    // Unknown defaults to Both
    assert_eq!(EvictionPolicy::from("unknown"), EvictionPolicy::Both);
}
