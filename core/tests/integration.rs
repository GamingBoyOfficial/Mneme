use mneme_core::{
    MemoryStore, SqliteBackend, HashEmbedder, Embedder,
    MemoryType, AccessScope, RecallOptions,
};
use std::sync::Arc;

#[tokio::test]
async fn test_roundtrip_export_import() {
    let backend = SqliteBackend::new(":memory:").await.unwrap();
    let embedder = Arc::new(HashEmbedder::new(384));
    let store = MemoryStore::new("test-agent", Arc::new(backend), embedder).await;

    store.remember(
        "User prefers dark mode",
        MemoryType::Semantic,
        "user1", "session1", "test", 1.0, None, vec!["preference".to_string()],
        AccessScope::default(), 1.0,
        None,
    ).await.unwrap();

    store.remember(
        "User clicked on settings",
        MemoryType::Episodic,
        "user1", "session1", "test", 1.0, None, vec!["action".to_string()],
        AccessScope::default(), 1.0,
        None,
    ).await.unwrap();

    let path = std::env::temp_dir().join("test_mneme_export.mneme");
    let path = path.to_str().unwrap();
    store.export(path).await.unwrap();

    let backend2 = SqliteBackend::new(":memory:").await.unwrap();
    let embedder2 = Arc::new(HashEmbedder::new(384));
    let store2 = MemoryStore::new("test-agent", Arc::new(backend2), embedder2).await;
    let count = store2.import_from(path).await.unwrap();
    assert_eq!(count, 2);

    let results = store2.recall("dark mode", RecallOptions::default(), None).await.unwrap();
    assert!(!results.is_empty());
    assert!(results[0].record.content.contains("dark mode"));

    std::fs::remove_file(path).ok();
}