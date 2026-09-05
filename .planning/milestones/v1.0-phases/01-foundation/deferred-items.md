# Deferred Items - Phase 01

## Pre-existing Test Flakiness

**config::tests::config_path_env_override** and **config::tests::env_var_precedence_server_url** are flaky when run in parallel with other tests that manipulate the same env vars. The Rust 2024 unsafe env var APIs combined with parallel test execution cause race conditions. These tests pass when run in isolation (`cargo test config_path_env_override`). Consider using a test mutex or `serial_test` crate.

**CARRIED to v1.1 milestone audit (probed 2026-09-04):** still flaky under
parallel execution with a real `~/.pipelite/config.toml` present — observed
4 failing runs in 20 (`env_var_precedence_server_url` and sibling
`env_var_precedence_api_key`; details in
`.planning/phases/08-foundations-error-layer-models-pagination/deferred-items.md`
item 3, same root-cause family). `config_path_env_override` itself did not
fail in those 20 runs but shares the env-mutation race. Candidate fix
unchanged: test mutex or `serial_test`-style serialization, or hermetic
temp config paths.
