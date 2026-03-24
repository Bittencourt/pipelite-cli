use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn nonexistent_command_exits_with_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("nonexistent-command")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn ping_with_bad_url_exits_with_code_1() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");

    let content = r#"
[server]
url = "http://127.0.0.1:1"
api_key = "pk_test_invalid"

[output]

[display]
"#;

    std::fs::write(&config_path, content).unwrap();

    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("ping")
        .env("PIPELITE_CONFIG", config_path.to_str().unwrap())
        .assert()
        .failure()
        .code(1);
}
