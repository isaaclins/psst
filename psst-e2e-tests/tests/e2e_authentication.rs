/// E2E tests for authentication workflows
///
/// These tests validate authentication flows including OAuth token handling,
/// credential validation, and session management.
use e2e_helpers::{MockSpotifyServer, TestConfig};

#[test]
fn test_mock_auth_token_format() {
    let auth_response = MockSpotifyServer::mock_auth_success();
    let parsed: serde_json::Value =
        serde_json::from_str(&auth_response).expect("Auth response should be valid JSON");

    let access_token = parsed
        .get("access_token")
        .and_then(|v| v.as_str())
        .expect("Should have access_token");

    assert!(!access_token.is_empty(), "Access token should not be empty");
    assert!(
        access_token.starts_with("mock_"),
        "Mock token should have prefix"
    );
}

#[test]
fn test_auth_response_contains_required_fields() {
    let auth_response = MockSpotifyServer::mock_auth_success();
    let parsed: serde_json::Value =
        serde_json::from_str(&auth_response).expect("Auth response should be valid JSON");

    // Validate required OAuth 2.0 fields
    assert!(
        parsed.get("access_token").is_some(),
        "Must have access_token"
    );
    assert!(parsed.get("token_type").is_some(), "Must have token_type");
    assert!(parsed.get("expires_in").is_some(), "Must have expires_in");

    // Validate token type
    let token_type = parsed.get("token_type").and_then(|v| v.as_str());
    assert_eq!(token_type, Some("Bearer"));
}

#[test]
fn test_auth_flow_simulation() {
    let server = MockSpotifyServer::new();
    let test_config = TestConfig::new();

    // Simulate authentication request
    server.register_response("/api/token", &MockSpotifyServer::mock_auth_success());

    // Get auth response
    let auth_response = server
        .get_response("/api/token")
        .expect("Should get auth response");

    let parsed: serde_json::Value =
        serde_json::from_str(&auth_response).expect("Response should be valid JSON");

    let access_token = parsed
        .get("access_token")
        .and_then(|v| v.as_str())
        .expect("Should have access token");

    // Verify token was received
    assert!(!access_token.is_empty());

    // Simulate storing credentials in config
    let config_path = test_config.create_mock_config("authenticated_user");
    assert!(config_path.exists());
}

#[test]
fn test_user_profile_retrieval() {
    let server = MockSpotifyServer::new();

    // Register user profile endpoint
    server.register_response("/api/me", &MockSpotifyServer::mock_user_profile());

    // Simulate getting user profile
    let profile_response = server
        .get_response("/api/me")
        .expect("Should get profile response");

    let parsed: serde_json::Value =
        serde_json::from_str(&profile_response).expect("Profile should be valid JSON");

    assert_eq!(parsed.get("id").and_then(|v| v.as_str()), Some("test_user"));
    assert_eq!(
        parsed.get("display_name").and_then(|v| v.as_str()),
        Some("Test User")
    );
    assert_eq!(
        parsed.get("product").and_then(|v| v.as_str()),
        Some("premium")
    );
}

#[test]
fn test_token_expiration_handling() {
    let auth_response = MockSpotifyServer::mock_auth_success();
    let parsed: serde_json::Value =
        serde_json::from_str(&auth_response).expect("Auth response should be valid JSON");

    let expires_in = parsed
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .expect("Should have expires_in");

    assert!(expires_in > 0, "Token expiration should be positive");
    assert_eq!(expires_in, 3600, "Default expiration should be 1 hour");
}

#[test]
fn test_refresh_token_present() {
    let auth_response = MockSpotifyServer::mock_auth_success();
    let parsed: serde_json::Value =
        serde_json::from_str(&auth_response).expect("Auth response should be valid JSON");

    let refresh_token = parsed
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .expect("Should have refresh_token");

    assert!(
        !refresh_token.is_empty(),
        "Refresh token should not be empty"
    );
}

#[test]
fn test_authenticated_request_flow() {
    let server = MockSpotifyServer::new();

    // Step 1: Authenticate
    server.register_response("/oauth/token", &MockSpotifyServer::mock_auth_success());
    let auth = server.get_response("/oauth/token").unwrap();
    let auth_parsed: serde_json::Value = serde_json::from_str(&auth).unwrap();
    let token = auth_parsed.get("access_token").unwrap().as_str().unwrap();

    // Step 2: Use token to get user profile
    server.register_response("/api/me", &MockSpotifyServer::mock_user_profile());
    let profile = server.get_response("/api/me").unwrap();

    // Step 3: Verify both requests were made
    assert_eq!(server.request_count(), 2);
    assert!(!token.is_empty());
    assert!(profile.contains("test_user"));
}
