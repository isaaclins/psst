/// E2E tests for mock Spotify API server
///
/// These tests validate the mock server functionality used in E2E tests
/// to simulate Spotify API responses.
use e2e_helpers::MockSpotifyServer;

#[test]
fn test_mock_server_initialization() {
    let server = MockSpotifyServer::new();
    assert_eq!(
        server.request_count(),
        0,
        "New server should have zero requests"
    );
}

#[test]
fn test_mock_server_register_endpoint() {
    let server = MockSpotifyServer::new();
    server.register_response("/api/auth", r#"{"token": "abc123"}"#);

    let response = server.get_response("/api/auth");
    assert!(response.is_some(), "Should return registered response");
    assert_eq!(response.unwrap(), r#"{"token": "abc123"}"#);
}

#[test]
fn test_mock_server_counts_requests() {
    let server = MockSpotifyServer::new();
    server.register_response("/test", "data");

    assert_eq!(server.request_count(), 0);

    server.get_response("/test");
    assert_eq!(server.request_count(), 1);

    server.get_response("/test");
    assert_eq!(server.request_count(), 2);

    server.get_response("/other"); // Non-existent endpoint still counts
    assert_eq!(server.request_count(), 3);
}

#[test]
fn test_mock_server_reset() {
    let server = MockSpotifyServer::new();
    server.register_response("/api/user", "user_data");
    server.get_response("/api/user");

    assert_eq!(server.request_count(), 1);

    server.reset();

    assert_eq!(server.request_count(), 0);
    assert!(server.get_response("/api/user").is_none());
}

#[test]
fn test_mock_auth_response_structure() {
    let auth_response = MockSpotifyServer::mock_auth_success();
    let parsed: serde_json::Value =
        serde_json::from_str(&auth_response).expect("Auth response should be valid JSON");

    assert!(parsed.get("access_token").is_some());
    assert!(parsed.get("token_type").is_some());
    assert!(parsed.get("expires_in").is_some());
    assert!(parsed.get("refresh_token").is_some());
    assert_eq!(
        parsed.get("token_type").and_then(|v| v.as_str()),
        Some("Bearer")
    );
}

#[test]
fn test_mock_user_profile_structure() {
    let profile_response = MockSpotifyServer::mock_user_profile();
    let parsed: serde_json::Value =
        serde_json::from_str(&profile_response).expect("User profile should be valid JSON");

    assert!(parsed.get("id").is_some());
    assert!(parsed.get("display_name").is_some());
    assert!(parsed.get("email").is_some());
    assert_eq!(
        parsed.get("product").and_then(|v| v.as_str()),
        Some("premium")
    );
}

#[test]
fn test_mock_track_structure() {
    let track_response = MockSpotifyServer::mock_track();
    let parsed: serde_json::Value =
        serde_json::from_str(&track_response).expect("Track should be valid JSON");

    assert!(parsed.get("id").is_some());
    assert!(parsed.get("name").is_some());
    assert!(parsed.get("duration_ms").is_some());
    assert!(parsed.get("artists").is_some());
    assert!(parsed.get("album").is_some());

    let artists = parsed.get("artists").and_then(|v| v.as_array());
    assert!(artists.is_some());
    assert!(!artists.unwrap().is_empty());
}

#[test]
fn test_mock_playlist_structure() {
    let playlist_response = MockSpotifyServer::mock_playlist();
    let parsed: serde_json::Value =
        serde_json::from_str(&playlist_response).expect("Playlist should be valid JSON");

    assert!(parsed.get("id").is_some());
    assert!(parsed.get("name").is_some());
    assert!(parsed.get("tracks").is_some());

    let tracks = parsed.get("tracks");
    assert!(tracks.is_some());
    assert!(tracks.unwrap().get("total").is_some());
}

#[test]
fn test_multiple_mock_servers_are_independent() {
    let server1 = MockSpotifyServer::new();
    let server2 = MockSpotifyServer::new();

    server1.register_response("/test", "data1");
    server2.register_response("/test", "data2");

    assert_eq!(server1.get_response("/test"), Some("data1".to_string()));
    assert_eq!(server2.get_response("/test"), Some("data2".to_string()));

    assert_eq!(server1.request_count(), 1);
    assert_eq!(server2.request_count(), 1);
}

#[test]
fn test_mock_server_handles_missing_endpoints() {
    let server = MockSpotifyServer::new();
    let response = server.get_response("/nonexistent");

    assert!(response.is_none());
    assert_eq!(server.request_count(), 1); // Request was still counted
}
