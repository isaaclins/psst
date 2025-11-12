# Copilot Instructions for Psst

Psst is a fast, native Spotify client written in Rust, without Electron. This repository contains a Cargo workspace with multiple crates that implement different aspects of the application.

## Project Structure

- `/psst-core` - Core library handling Spotify TCP session, audio retrieval, decoding, output, and playback queue
- `/psst-gui` - GUI application built with [Druid](https://github.com/linebender/druid)
- `/psst-cli` - Example CLI application for testing and development
- `/psst-protocol` - Internal Protobuf definitions for Spotify communication

## Technology Stack

- **Language**: Rust (stable, minimum version 1.65.0)
- **GUI Framework**: Druid (with GTK and X11 backends on Linux)
- **Build System**: Cargo
- **Audio**: Custom implementation using symphonia and libsamplerate

## Development Guidelines

### Code Style

- Follow Rust standard formatting using `rustfmt`
- Configuration in `.rustfmt.toml`:
  - Use crate-level import granularity
  - Wrap comments at line boundaries
- Run `cargo clippy -- -D warnings` to check for linting issues
- All clippy warnings should be addressed before committing

### Building and Testing

1. **Build the project**:
   ```bash
   cargo build
   # For release builds:
   cargo build --release
   ```

2. **Run tests**:
   ```bash
   cargo test --workspace --all-targets
   ```

3. **Check formatting and linting**:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   ```

4. **Run the GUI application**:
   ```bash
   cargo run --bin psst-gui
   ```

### Platform-Specific Considerations

- **Linux**: Requires GTK-3, OpenSSL, and ALSA development libraries
  - Debian/Ubuntu: `libssl-dev libgtk-3-dev libcairo2-dev libasound2-dev`
  - RHEL/Fedora: `openssl-devel gtk3-devel cairo-devel alsa-lib-devel`
- **macOS**: Standard development tools required
- **Windows**: Standard development tools required

### Code Organization

- Keep synchronous architecture (no tokio or async runtime currently)
- Use HTTPS-based CDN for audio file retrieval
- Separate concerns between protocol, core logic, and UI layers
- Follow existing patterns in each crate

### Testing

- Write unit tests in the crate's `tests` directory
- Integration tests should cover cross-crate functionality
- Test both success and error paths
- Mock external Spotify API calls when possible

## Pull Request Guidelines

When creating or updating pull requests:

1. Ensure all tests pass: `cargo test --workspace --all-targets`
2. Verify code style: `cargo fmt --check && cargo clippy -- -D warnings`
3. Keep changes focused and minimal
4. Update relevant documentation in README.md or code comments
5. Reference related issues using "Fixes #issue" or "Towards #issue"
6. Label pull requests appropriately
7. For LLM-generated PRs, always indicate this in the description

## Privacy and Security

- **Never commit Spotify credentials** or authentication tokens
- Use reusable authentication tokens from Spotify (not stored user credentials)
- Only connect to official Spotify servers
- Keep local caches user-deletable
- Follow Spotify's API terms of service

## Common Tasks

### Adding a New Feature

1. Identify which crate(s) need changes
2. Add necessary dependencies to `Cargo.toml`
3. Implement feature following existing patterns
4. Add tests for new functionality
5. Update documentation
6. Test on all target platforms if possible

### Fixing Bugs

1. Reproduce the issue
2. Add a failing test that demonstrates the bug
3. Fix the issue
4. Verify the test now passes
5. Check for similar issues elsewhere

### Performance Optimization

- Profile before optimizing
- Use `cargo build --release` for benchmarking
- Consider impact on audio playback quality
- Test on resource-constrained systems

## Important Notes

- This project does not support Spotify Connect (remote control) yet
- Audio playback should be glitch-free and responsive
- UI should remain responsive during network operations
- Cache management should be memory-efficient
- A Spotify Premium account is required for testing

## Resources

- Project README: `/README.md`
- Rust documentation: Standard library docs
- Druid documentation: https://github.com/linebender/druid
- Spotify protocol details: See psst-protocol crate
