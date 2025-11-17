# Copilot Instructions

This is a multi-crate Rust workspace that implements the psst Spotify client, including a GUI (`psst-gui`), CLI (`psst-cli`), shared core library (`psst-core`), and protocol bindings (`psst-protocol`). It supports macOS, Windows, and Linux builds via `cargo` and Cross.

## Code Standards

### Required Before Each Commit

- Run `cargo fmt --all` to keep Rust sources formatted consistently across crates.
- Run `cargo clippy --all-targets --all-features -- -D warnings` to prevent lints from slipping into main.
- Ensure `./scripts/run-tests.sh` passes; it drives the standard unit/integration suite across the workspace.

### Development Flow

- Build (workspace): `cargo build --workspace`
- Build (GUI app with release profile): `cargo build -p psst-gui --release`
- Test (workspace): `cargo test --workspace`
- Full local verification: `./scripts/run-tests.sh`
- Run GUI app (dev mode): `cargo run -p psst-gui`
- Cross-platform release check (optional): `cross build --workspace --release`

## Repository Structure

- `psst-core/`: Core playback, session, caching, and Spotify protocol logic shared by front ends.
- `psst-gui/`: GUI application built with Druid; handles controller/data/ui layers and platform integrations.
- `psst-cli/`: Minimal terminal client demonstrating core playback and session routines.
- `psst-protocol/`: Generated protocol bindings and protobuf definitions; run `./psst-protocol/build.sh` or `cargo build -p psst-protocol` after editing `proto/`.
- `scripts/`: Helper scripts such as `run-tests.sh` for unified local CI.
- `target/`: Cargo build artifacts (ignored by git).

## Key Guidelines

1. Follow idiomatic Rust patterns: prefer `Result` error handling, avoid `unwrap` in production paths, and document `unsafe` usage clearly if unavoidable.
2. Keep shared abstractions inside `psst-core`; front ends should depend on those APIs rather than duplicating logic.
3. When touching the GUI, ensure state flows through the controller/data layers to keep UI widgets declarative.
4. Add tests for new behaviour. Use integration tests for player/session flows and unit tests for isolated components. Wire new suites into `./scripts/run-tests.sh`.
5. Adopt the branching strategy below before committing changes.

## Branching Strategy

- Start new feature work from `dev` using a short-lived feature branch (`feature/<slug>`). Keep the branch focused and rebased as needed.
- Build and test the feature branch locally; once it passes, merge it back into `dev` and delete the feature branch.
- Periodically (after a batch of features or when a release feels ready), merge `dev` into `main` following the same test/verification checklist.
