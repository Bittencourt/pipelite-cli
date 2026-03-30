---
phase: 1
slug: batch-operations-for-all-entities
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[cfg(test)]` + `cargo test`) |
| **Config file** | Cargo.toml (test dependencies already configured) |
| **Quick run command** | `cargo test batch` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test batch`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | batch-update | unit | `cargo test batch_update` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | batch-delete | unit | `cargo test batch_delete` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | error-handling | unit | `cargo test batch_error` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Batch module test stubs for update/delete operations
- [ ] Test helpers for simulating partial batch failures

*Existing test infrastructure covers basic patterns — stubs needed for new batch operations.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Confirmation prompt on batch delete | D-05 | Requires TTY interaction | Run `pipelite deals delete id1 id2` in terminal, verify prompt appears |
| Stdin detection | D-01 | Requires piped input | Pipe JSON via `echo '[...]' \| pipelite deals update --stdin` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
