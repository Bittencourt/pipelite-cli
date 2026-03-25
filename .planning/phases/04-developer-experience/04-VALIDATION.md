---
phase: 4
slug: developer-experience
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | assert_cmd 2.0 + predicates 3.0 (integration), cargo test (unit) |
| **Config file** | Cargo.toml [dev-dependencies] |
| **Quick run command** | `cargo test --test cli_skeleton` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test cli_skeleton`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | INTR-03 | integration | `cargo test --test headless_test` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | HEAD-01 | integration | `cargo test --test headless_test::no_input_flag` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | HEAD-02 | integration | `cargo test --test headless_test::missing_fields` | ❌ W0 | ⬜ pending |
| 04-01-04 | 01 | 1 | HEAD-03 | integration | `cargo test --test headless_test::flags_only` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 1 | SHLL-01 | integration | `cargo test --test completions_test::bash` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 1 | SHLL-02 | integration | `cargo test --test completions_test::zsh` | ❌ W0 | ⬜ pending |
| 04-02-03 | 02 | 1 | SHLL-03 | integration | `cargo test --test completions_test::fish` | ❌ W0 | ⬜ pending |
| 04-02-04 | 02 | 1 | UX-04 | integration | `cargo test --test dry_run_test` | ❌ W0 | ⬜ pending |
| 04-01-05 | 01 | 1 | INTR-01 | manual-only | Manual: run `pipelite deals create` in terminal | N/A | ⬜ pending |
| 04-01-06 | 01 | 1 | INTR-02 | manual-only | Manual: run create and verify dropdown appears | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/headless_test.rs` — stubs for INTR-03, HEAD-01, HEAD-02, HEAD-03
- [ ] `tests/completions_test.rs` — stubs for SHLL-01, SHLL-02, SHLL-03
- [ ] `tests/dry_run_test.rs` — stubs for UX-04

*Note: INTR-01 and INTR-02 are manual-only (require real TTY with interactive terminal input).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Create with no flags prompts interactively | INTR-01 | Requires real TTY with stdin attached | Run `pipelite deals create` in terminal, verify prompts appear |
| FuzzySelect dropdown for known values | INTR-02 | Requires interactive terminal for fuzzy selection UI | Run create command, verify dropdown with known pipeline/stage values |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
