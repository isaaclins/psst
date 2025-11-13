# End-to-End Tests for Psst

This package contains end-to-end (E2E) tests for the Psst application.

## Overview

Since Psst is a native Rust GUI application built with Druid, traditional web-based E2E testing tools like Cypress are not directly applicable. Instead, this test suite uses a practical approach that focuses on:

1. **Integration Tests**: Test core functionality and business logic without GUI dependencies
2. **Mock Services**: Simulate Spotify API responses for reproducible tests
3. **Workflow Testing**: Validate application state transitions and user workflows

## Test Structure

```
psst-e2e-tests/
├── README.md                  # This file
├── Cargo.toml                # Package manifest
├── src/
│   ├── lib.rs                # Helper library exports
│   ├── mock_spotify.rs       # Mock Spotify API server
│   └── test_config.rs        # Test configuration utilities
└── tests/
    ├── e2e_authentication.rs # Authentication flow tests
    ├── e2e_config.rs         # Configuration management tests
    ├── e2e_mock_spotify.rs   # Mock server tests
    └── e2e_playback.rs       # Playback functionality tests
```

## Running Tests

Run all E2E tests:
```bash
cargo test -p psst-e2e-tests
```

Run tests with detailed output:
```bash
cargo test -p psst-e2e-tests -- --nocapture
```

Run a specific test file:
```bash
cargo test -p psst-e2e-tests --test e2e_authentication
```

Run a specific test:
```bash
cargo test -p psst-e2e-tests test_auth_flow_simulation
```

## Test Categories

### Configuration Tests (`e2e_config.rs`)
Tests for configuration management including:
- Config directory creation
- Mock config file generation
- Config validation
- Multiple config isolation

### Mock Spotify Tests (`e2e_mock_spotify.rs`)
Tests for the mock Spotify API server:
- Mock server initialization
- Response registration
- Request counting
- Mock data structure validation

### Authentication Tests (`e2e_authentication.rs`)
Tests for authentication workflows:
- OAuth token handling
- Authentication flow simulation
- User profile retrieval
- Token expiration handling

### Playback Tests (`e2e_playback.rs`)
Tests for playback functionality:
- Track metadata structure
- Playlist management
- Playback queue simulation
- State transitions

## Writing New Tests

When adding new E2E tests:

1. Create a new test file in `tests/` with the prefix `e2e_`
2. Import helpers: `use e2e_helpers::{MockSpotifyServer, TestConfig};`
3. Use the helper utilities for common operations:
   - `TestConfig::new()` for test configuration
   - `MockSpotifyServer::new()` for API mocking
4. Mock external dependencies (Spotify API) for reliability
5. Test user workflows and state transitions
6. Ensure tests are deterministic and can run in CI

Example:
```rust
use e2e_helpers::{MockSpotifyServer, TestConfig};

#[test]
fn test_my_feature() {
    let server = MockSpotifyServer::new();
    let config = TestConfig::new();
    
    // Register mock responses
    server.register_response("/api/endpoint", r#"{"data": "value"}"#);
    
    // Test your feature
    let response = server.get_response("/api/endpoint");
    assert!(response.is_some());
}
```

## Helper Utilities

### TestConfig

Provides test configuration and temporary directories:
- `TestConfig::new()` - Create new test config with temp directory
- `.temp_path()` - Get temporary directory path
- `.cache_dir()` - Get cache directory path
- `.config_file()` - Get config file path
- `.create_mock_config(username)` - Create a mock config file

### MockSpotifyServer

Mock Spotify API server for testing:
- `MockSpotifyServer::new()` - Create new mock server
- `.register_response(endpoint, response)` - Register mock response
- `.get_response(endpoint)` - Get response for endpoint
- `.request_count()` - Get number of requests made
- `.reset()` - Reset server state

Static mock data generators:
- `MockSpotifyServer::mock_auth_success()` - Authentication response
- `MockSpotifyServer::mock_user_profile()` - User profile data
- `MockSpotifyServer::mock_track()` - Track metadata
- `MockSpotifyServer::mock_playlist()` - Playlist data

## CI Integration

E2E tests are part of the workspace and run automatically in CI:

```bash
# Run all workspace tests (includes E2E tests)
cargo test --workspace --all-targets

# Run only E2E tests
cargo test -p psst-e2e-tests
```

## Limitations

- **No Visual Testing**: These tests don't validate the visual appearance of the UI
- **No User Interaction Simulation**: Tests don't simulate actual mouse clicks or keyboard input
- **Focus on Logic**: Tests focus on application logic, state management, and workflows
- **Mock Data**: Uses mock Spotify API responses, not real API calls

## Future Improvements

- Add GUI automation using accessibility APIs (AccessKit)
- Implement screenshot comparison testing
- Add performance benchmarks for key workflows
- Create load testing scenarios
- Add network condition simulation (latency, timeouts)
- Test with real Spotify API in isolated environment
- Add UI state validation tests
- Implement integration with CI/CD for automated testing

## Contributing

When contributing E2E tests:
1. Follow the existing test structure and naming conventions
2. Use the provided helper utilities
3. Ensure tests are isolated and deterministic
4. Add documentation for new test scenarios
5. Run all tests before submitting: `cargo test -p psst-e2e-tests`
