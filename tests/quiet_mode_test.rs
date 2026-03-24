use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a temp config file and return the dir + path.
fn create_temp_config() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");

    let content = r#"
[server]
url = "https://test.example.com"
api_key = "pk_test_abc123"

[output]

[display]
"#;

    std::fs::write(&config_path, content).unwrap();
    (dir, config_path)
}

#[test]
fn quiet_mode_suppresses_stderr_on_config_show() {
    let (_dir, config_path) = create_temp_config();

    // When piped (non-TTY), output format defaults to JSON, so check for JSON content.
    // The key assertion: quiet mode produces NO stderr output.
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["config", "show", "-q"])
        .env("PIPELITE_CONFIG", config_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("test.example.com"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn config_show_produces_stdout_output() {
    let (_dir, config_path) = create_temp_config();

    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["config", "show"])
        .env("PIPELITE_CONFIG", config_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("test.example.com"));
}
