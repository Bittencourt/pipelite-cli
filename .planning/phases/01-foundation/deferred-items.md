# Deferred Items - Phase 01

## Pre-existing Test Flakiness

**config::tests::config_path_env_override** and **config::tests::env_var_precedence_server_url** are flaky when run in parallel with other tests that manipulate the same env vars. The Rust 2024 unsafe env var APIs combined with parallel test execution cause race conditions. These tests pass when run in isolation (`cargo test config_path_env_override`). Consider using a test mutex or `serial_test` crate.
