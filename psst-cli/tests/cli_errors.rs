use std::process::Command;

#[test]
fn cli_exits_with_error_when_track_id_is_missing() {
    let binary = env!("CARGO_BIN_EXE_psst-cli");

    let output = Command::new(binary)
        .env("SPOTIFY_USERNAME", "dummy-user")
        .env("SPOTIFY_PASSWORD", "dummy-pass")
        .output()
        .expect("failed to invoke psst-cli");

    assert!(
        !output.status.success(),
        "psst-cli should exit with failure when no track id is supplied"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Expected <track_id> in the first parameter"),
        "unexpected stderr: {stderr}"
    );
}
