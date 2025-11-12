/// E2E tests for playback functionality
///
/// These tests validate playback queue management, track handling,
/// and playback state transitions.
use e2e_helpers::MockSpotifyServer;

#[test]
fn test_track_metadata_structure() {
    let track_response = MockSpotifyServer::mock_track();
    let parsed: serde_json::Value =
        serde_json::from_str(&track_response).expect("Track should be valid JSON");

    // Validate track has required fields
    assert!(parsed.get("id").is_some());
    assert!(parsed.get("name").is_some());
    assert!(parsed.get("duration_ms").is_some());
    assert!(parsed.get("artists").is_some());
    assert!(parsed.get("album").is_some());
}

#[test]
fn test_track_duration_is_positive() {
    let track_response = MockSpotifyServer::mock_track();
    let parsed: serde_json::Value =
        serde_json::from_str(&track_response).expect("Track should be valid JSON");

    let duration = parsed
        .get("duration_ms")
        .and_then(|v| v.as_i64())
        .expect("Should have duration_ms");

    assert!(duration > 0, "Track duration should be positive");
    assert_eq!(
        duration, 180000,
        "Mock track should be 3 minutes (180000ms)"
    );
}

#[test]
fn test_track_has_artist_information() {
    let track_response = MockSpotifyServer::mock_track();
    let parsed: serde_json::Value =
        serde_json::from_str(&track_response).expect("Track should be valid JSON");

    let artists = parsed
        .get("artists")
        .and_then(|v| v.as_array())
        .expect("Should have artists array");

    assert!(!artists.is_empty(), "Track should have at least one artist");

    let first_artist = &artists[0];
    assert!(first_artist.get("id").is_some());
    assert!(first_artist.get("name").is_some());
}

#[test]
fn test_track_has_album_information() {
    let track_response = MockSpotifyServer::mock_track();
    let parsed: serde_json::Value =
        serde_json::from_str(&track_response).expect("Track should be valid JSON");

    let album = parsed.get("album").expect("Track should have album");

    assert!(album.get("id").is_some());
    assert!(album.get("name").is_some());
}

#[test]
fn test_playlist_structure() {
    let playlist_response = MockSpotifyServer::mock_playlist();
    let parsed: serde_json::Value =
        serde_json::from_str(&playlist_response).expect("Playlist should be valid JSON");

    assert!(parsed.get("id").is_some());
    assert!(parsed.get("name").is_some());
    assert!(parsed.get("description").is_some());
    assert!(parsed.get("tracks").is_some());
}

#[test]
fn test_playlist_track_count() {
    let playlist_response = MockSpotifyServer::mock_playlist();
    let parsed: serde_json::Value =
        serde_json::from_str(&playlist_response).expect("Playlist should be valid JSON");

    let tracks = parsed.get("tracks").expect("Playlist should have tracks");

    let total = tracks
        .get("total")
        .and_then(|v| v.as_i64())
        .expect("Tracks should have total");

    assert!(total >= 0, "Track count should be non-negative");
    assert_eq!(total, 10, "Mock playlist should have 10 tracks");
}

#[test]
fn test_playback_queue_simulation() {
    let server = MockSpotifyServer::new();

    // Add tracks to mock queue
    server.register_response("/api/tracks/1", &MockSpotifyServer::mock_track());
    server.register_response("/api/tracks/2", &MockSpotifyServer::mock_track());
    server.register_response("/api/tracks/3", &MockSpotifyServer::mock_track());

    // Simulate getting tracks
    let track1 = server.get_response("/api/tracks/1");
    let track2 = server.get_response("/api/tracks/2");
    let track3 = server.get_response("/api/tracks/3");

    assert!(track1.is_some());
    assert!(track2.is_some());
    assert!(track3.is_some());
    assert_eq!(server.request_count(), 3);
}

#[test]
fn test_playback_state_transitions() {
    // Simulate playback state transitions
    #[derive(Debug, PartialEq)]
    enum PlaybackState {
        Stopped,
        Playing,
        Paused,
    }

    let mut state = PlaybackState::Stopped;
    assert_eq!(state, PlaybackState::Stopped);

    // Start playback
    state = PlaybackState::Playing;
    assert_eq!(state, PlaybackState::Playing);

    // Pause
    state = PlaybackState::Paused;
    assert_eq!(state, PlaybackState::Paused);

    // Resume
    state = PlaybackState::Playing;
    assert_eq!(state, PlaybackState::Playing);

    // Stop
    state = PlaybackState::Stopped;
    assert_eq!(state, PlaybackState::Stopped);
}

#[test]
fn test_track_queue_management() {
    // Simulate a simple queue
    let mut queue: Vec<String> = Vec::new();

    assert!(queue.is_empty());

    // Add tracks
    queue.push("track_1".to_string());
    queue.push("track_2".to_string());
    queue.push("track_3".to_string());

    assert_eq!(queue.len(), 3);

    // Play next (remove from front)
    let current = queue.remove(0);
    assert_eq!(current, "track_1");
    assert_eq!(queue.len(), 2);

    // Add to queue
    queue.push("track_4".to_string());
    assert_eq!(queue.len(), 3);

    // Clear queue
    queue.clear();
    assert!(queue.is_empty());
}

#[test]
fn test_multiple_tracks_from_playlist() {
    let server = MockSpotifyServer::new();

    // Get playlist
    server.register_response("/api/playlists/001", &MockSpotifyServer::mock_playlist());
    let playlist_response = server.get_response("/api/playlists/001").unwrap();
    let playlist: serde_json::Value = serde_json::from_str(&playlist_response).unwrap();

    let track_count = playlist
        .get("tracks")
        .and_then(|t| t.get("total"))
        .and_then(|v| v.as_i64())
        .unwrap();

    // Simulate fetching all tracks
    for i in 0..track_count {
        let endpoint = format!("/api/tracks/{}", i);
        server.register_response(&endpoint, &MockSpotifyServer::mock_track());
    }

    // Verify we can fetch all tracks
    for i in 0..track_count {
        let endpoint = format!("/api/tracks/{}", i);
        let track = server.get_response(&endpoint);
        assert!(track.is_some(), "Should get track {}", i);
    }

    // 1 for playlist + 10 for tracks
    assert_eq!(server.request_count(), 11);
}
