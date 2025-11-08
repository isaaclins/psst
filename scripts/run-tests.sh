#!/usr/bin/env bash
set -euo pipefail

log() {
    printf '\n[%s] %s\n' "$(date +%H:%M:%S)" "$*"
}

log "Running clippy for psst-core"
cargo clippy -p psst-core --all-targets --all-features -- -D warnings

log "Running workspace tests"
cargo test --workspace --all-targets --all-features

log "Running documentation tests"
cargo test --workspace --doc

log "All checks passed"
