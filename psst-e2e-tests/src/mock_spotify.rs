use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock Spotify API server for testing
pub struct MockSpotifyServer {
    responses: Arc<Mutex<HashMap<String, String>>>,
    request_count: Arc<Mutex<usize>>,
}

impl MockSpotifyServer {
    /// Create a new mock server
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            request_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Register a mock response for a given endpoint
    pub fn register_response(&self, endpoint: &str, response: &str) {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(endpoint.to_string(), response.to_string());
    }

    /// Get a mock response for an endpoint
    pub fn get_response(&self, endpoint: &str) -> Option<String> {
        let responses = self.responses.lock().unwrap();
        let mut count = self.request_count.lock().unwrap();
        *count += 1;
        responses.get(endpoint).cloned()
    }

    /// Get the number of requests made
    pub fn request_count(&self) -> usize {
        *self.request_count.lock().unwrap()
    }

    /// Reset the mock server state
    pub fn reset(&self) {
        let mut responses = self.responses.lock().unwrap();
        responses.clear();
        let mut count = self.request_count.lock().unwrap();
        *count = 0;
    }

    /// Create a mock authentication response
    pub fn mock_auth_success() -> String {
        r#"{
            "access_token": "mock_access_token_12345",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "mock_refresh_token_67890",
            "scope": "user-read-private user-read-email"
        }"#
        .to_string()
    }

    /// Create a mock user profile response
    pub fn mock_user_profile() -> String {
        r#"{
            "id": "test_user",
            "display_name": "Test User",
            "email": "test@example.com",
            "country": "US",
            "product": "premium"
        }"#
        .to_string()
    }

    /// Create a mock track response
    pub fn mock_track() -> String {
        r#"{
            "id": "track_123",
            "name": "Test Track",
            "duration_ms": 180000,
            "artists": [
                {
                    "id": "artist_456",
                    "name": "Test Artist"
                }
            ],
            "album": {
                "id": "album_789",
                "name": "Test Album"
            }
        }"#
        .to_string()
    }

    /// Create a mock playlist response
    pub fn mock_playlist() -> String {
        r#"{
            "id": "playlist_001",
            "name": "Test Playlist",
            "description": "A test playlist",
            "tracks": {
                "total": 10
            }
        }"#
        .to_string()
    }
}

impl Default for MockSpotifyServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_creation() {
        let server = MockSpotifyServer::new();
        assert_eq!(server.request_count(), 0);
    }

    #[test]
    fn test_register_and_get_response() {
        let server = MockSpotifyServer::new();
        server.register_response("/api/token", "test_response");

        let response = server.get_response("/api/token");
        assert_eq!(response, Some("test_response".to_string()));
        assert_eq!(server.request_count(), 1);
    }

    #[test]
    fn test_reset_clears_state() {
        let server = MockSpotifyServer::new();
        server.register_response("/test", "data");
        server.get_response("/test");

        server.reset();

        assert_eq!(server.request_count(), 0);
        assert_eq!(server.get_response("/test"), None);
    }

    #[test]
    fn test_mock_responses_are_valid_json() {
        // Verify that mock responses are valid JSON
        let _ = serde_json::from_str::<serde_json::Value>(&MockSpotifyServer::mock_auth_success())
            .expect("Auth response should be valid JSON");
        let _ = serde_json::from_str::<serde_json::Value>(&MockSpotifyServer::mock_user_profile())
            .expect("User profile should be valid JSON");
        let _ = serde_json::from_str::<serde_json::Value>(&MockSpotifyServer::mock_track())
            .expect("Track should be valid JSON");
        let _ = serde_json::from_str::<serde_json::Value>(&MockSpotifyServer::mock_playlist())
            .expect("Playlist should be valid JSON");
    }
}
