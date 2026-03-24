---
phase: 1
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test framework (`#[cfg(test)]` + `#[test]`) |
| **Config file** | None — Rust tests need no config file |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test -- --include-ignored` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test -- --include-ignored`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | AUTH-01 | integration | `cargo test --test init_test` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | AUTH-02 | unit | `cargo test config::tests::env_var_precedence` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | AUTH-03 | unit | `cargo test config::tests::file_permissions` | ❌ W0 | ⬜ pending |
| 01-01-04 | 01 | 1 | AUTH-04 | integration | `cargo test --test ping_test` | ❌ W0 | ⬜ pending |
| 01-01-05 | 01 | 1 | CONF-01 | unit | `cargo test config::tests::load_and_save` | ❌ W0 | ⬜ pending |
| 01-01-06 | 01 | 1 | CONF-02 | integration | `cargo test --test config_show_test` | ❌ W0 | ⬜ pending |
| 01-01-07 | 01 | 1 | CONF-03 | unit | `cargo test config::tests::set_dotted_path` | ❌ W0 | ⬜ pending |
| 01-01-08 | 01 | 1 | CONF-04 | unit | `cargo test config::tests::precedence_order` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | ERRH-01 | integration | `cargo test --test exit_code_test` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | ERRH-02 | unit | `cargo test error::tests::error_formatting` | ❌ W0 | ⬜ pending |
| 01-02-03 | 02 | 1 | ERRH-03 | integration | `cargo test --test help_examples_test` | ❌ W0 | ⬜ pending |
| 01-02-04 | 02 | 1 | UX-02 | integration | `cargo test --test quiet_mode_test` | ❌ W0 | ⬜ pending |
| 01-02-05 | 02 | 1 | UX-03 | integration | `cargo test --test version_test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/` directory — create integration test directory
- [ ] `assert_cmd = "2.0"` in `[dev-dependencies]` — CLI binary testing
- [ ] `predicates = "3.0"` in `[dev-dependencies]` — output assertion matchers
- [ ] `tempfile = "3.0"` in `[dev-dependencies]` — temporary directories for config tests
- [ ] Unit test modules (`#[cfg(test)] mod tests`) in each source file
- [ ] Test fixtures: sample config.toml files for config loading tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Init wizard interactive flow | AUTH-01 | dialoguer prompts require TTY interaction | Run `pipelite init`, follow prompts, verify config created |
| Colored error output | ERRH-02 | ANSI color rendering needs visual check | Run invalid command in TTY, verify red/yellow/dim colors |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
