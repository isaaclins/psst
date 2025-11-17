# Testing Guide

This document describes the testing strategy and practices for the Psst project.

## Overview

The test suite is designed to ensure code quality and prevent "happy path" oriented development by covering:
- **Unit tests**: Testing individual modules and functions
- **Integration tests**: Testing interactions between components  
- **Edge cases**: Boundary conditions, empty inputs, invalid data
- **Error handling**: Ensuring proper error propagation and recovery

## Running Tests

### Quick Start

Run the full test suite:
```bash
./scripts/run-tests.sh
```

This script will:
1. Run clippy with strict warnings (`-D warnings`)
2. Execute all workspace tests
3. Run documentation tests
4. Report test statistics

### Individual Test Suites

Run tests for a specific package:
```bash
cargo test -p psst-core
cargo test -p psst-cli
```

Run a specific test file:
```bash
cargo test --test item_id_tests
cargo test --test cache_tests
```

Run a specific test:
```bash
cargo test item_id_from_base16_with_valid_input
```

### Continuous Integration

The CI pipeline uses `./scripts/run-tests.sh` to gate all pull requests and commits. Tests must pass before code can be merged.

## Test Organization

### Unit Tests

Unit tests live in two locations:

1. **Inline tests**: In the same file as the code being tested
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_something() {
           // test implementation
       }
   }
   ```

2. **Integration tests**: In the `tests/` directory of each crate
   ```
   psst-core/tests/
   ├── item_id_tests.rs      (36 tests)
   ├── cache_tests.rs        (10 tests)
   ├── connection_tests.rs   (9 tests)
   └── protobuf_failure.rs   (1 test)
   
   psst-cli/tests/
   ├── cli_errors.rs         (1 test)
   └── cli_integration_tests.rs (7 tests)
   ```

## Test Coverage by Module

### psst-core

#### item_id Module (36 tests)
- Base16/Base62 encoding and decoding
- URI parsing for tracks, episodes, and podcasts
- Local file ID registry
- Round-trip conversions
- Edge cases: empty strings, invalid characters, boundary values

#### cache Module (10 tests)
- File creation and directory structure
- Track and episode serialization/deserialization
- Cache clearing and recreation
- Error handling: corrupted data, missing files
- Collision prevention between different item IDs

#### connection Module (9 tests)
- Diffie-Hellman key exchange
- Shared secret generation
- Key consistency and determinism
- Edge cases: empty keys, multiple exchanges

#### error Module (6 tests)
- Error display formatting
- Error type conversions
- Channel error handling

#### util Module (3 tests)
- Protobuf serialization/deserialization
- File offset operations
- Error handling for invalid data

### psst-cli

#### CLI Integration (8 tests)
- Missing credentials detection
- Track ID validation
- Parameter handling
- Error message formatting

## Writing Good Tests

### Test Naming Convention

Use descriptive names that explain what is being tested:
```rust
#[test]
fn function_name_with_valid_input() { }

#[test]
fn function_name_with_invalid_input_returns_error() { }

#[test]
fn function_name_with_empty_input() { }
```

### Testing Edge Cases

Always test:
- Empty inputs (`""`, `vec![]`, `None`)
- Boundary values (0, -1, MAX, MIN)
- Invalid inputs (malformed data, wrong types)
- Error paths (missing files, network failures)

Example:
```rust
#[test]
fn handles_empty_string() {
    let result = parse("");
    assert!(result.is_some()); // or is_none() depending on spec
}

#[test]
fn handles_invalid_format() {
    let result = parse("invalid@#$");
    assert!(result.is_none());
}
```

### Testing Error Conditions

Use temporary directories and files to test file operations:
```rust
use tempfile::TempDir;

#[test]
fn cache_handles_corrupted_data() {
    let temp_dir = TempDir::new().unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf()).unwrap();
    
    // Write invalid data
    fs::write(cache_file_path, b"corrupted").unwrap();
    
    // Verify error handling
    let result = cache.get_item();
    assert!(result.is_none());
}
```

### Testing Panics

Use `#[should_panic]` for functions that should panic:
```rust
#[test]
#[should_panic(expected = "expected error message")]
fn function_panics_on_invalid_state() {
    dangerous_operation();
}
```

### Avoiding Test Interference

When using shared state (like the LocalItemRegistry):
- Use unique identifiers for test data
- Consider test isolation strategies
- Clean up resources in test teardown

```rust
#[test]
fn test_with_unique_path() {
    // Use unique paths to avoid interference from other tests
    let path = PathBuf::from("/tmp/test_unique_xyz123.mp3");
    // test implementation
}
```

## Test Data and Fixtures

### Using Temporary Files

Use the `tempfile` crate for test files:
```rust
use tempfile::TempDir;

let temp_dir = TempDir::new().expect("failed to create temp dir");
let file_path = temp_dir.path().join("test_file.dat");
```

### Mock Data

Create helper functions for common test data:
```rust
fn create_test_track() -> Track {
    Track {
        name: Some("Test Track".to_string()),
        duration: Some(180000),
        // ... other fields
    }
}
```

## Debugging Tests

### Running Tests with Output

```bash
cargo test -- --nocapture
```

### Running Tests with Logging

```bash
RUST_LOG=debug cargo test
```

### Running a Single Test

```bash
cargo test specific_test_name -- --exact
```

## Code Coverage

While code coverage tools are not currently integrated, you can manually review coverage by:
1. Ensuring each public function has at least one test
2. Testing all error paths
3. Testing edge cases and boundary conditions

## Best Practices

1. **Test behavior, not implementation**: Tests should verify what the code does, not how it does it
2. **Keep tests focused**: Each test should verify one specific behavior
3. **Make tests readable**: Use clear variable names and comments
4. **Avoid test dependencies**: Tests should be independent and runnable in any order
5. **Test the error path**: Don't just test the happy path
6. **Use assertions effectively**: Choose the right assertion (`assert_eq!`, `assert!`, `matches!`)

## Continuous Improvement

The test suite should grow with the codebase:
- Add tests when fixing bugs
- Add tests for new features
- Improve edge case coverage
- Add integration tests for complex workflows

## Getting Help

If you're unsure about how to test something:
1. Look at existing tests for similar functionality
2. Review this guide
3. Ask in code reviews or discussions
