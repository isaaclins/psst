use std::path::PathBuf;
use tempfile::TempDir;

/// Test configuration helper for E2E tests
pub struct TestConfig {
    temp_dir: TempDir,
}

impl TestConfig {
    /// Create a new test configuration with a temporary directory
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        Self { temp_dir }
    }

    /// Get the path to the temporary directory
    pub fn temp_path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    /// Get a path for cache directory
    pub fn cache_dir(&self) -> PathBuf {
        let cache = self.temp_path().join("cache");
        std::fs::create_dir_all(&cache).expect("Failed to create cache directory");
        cache
    }

    /// Get a path for config file
    pub fn config_file(&self) -> PathBuf {
        self.temp_path().join("config.json")
    }

    /// Create a mock config file with test credentials
    pub fn create_mock_config(&self, username: &str) -> PathBuf {
        let config_path = self.config_file();
        let config_content = format!(
            r#"{{
                "username": "{}",
                "password": null,
                "credentials_location": null,
                "oauth_refresh_token": null,
                "oauth_access_token": null
            }}"#,
            username
        );
        std::fs::write(&config_path, config_content).expect("Failed to write config file");
        config_path
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creates_temp_dir() {
        let config = TestConfig::new();
        assert!(config.temp_path().exists());
    }

    #[test]
    fn test_config_creates_cache_dir() {
        let config = TestConfig::new();
        let cache_dir = config.cache_dir();
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());
    }

    #[test]
    fn test_create_mock_config() {
        let config = TestConfig::new();
        let config_path = config.create_mock_config("test_user");
        assert!(config_path.exists());

        let content = std::fs::read_to_string(config_path).unwrap();
        assert!(content.contains("test_user"));
    }
}
