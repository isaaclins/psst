#!/usr/bin/env bash
set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    printf '\n[%s] %s\n' "$(date +%H:%M:%S)" "$*"
}

error() {
    printf "${RED}[ERROR]${NC} %s\n" "$*" >&2
}

success() {
    printf "${GREEN}[SUCCESS]${NC} %s\n" "$*"
}

warning() {
    printf "${YELLOW}[WARNING]${NC} %s\n" "$*"
}

# Track overall status
OVERALL_STATUS=0

log "Starting comprehensive test suite"

# Run clippy for all packages with strict warnings
log "Running clippy for all workspace packages"
if cargo clippy --workspace --all-targets -- -D warnings; then
    success "Clippy checks passed"
else
    error "Clippy checks failed"
    OVERALL_STATUS=1
fi

# Run all workspace tests
log "Running workspace tests"
if cargo test --workspace --all-targets --verbose; then
    success "Workspace tests passed"
else
    error "Workspace tests failed"
    OVERALL_STATUS=1
fi

# Run documentation tests
log "Running documentation tests"
if cargo test --workspace --doc; then
    success "Documentation tests passed"
else
    error "Documentation tests failed"
    OVERALL_STATUS=1
fi

# Count and report test statistics
log "Gathering test statistics"
TEST_COUNT=$(cargo test --workspace --all-targets 2>&1 | grep "test result:" | awk '{sum+=$4} END {print sum}')
log "Total tests executed: ${TEST_COUNT}"

# Check for minimum test coverage (ensure we have tests)
if [ "${TEST_COUNT}" -lt 10 ]; then
    warning "Low test count detected: ${TEST_COUNT} tests. Consider adding more tests."
fi

# Final status
if [ $OVERALL_STATUS -eq 0 ]; then
    success "All checks passed successfully!"
    log "Test suite completed successfully"
    exit 0
else
    error "Some checks failed. Please review the output above."
    exit 1
fi
