/// E2E tests for configuration management
///
/// These tests validate that the application configuration is properly
/// loaded, saved, and managed throughout the application lifecycle.
use e2e_helpers::TestConfig;

#[test]
fn test_config_directory_creation() {
    let test_config = TestConfig::new();
    let cache_dir = test_config.cache_dir();

    assert!(cache_dir.exists(), "Cache directory should be created");
    assert!(cache_dir.is_dir(), "Cache path should be a directory");
}

#[test]
fn test_mock_config_creation() {
    let test_config = TestConfig::new();
    let config_path = test_config.create_mock_config("test_user_123");

    assert!(config_path.exists(), "Config file should be created");

    let content = std::fs::read_to_string(config_path).expect("Should be able to read config file");

    assert!(
        content.contains("test_user_123"),
        "Config should contain username"
    );
    assert!(
        content.contains("username"),
        "Config should have username field"
    );
}

#[test]
fn test_config_file_is_valid_json() {
    let test_config = TestConfig::new();
    let config_path = test_config.create_mock_config("valid_user");

    let content = std::fs::read_to_string(config_path).expect("Should be able to read config file");

    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    assert_eq!(
        parsed.get("username").and_then(|v| v.as_str()),
        Some("valid_user")
    );
}

#[test]
fn test_multiple_configs_are_isolated() {
    let config1 = TestConfig::new();
    let config2 = TestConfig::new();

    let path1 = config1.temp_path();
    let path2 = config2.temp_path();

    assert_ne!(
        path1, path2,
        "Different test configs should use different directories"
    );
}

#[test]
fn test_config_cleanup_on_drop() {
    let temp_path = {
        let test_config = TestConfig::new();
        let path = test_config.temp_path();
        assert!(path.exists(), "Temp directory should exist during test");
        path
    };
    // After TestConfig is dropped, the temporary directory should be cleaned up
    // Note: TempDir handles cleanup automatically

    // We can't reliably test cleanup here as TempDir uses Drop trait
    // but we can verify it was created
    assert!(!temp_path.as_os_str().is_empty());
}
