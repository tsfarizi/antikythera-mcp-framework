use std::time::Duration;

use antikythera_storage::backend::StorageBackend;
use antikythera_storage::backend::filesystem::FilesystemBackend;
use antikythera_storage::config::StorageConfig;
use antikythera_storage::StorageEngine;
use tempfile::tempdir;

#[tokio::test]
async fn test_storage_engine_save_load_delete() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let backup_dir = dir.path().join("backups");

    let mut config = StorageConfig::default();
    config.data_dir = data_dir;
    config.backup_dir = backup_dir;
    // Disable backup so we don't need a full backup setup
    config.backup.enabled = false;

    let mut engine = StorageEngine::new(config).await.unwrap();

    let session_id = "integration-session";
    let data = b"{\"state\":\"active\",\"payload\":[1,2,3]}".to_vec();

    // Save
    engine.save(session_id, data.clone()).await.unwrap();

    // Load from cache (populated on save)
    let loaded = engine.load(session_id).await.unwrap();
    assert_eq!(loaded, Some(data.clone()));

    // Exists
    assert!(engine.exists(session_id).await.unwrap());

    // List should include our session
    let ids = engine.list().await.unwrap();
    assert!(ids.contains(&session_id.to_string()));

    // Delete
    engine.delete(session_id).await.unwrap();
    assert!(!engine.exists(session_id).await.unwrap());

    // Load after delete returns None
    let loaded = engine.load(session_id).await.unwrap();
    assert_eq!(loaded, None);
}

#[tokio::test]
async fn test_storage_engine_cache_behavior() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let backup_dir = dir.path().join("backups");

    let mut config = StorageConfig::default();
    config.data_dir = data_dir;
    config.backup_dir = backup_dir;
    config.backup.enabled = false;
    config.cache.enabled = true;
    config.cache.max_sessions = 4;
    config.cache.ttl_seconds = 3600;

    let mut engine = StorageEngine::new(config).await.unwrap();

    // Save multiple sessions
    for i in 0..4 {
        let id = format!("session-{i}");
        engine.save(&id, format!("data-{i}").into_bytes()).await.unwrap();
    }

    // Cache stats should reflect 4 entries
    let stats = engine.cache_stats();
    assert_eq!(stats.total_entries, 4);
    assert_eq!(stats.max_entries, 4);

    // Inserting a 5th should trigger LRU eviction
    engine.save("session-4", b"data-4".to_vec()).await.unwrap();
    let stats = engine.cache_stats();
    assert_eq!(stats.total_entries, 4);
}

#[tokio::test]
async fn test_storage_engine_multiple_sessions_lifecycle() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let backup_dir = dir.path().join("backups");

    let mut config = StorageConfig::default();
    config.data_dir = data_dir;
    config.backup_dir = backup_dir;
    config.backup.enabled = false;

    let mut engine = StorageEngine::new(config).await.unwrap();

    // Save sessions
    engine.save("s1", b"one".to_vec()).await.unwrap();
    engine.save("s2", b"two".to_vec()).await.unwrap();
    engine.save("s3", b"three".to_vec()).await.unwrap();

    let mut ids = engine.list().await.unwrap();
    ids.sort();
    assert_eq!(ids, vec!["s1", "s2", "s3"]);

    // Delete s2
    engine.delete("s2").await.unwrap();

    let mut ids = engine.list().await.unwrap();
    ids.sort();
    assert_eq!(ids, vec!["s1", "s3"]);

    // Update s1
    engine.save("s1", b"ONE_UPDATED".to_vec()).await.unwrap();
    let loaded = engine.load("s1").await.unwrap();
    assert_eq!(loaded, Some(b"ONE_UPDATED".to_vec()));
}

#[tokio::test]
async fn test_storage_engine_sweep_cache() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let backup_dir = dir.path().join("backups");

    let mut config = StorageConfig::default();
    config.data_dir = data_dir;
    config.backup_dir = backup_dir;
    config.backup.enabled = false;
    config.cache.ttl_seconds = 0; // Immediate expiration

    let mut engine = StorageEngine::new(config).await.unwrap();

    engine.save("s1", b"d1".to_vec()).await.unwrap();
    engine.save("s2", b"d2".to_vec()).await.unwrap();

    std::thread::sleep(Duration::from_secs(1));

    let evicted = engine.sweep_cache();
    assert_eq!(evicted.len(), 2);
}

#[tokio::test]
async fn test_filesystem_backend_direct_usage() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let backup_dir = dir.path().join("backups");
    let backend = FilesystemBackend::new(&data_dir, &backup_dir).unwrap();

    // Save and load
    backend.save("direct", b"direct-data").await.unwrap();
    assert_eq!(backend.load("direct").await.unwrap(), Some(b"direct-data".to_vec()));

    // Overwrite
    backend.save("direct", b"updated").await.unwrap();
    assert_eq!(backend.load("direct").await.unwrap(), Some(b"updated".to_vec()));

    // Delete
    backend.delete("direct").await.unwrap();
    assert_eq!(backend.load("direct").await.unwrap(), None);

    // Delete nonexistent is idempotent
    backend.delete("direct").await.unwrap();
}
