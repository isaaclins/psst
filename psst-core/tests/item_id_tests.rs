use psst_core::item_id::{FileId, ItemId, ItemIdType};
use std::path::PathBuf;

#[test]
fn item_id_invalid_constant_is_zero() {
    assert_eq!(ItemId::INVALID.id, 0);
    assert_eq!(ItemId::INVALID.id_type, ItemIdType::Unknown);
}

#[test]
fn item_id_default_equals_invalid() {
    let default_id = ItemId::default();
    assert_eq!(default_id, ItemId::INVALID);
}

#[test]
fn item_id_from_base16_with_valid_input() {
    let result = ItemId::from_base16("deadbeef", ItemIdType::Track);
    assert!(result.is_some());
    let id = result.unwrap();
    assert_eq!(id.id, 0xdeadbeef);
    assert_eq!(id.id_type, ItemIdType::Track);
}

#[test]
fn item_id_from_base16_with_invalid_char() {
    let result = ItemId::from_base16("xyz", ItemIdType::Track);
    assert!(result.is_none());
}

#[test]
fn item_id_from_base16_with_empty_string() {
    let result = ItemId::from_base16("", ItemIdType::Track);
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, 0);
}

#[test]
fn item_id_from_base62_with_valid_input() {
    // Test with a simple base62 string
    let result = ItemId::from_base62("abc", ItemIdType::Track);
    assert!(result.is_some());
    let id = result.unwrap();
    assert_eq!(id.id_type, ItemIdType::Track);
}

#[test]
fn item_id_from_base62_with_invalid_char() {
    // '@' is not a valid base62 character
    let result = ItemId::from_base62("@#$", ItemIdType::Track);
    assert!(result.is_none());
}

#[test]
fn item_id_from_base62_with_empty_string() {
    let result = ItemId::from_base62("", ItemIdType::Track);
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, 0);
}

#[test]
fn item_id_base62_roundtrip() {
    let original = ItemId::new(123456789, ItemIdType::Track);
    let base62_str = original.to_base62();
    let recovered = ItemId::from_base62(&base62_str, ItemIdType::Track).unwrap();
    assert_eq!(original.id, recovered.id);
}

#[test]
fn item_id_base16_roundtrip() {
    let original = ItemId::new(0xdeadbeefcafe, ItemIdType::Track);
    let base16_str = original.to_base16();
    let recovered = ItemId::from_base16(&base16_str, ItemIdType::Track).unwrap();
    assert_eq!(original.id, recovered.id);
}

#[test]
fn item_id_raw_roundtrip() {
    let original = ItemId::new(0x123456789abcdef, ItemIdType::Track);
    let raw_bytes = original.to_raw();
    let recovered = ItemId::from_raw(&raw_bytes, ItemIdType::Track).unwrap();
    assert_eq!(original.id, recovered.id);
}

#[test]
fn item_id_from_raw_with_invalid_length() {
    let short_bytes = &[0u8; 10];
    let result = ItemId::from_raw(short_bytes, ItemIdType::Track);
    assert!(result.is_none());
}

#[test]
fn item_id_from_uri_with_track() {
    let uri = "spotify:track:4cOdK2wGLETKBW3PvgPWqT";
    let result = ItemId::from_uri(uri);
    assert!(result.is_some());
    let id = result.unwrap();
    assert_eq!(id.id_type, ItemIdType::Track);
}

#[test]
fn item_id_from_uri_with_episode() {
    let uri = "spotify:episode:4cOdK2wGLETKBW3PvgPWqT";
    let result = ItemId::from_uri(uri);
    assert!(result.is_some());
    let id = result.unwrap();
    assert_eq!(id.id_type, ItemIdType::Podcast);
}

#[test]
fn item_id_from_uri_with_unknown_type() {
    let uri = "spotify:unknown:4cOdK2wGLETKBW3PvgPWqT";
    let result = ItemId::from_uri(uri);
    assert!(result.is_some());
    let id = result.unwrap();
    assert_eq!(id.id_type, ItemIdType::Unknown);
}

#[test]
fn item_id_from_uri_with_invalid_format() {
    let uri = "invalid_uri";
    let result = ItemId::from_uri(uri);
    // Should return None because split(':').next_back() returns Some("invalid_uri")
    // but from_base62 will fail on invalid characters
    assert!(result.is_none());
}

#[test]
fn item_id_from_uri_with_empty_string() {
    let uri = "";
    let result = ItemId::from_uri(uri);
    // Empty string returns Some("") from next_back, then from_base62 with empty string
    assert!(result.is_some());
}

#[test]
fn item_id_to_uri_for_track() {
    let id = ItemId::new(123456, ItemIdType::Track);
    let uri = id.to_uri();
    assert!(uri.is_some());
    assert!(uri.unwrap().starts_with("spotify:track:"));
}

#[test]
fn item_id_to_uri_for_podcast() {
    let id = ItemId::new(123456, ItemIdType::Podcast);
    let uri = id.to_uri();
    assert!(uri.is_some());
    assert!(uri.unwrap().starts_with("spotify:podcast:"));
}

#[test]
fn item_id_to_uri_for_local_file() {
    let id = ItemId::new(123456, ItemIdType::LocalFile);
    let uri = id.to_uri();
    assert!(uri.is_none());
}

#[test]
fn item_id_to_uri_for_unknown() {
    let id = ItemId::new(123456, ItemIdType::Unknown);
    let uri = id.to_uri();
    assert!(uri.is_none());
}

#[test]
fn item_id_to_base16_has_correct_length() {
    let id = ItemId::new(123456, ItemIdType::Track);
    let base16 = id.to_base16();
    assert_eq!(base16.len(), 32); // 128 bits = 32 hex chars
}

#[test]
fn item_id_to_base62_has_correct_length() {
    let id = ItemId::new(123456, ItemIdType::Track);
    let base62 = id.to_base62();
    assert_eq!(base62.len(), 22); // Fixed length base62 encoding
}

#[test]
fn item_id_local_file_roundtrip() {
    let path = PathBuf::from("/tmp/test_audio.mp3");
    let id = ItemId::from_local(path.clone());
    assert_eq!(id.id_type, ItemIdType::LocalFile);
    
    let recovered_path = id.to_local();
    assert_eq!(recovered_path, path);
}

#[test]
fn item_id_local_file_same_path_same_id() {
    // Use a unique path for this test to avoid interference from other tests
    let unique_path = PathBuf::from("/tmp/test_same_unique_xyz123.mp3");
    
    let id1 = ItemId::from_local(unique_path.clone());
    let id2 = ItemId::from_local(unique_path);
    
    assert_eq!(id1.id, id2.id);
}

#[test]
fn item_id_local_file_different_paths_different_ids() {
    let path1 = PathBuf::from("/tmp/test_different1.mp3");
    let path2 = PathBuf::from("/tmp/test_different2.mp3");
    
    let id1 = ItemId::from_local(path1);
    let id2 = ItemId::from_local(path2);
    
    assert_ne!(id1.id, id2.id);
}

#[test]
#[should_panic(expected = "expected local file")]
fn item_id_to_local_panics_on_non_local() {
    let id = ItemId::new(123456, ItemIdType::Track);
    let _path = id.to_local();
}

#[test]
fn file_id_from_raw_with_valid_length() {
    let data = [0u8; 20];
    let result = FileId::from_raw(&data);
    assert!(result.is_some());
}

#[test]
fn file_id_from_raw_with_invalid_length() {
    let data = [0u8; 15];
    let result = FileId::from_raw(&data);
    assert!(result.is_none());
}

#[test]
fn file_id_to_base16_has_correct_length() {
    let file_id = FileId([0u8; 20]);
    let base16 = file_id.to_base16();
    assert_eq!(base16.len(), 40); // 20 bytes * 2 hex chars per byte
}

#[test]
fn file_id_to_base16_format() {
    let mut data = [0u8; 20];
    data[0] = 0xDE;
    data[1] = 0xAD;
    let file_id = FileId(data);
    let base16 = file_id.to_base16();
    assert!(base16.starts_with("dead"));
}

#[test]
fn file_id_deref_returns_slice() {
    let data = [42u8; 20];
    let file_id = FileId(data);
    let slice: &[u8] = &file_id;
    assert_eq!(slice.len(), 20);
    assert_eq!(slice[0], 42);
}

#[test]
fn item_id_string_conversion() {
    let id = ItemId::new(123456, ItemIdType::Track);
    let string: String = id.into();
    assert_eq!(string.len(), 22); // base62 encoding
}

#[test]
fn item_id_zero_value() {
    let id = ItemId::new(0, ItemIdType::Track);
    let base62 = id.to_base62();
    assert_eq!(base62, "0000000000000000000000");
}

#[test]
fn item_id_max_value() {
    let id = ItemId::new(u128::MAX, ItemIdType::Track);
    let base16 = id.to_base16();
    assert_eq!(base16, "ffffffffffffffffffffffffffffffff");
}

#[test]
fn item_id_types_equality() {
    let id1 = ItemId::new(123, ItemIdType::Track);
    let id2 = ItemId::new(123, ItemIdType::Track);
    let id3 = ItemId::new(123, ItemIdType::Podcast);
    
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}
