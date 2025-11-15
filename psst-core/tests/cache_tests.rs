use psst_core::cache::Cache;
use psst_core::item_id::{ItemId, ItemIdType};
use psst_core::protocol::metadata::Track;
use std::fs;
use tempfile::TempDir;

fn create_test_cache() -> (TempDir, std::sync::Arc<Cache>) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cache = Cache::new(temp_dir.path().to_path_buf()).expect("failed to create cache");
    (temp_dir, cache)
}

#[test]
fn cache_new_creates_directory_structure() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();

    let _cache = Cache::new(cache_path.clone()).expect("failed to create cache");

    assert!(cache_path.join("track").exists());
    assert!(cache_path.join("episode").exists());
    assert!(cache_path.join("audio").exists());
    assert!(cache_path.join("key").exists());
}

#[test]
fn cache_new_with_nonexistent_path() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cache_path = temp_dir.path().join("nonexistent");

    let result = Cache::new(cache_path.clone());
    assert!(result.is_ok());
    assert!(cache_path.exists());
}

#[test]
fn cache_save_and_get_track() {
    let (_temp_dir, cache) = create_test_cache();

    let item_id = ItemId::new(123456, ItemIdType::Track);
    let track = Track {
        gid: None,
        name: Some("Test Track".to_string()),
        album: None,
        artist: vec![],
        number: Some(1),
        disc_number: Some(1),
        duration: Some(180000),
        popularity: Some(75),
        explicit: Some(false),
        external_id: vec![],
        restriction: vec![],
        file: vec![],
        alternative: vec![],
        sale_period: vec![],
        preview: vec![],
    };

    let save_result = cache.save_track(item_id, &track);
    assert!(save_result.is_ok());

    let retrieved = cache.get_track(item_id);
    assert!(retrieved.is_some());
    let retrieved_track = retrieved.unwrap();
    assert_eq!(retrieved_track.name, track.name);
    assert_eq!(retrieved_track.duration, track.duration);
}

#[test]
fn cache_get_nonexistent_track() {
    let (_temp_dir, cache) = create_test_cache();

    let item_id = ItemId::new(999999, ItemIdType::Track);
    let retrieved = cache.get_track(item_id);
    assert!(retrieved.is_none());
}

#[test]
fn cache_save_track_overwrites_existing() {
    let (_temp_dir, cache) = create_test_cache();

    let item_id = ItemId::new(123456, ItemIdType::Track);
    let track1 = Track {
        gid: None,
        name: Some("First Track".to_string()),
        album: None,
        artist: vec![],
        number: Some(1),
        disc_number: Some(1),
        duration: Some(180000),
        popularity: Some(75),
        explicit: Some(false),
        external_id: vec![],
        restriction: vec![],
        file: vec![],
        alternative: vec![],
        sale_period: vec![],
        preview: vec![],
    };

    let track2 = Track {
        name: Some("Second Track".to_string()),
        ..track1.clone()
    };

    cache
        .save_track(item_id, &track1)
        .expect("first save failed");
    cache
        .save_track(item_id, &track2)
        .expect("second save failed");

    let retrieved = cache.get_track(item_id).expect("retrieval failed");
    assert_eq!(retrieved.name, Some("Second Track".to_string()));
}

#[test]
fn cache_clear_removes_all_cached_items() {
    let (_temp_dir, cache) = create_test_cache();

    let item_id = ItemId::new(123456, ItemIdType::Track);
    let track = Track {
        gid: None,
        name: Some("Test Track".to_string()),
        album: None,
        artist: vec![],
        number: Some(1),
        disc_number: Some(1),
        duration: Some(180000),
        popularity: Some(75),
        explicit: Some(false),
        external_id: vec![],
        restriction: vec![],
        file: vec![],
        alternative: vec![],
        sale_period: vec![],
        preview: vec![],
    };

    cache.save_track(item_id, &track).expect("save failed");
    assert!(cache.get_track(item_id).is_some());

    cache.clear().expect("clear failed");

    assert!(cache.get_track(item_id).is_none());
}

#[test]
fn cache_clear_recreates_directory_structure() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    let cache = Cache::new(cache_path.clone()).expect("failed to create cache");

    cache.clear().expect("clear failed");

    assert!(cache_path.join("track").exists());
    assert!(cache_path.join("episode").exists());
    assert!(cache_path.join("audio").exists());
    assert!(cache_path.join("key").exists());
}

#[test]
fn cache_different_item_ids_dont_collide() {
    let (_temp_dir, cache) = create_test_cache();

    let item_id1 = ItemId::new(123, ItemIdType::Track);
    let item_id2 = ItemId::new(456, ItemIdType::Track);

    let track1 = Track {
        gid: None,
        name: Some("Track 1".to_string()),
        album: None,
        artist: vec![],
        number: Some(1),
        disc_number: Some(1),
        duration: Some(180000),
        popularity: Some(75),
        explicit: Some(false),
        external_id: vec![],
        restriction: vec![],
        file: vec![],
        alternative: vec![],
        sale_period: vec![],
        preview: vec![],
    };

    let track2 = Track {
        name: Some("Track 2".to_string()),
        ..track1.clone()
    };

    cache.save_track(item_id1, &track1).expect("save 1 failed");
    cache.save_track(item_id2, &track2).expect("save 2 failed");

    let retrieved1 = cache.get_track(item_id1).expect("retrieval 1 failed");
    let retrieved2 = cache.get_track(item_id2).expect("retrieval 2 failed");

    assert_eq!(retrieved1.name, Some("Track 1".to_string()));
    assert_eq!(retrieved2.name, Some("Track 2".to_string()));
}

#[test]
fn cache_get_track_with_corrupted_data() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    let cache = Cache::new(cache_path.clone()).expect("failed to create cache");

    let item_id = ItemId::new(123456, ItemIdType::Track);

    // Write invalid protobuf data
    let track_file_path = cache_path.join("track").join(item_id.to_base62());
    fs::write(track_file_path, b"invalid protobuf data").expect("write failed");

    let retrieved = cache.get_track(item_id);
    assert!(retrieved.is_none());
}

#[test]
fn cache_handles_empty_track_data() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    let cache = Cache::new(cache_path.clone()).expect("failed to create cache");

    let item_id = ItemId::new(123456, ItemIdType::Track);

    // Write empty file
    let track_file_path = cache_path.join("track").join(item_id.to_base62());
    fs::write(track_file_path, b"").expect("write failed");

    let retrieved = cache.get_track(item_id);
    // Empty protobuf might deserialize to default values or fail
    // Either way is acceptable, just testing it doesn't crash
    let _ = retrieved;
}
