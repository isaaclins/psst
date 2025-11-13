# End-to-End Testing in Psst

## Overview

Psst uses a practical approach to end-to-end (E2E) testing that is tailored for native Rust GUI applications. Unlike web applications where tools like Cypress can simulate user interactions in a browser, native desktop applications require different testing strategies.

## Why Not Cypress?

Cypress and similar tools are designed for web applications running in browsers. Psst is a native desktop application built with:
- **Rust** programming language
- **Druid** GUI framework
- Native OS widgets and APIs

These technologies are fundamentally different from web technologies (HTML/CSS/JavaScript), making traditional web E2E tools incompatible.

## Our Testing Approach

Instead of attempting to adapt web-testing tools, we've implemented a testing strategy optimized for Rust desktop applications:

### 1. Integration Testing
We test core application logic and workflows without requiring a running GUI. This includes:
- Configuration management
- Authentication flows
- Playback state management
- Queue operations

### 2. Mock Services
External dependencies (like the Spotify API) are mocked to ensure:
- Tests are deterministic and reproducible
- Tests can run offline
- Tests execute quickly
- No external service rate limits or failures affect tests

### 3. Helper Utilities
A dedicated test package (`psst-e2e-tests`) provides reusable utilities:
- **TestConfig**: Manages temporary test directories and configuration files
- **MockSpotifyServer**: Simulates Spotify API responses with realistic data structures

## Test Organization

```
psst/
├── psst-e2e-tests/              # E2E test package
│   ├── src/
│   │   ├── lib.rs              # Helper library
│   │   ├── mock_spotify.rs     # Mock Spotify API
│   │   └── test_config.rs      # Test configuration
│   ├── tests/
│   │   ├── e2e_authentication.rs   # Auth tests
│   │   ├── e2e_config.rs          # Config tests
│   │   ├── e2e_mock_spotify.rs    # Mock server tests
│   │   └── e2e_playback.rs        # Playback tests
│   └── README.md               # Detailed test documentation
└── .github/
    └── workflows/
        └── build.yml           # CI includes E2E tests
```

## Running Tests

### Run All E2E Tests
```bash
cargo test -p psst-e2e-tests
```

### Run Specific Test Category
```bash
cargo test -p psst-e2e-tests --test e2e_authentication
cargo test -p psst-e2e-tests --test e2e_playback
cargo test -p psst-e2e-tests --test e2e_config
```

### Run All Tests Including E2E
```bash
cargo test --workspace --all-targets
```

### Run with Detailed Output
```bash
cargo test -p psst-e2e-tests -- --nocapture
```

## CI Integration

E2E tests run automatically in GitHub Actions as part of the test suite:

```yaml
- name: Run E2E Tests
  run: cargo test -p psst-e2e-tests
```

Tests are executed on every pull request and push to main/dev branches.

## Test Coverage

Current E2E test coverage includes:

### ✅ Authentication
- OAuth token format validation
- Authentication response structure
- Token expiration handling
- User profile retrieval
- Multi-step auth flows

### ✅ Configuration
- Config directory creation
- Config file generation and validation
- Multiple config isolation
- JSON structure validation

### ✅ Mock API Server
- Response registration and retrieval
- Request counting
- Server state management
- Realistic mock data structures

### ✅ Playback
- Track metadata validation
- Playlist structure
- Queue management simulation
- Playback state transitions
- Multi-track workflows

## Limitations

### What We DON'T Test
- **Visual Appearance**: No pixel-perfect UI validation
- **User Input**: No mouse click or keyboard simulation
- **Rendering**: No screenshot comparison
- **Cross-Platform UI**: No platform-specific UI validation
- **Performance**: No UI rendering performance tests

### Why These Limitations?
These limitations are intentional trade-offs:
1. **Maintainability**: Visual tests are brittle and expensive to maintain
2. **Speed**: Logic tests run in milliseconds vs. seconds for UI tests
3. **Reliability**: UI automation is flaky; logic tests are deterministic
4. **Focus**: We test what matters most - application behavior and correctness

## Future Enhancements

Potential improvements to the E2E testing framework:

### Short Term
- [ ] Add more test scenarios for edge cases
- [ ] Test error handling and recovery
- [ ] Add network simulation (timeouts, errors)
- [ ] Test concurrent operations

### Medium Term
- [ ] Integration with actual Spotify API (isolated test account)
- [ ] Performance benchmarks for key workflows
- [ ] Load testing for cache and queue operations
- [ ] Memory leak detection in long-running scenarios

### Long Term
- [ ] GUI automation using AccessKit (accessibility APIs)
- [ ] Screenshot comparison for critical UI states
- [ ] Platform-specific integration tests
- [ ] Automated exploratory testing

## Writing New Tests

When adding new E2E tests:

1. **Determine the Category**: Does it fit authentication, config, playback, or a new category?
2. **Use Helpers**: Leverage `TestConfig` and `MockSpotifyServer`
3. **Keep Tests Isolated**: Each test should be independent
4. **Mock External Dependencies**: Don't rely on real services
5. **Test Business Logic**: Focus on workflows and state management
6. **Document Test Intent**: Add clear comments about what's being tested

### Example Test

```rust
use e2e_helpers::{MockSpotifyServer, TestConfig};

#[test]
fn test_user_playlist_loading() {
    // Setup
    let server = MockSpotifyServer::new();
    let config = TestConfig::new();
    
    // Register mock responses
    server.register_response(
        "/api/playlists/mine",
        &MockSpotifyServer::mock_playlist()
    );
    
    // Execute
    let response = server.get_response("/api/playlists/mine")
        .expect("Should get playlist");
    
    // Verify
    let playlist: serde_json::Value = 
        serde_json::from_str(&response).unwrap();
    assert!(playlist.get("tracks").is_some());
    assert_eq!(server.request_count(), 1);
}
```

## Best Practices

1. **Test Behavior, Not Implementation**: Focus on what the code does, not how
2. **Use Descriptive Names**: Test names should explain what they validate
3. **One Assertion Per Test**: Keep tests focused and easy to debug
4. **Setup and Teardown**: Use `TestConfig` for automatic cleanup
5. **Mock Realistically**: Mock data should match real API responses
6. **Test Happy and Sad Paths**: Include error scenarios
7. **Keep Tests Fast**: Tests should run in milliseconds

## Contributing

We welcome contributions to the E2E test suite! Please:

1. Follow the existing test structure and patterns
2. Add tests for new features or bug fixes
3. Ensure all tests pass before submitting PR
4. Update documentation for new test categories
5. Use the helper utilities provided

## Resources

- [E2E Test Package README](../psst-e2e-tests/README.md) - Detailed test documentation
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html) - Official Rust testing docs
- [Integration Testing](https://doc.rust-lang.org/book/ch11-03-test-organization.html) - Rust integration tests

## Questions?

If you have questions about E2E testing in Psst:
- Review the test examples in `psst-e2e-tests/tests/`
- Check the helper documentation in `psst-e2e-tests/README.md`
- Open an issue on GitHub for clarification
