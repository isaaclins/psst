use std::process::Command;
use std::env;

#[test]
fn cli_exits_with_error_when_username_missing() {
    let binary = env!("CARGO_BIN_EXE_psst-cli");
    
    let output = Command::new(binary)
        .env_remove("SPOTIFY_USERNAME")
        .env("SPOTIFY_PASSWORD", "dummy-pass")
        .arg("test-track-id")
        .output()
        .expect("failed to invoke psst-cli");
    
    assert!(
        !output.status.success(),
        "psst-cli should exit with failure when username is missing"
    );
}

#[test]
fn cli_exits_with_error_when_password_missing() {
    let binary = env!("CARGO_BIN_EXE_psst-cli");
    
    let output = Command::new(binary)
        .env("SPOTIFY_USERNAME", "dummy-user")
        .env_remove("SPOTIFY_PASSWORD")
        .arg("test-track-id")
        .output()
        .expect("failed to invoke psst-cli");
    
    assert!(
        !output.status.success(),
        "psst-cli should exit with failure when password is missing"
    );
}

#[test]
fn cli_exits_with_error_when_both_credentials_missing() {
    let binary = env!("CARGO_BIN_EXE_psst-cli");
    
    let output = Command::new(binary)
        .env_remove("SPOTIFY_USERNAME")
        .env_remove("SPOTIFY_PASSWORD")
        .arg("test-track-id")
        .output()
        .expect("failed to invoke psst-cli");
    
    assert!(
        !output.status.success(),
        "psst-cli should exit with failure when credentials are missing"
    );
}

#[test]
fn cli_prints_error_message_for_missing_track_id() {
    let binary = env!("CARGO_BIN_EXE_psst-cli");
    
    let output = Command::new(binary)
        .env("SPOTIFY_USERNAME", "dummy-user")
        .env("SPOTIFY_PASSWORD", "dummy-pass")
        .output()
        .expect("failed to invoke psst-cli");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Expected <track_id> in the first parameter"),
        "expected error message not found in stderr: {stderr}"
    );
}

#[test]
fn cli_accepts_track_id_parameter() {
    let binary = env!("CARGO_BIN_EXE_psst-cli");
    
    // We're just testing that the CLI accepts the parameter format
    // It will fail later in authentication, but that's expected
    let output = Command::new(binary)
        .env("SPOTIFY_USERNAME", "dummy-user")
        .env("SPOTIFY_PASSWORD", "dummy-pass")
        .arg("4cOdK2wGLETKBW3PvgPWqT")
        .output()
        .expect("failed to invoke psst-cli");
    
    // Should at least parse the track ID without panicking on missing parameter
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Expected <track_id> in the first parameter"),
        "should not error on missing track_id when it's provided"
    );
}

#[test]
fn cli_handles_empty_track_id() {
    let binary = env!("CARGO_BIN_EXE_psst-cli");
    
    let output = Command::new(binary)
        .env("SPOTIFY_USERNAME", "dummy-user")
        .env("SPOTIFY_PASSWORD", "dummy-pass")
        .arg("")
        .output()
        .expect("failed to invoke psst-cli");
    
    // Empty string is still a parameter, so shouldn't complain about missing parameter
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Expected <track_id> in the first parameter"),
        "empty track_id is still a parameter"
    );
}

#[test]
fn cli_handles_multiple_arguments() {
    let binary = env!("CARGO_BIN_EXE_psst-cli");
    
    let output = Command::new(binary)
        .env("SPOTIFY_USERNAME", "dummy-user")
        .env("SPOTIFY_PASSWORD", "dummy-pass")
        .arg("4cOdK2wGLETKBW3PvgPWqT")
        .arg("extra-arg")
        .output()
        .expect("failed to invoke psst-cli");
    
    // Should process first argument and ignore extras
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Expected <track_id> in the first parameter"),
        "should not error when extra arguments are provided"
    );
}
