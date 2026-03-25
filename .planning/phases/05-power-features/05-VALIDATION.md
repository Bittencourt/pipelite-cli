---
phase: 5
slug: power-features
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | assert_cmd 2.0 + predicates 3.0 (integration), built-in #[test] (unit) |
| **Config file** | tests/ directory with integration tests |
| **Quick run command** | `cargo test --lib -q` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -q`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | CACH-01 | unit | `cargo test cache::tests -q` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | CACH-02 | integration | `cargo test --test cache_test -q` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | CACH-03 | unit | `cargo test cache::tests::completions -q` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 2 | DASH-01 | integration | `cargo test --test dashboard_test -q` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 2 | DASH-02 | unit | `cargo test commands::dashboard::tests -q` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 2 | UX-01 | integration | `cargo test --test splash_test -q` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/cache.rs` — unit tests for get/set/clear/invalidate/TTL expiry
- [ ] `tests/cache_test.rs` — integration test for `pipelite cache clear` and `pipelite cache refresh`
- [ ] `tests/dashboard_test.rs` — integration test for `pipelite dashboard` output
- [ ] `tests/splash_test.rs` — integration test for no-subcommand splash behavior

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Splash ASCII art renders correctly | UX-01 | Visual appearance | Run `pipelite` with no args on TTY, verify bold block ASCII logo with color |
| Dynamic completions in actual shell | CACH-03 | Requires real shell integration | Run `pipelite cache refresh`, then tab-complete `pipelite deals get <TAB>` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
