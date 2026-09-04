# Phase 11 Validation Map — Webhooks, Trash & Audit

**Requirement → test → gate mapping.** Every requirement traces to named stub-server tests plus typed grep gates. Wave-0 gaps (test files) are closed inside the plans — the secret-truncation test is written RED before any create handler exists (ROADMAP note), and the purge zero-HTTP refusal test is pinned on an unreachable server.

## Requirement → Test → Gate Map

| Req | Behavior under test | Test file (created in) | Pinning tests | Automated gate |
|-----|--------------------|------------------------|---------------|----------------|
| WHOK-01 | Webhook list/get/create/update/delete against stubs; merged PUT body; verbatim stdin; delete contract | `tests/webhooks_stub_test.rs` (11-01 T1/T2) | list_table, get_placeholder, create_table_basic, create_stdin_verbatim, update_merges, update_stdin_no_get, delete_force, delete_non_tty_refusal | `cargo test --test webhooks_stub_test` |
| WHOK-02 | 13-event allow-list, exit 2 pre-HTTP (counter 0), all 13 listed in the hint; enforced on --events AND --stdin arrays; https-only URL mirror | `tests/webhooks_stub_test.rs` (11-01 T1/T2) | create_unknown_event, create_stdin_bad_event, create_http_url, update_flag_validation_precedes_get + validator unit tests | same + `grep '"deal.stage_changed"' src/commands/webhooks/mod.rs` == 1 |
| WHOK-03 | Secret exactly once on create — full 64 chars, own line, save-it-now warning; truncation-proof vs 120-col renderer; never in list/get (placeholder), never on disk (cache = id/url only) | `tests/webhooks_stub_test.rs` (11-01 T1 — FIRST tests in file) | secret_shown_once_full_on_own_line (RED first), secret_json_format, list_json_placeholder, delete_invalidates_cache + HOME-walk cache-absence + `grep -ci secret src/cache.rs` == 0 | same |
| TRSH-01 | trash list: 9 singular/plural aliases normalized pre-HTTP → plural tab on the wire; default no-type (server deals tab); bounded --all fan-out with 4 break conditions; linked_parents/deleted_by truncated cells + full json | `tests/trash_stub_test.rs` (11-02 T1) | list_type_deal_alias, list_type_aliases, list_unknown_type, list_default_no_type, list_all_fanout_partial_break, list_all_empty_first_page, list_linked_parents_truncation, list_deleted_by_variants + normalizer unit matrix | `cargo test --test trash_stub_test` |
| TRSH-02 | restore <type> <id>: no confirm, plural-tab URL, 204 success, 404 re-wrap hint, foreign-403 general hint (never purge), dry-run | `tests/trash_stub_test.rs` (11-02 T2) | restore_deal_alias, restore_people_alias, restore_404_rewrap, restore_403_general_hint, restore_dry_run | same |
| TRSH-03 | purge: admin-only hint, strongest confirm (scope + count + "permanently destroys"), --force bypass, **non-TTY no-force → exit 2 ZERO HTTP (pinned)**, scope validated before any HTTP, continue-on-error fan-out with N ok / M failed | `tests/trash_stub_test.rs` (11-02 T2) | **purge_refusal_zero_http (pinned: `.code(2)`, unreachable server, lacks "Connection failed")**, purge_scope_validation_first, purge_force_fanout, purge_continue_on_error, purge_403_admin_hint, purge_dry_run_victims, purge_prompt unit (wording gate) + `grep -c Validation src/commands/trash/purge.rs` == 0 | same |
| AUDT-01 | audit list: 4 filters verbatim on wire; empty flags omitted; limit clamped 1..=100; no --all; default ts/actor/action/entity columns; --json changes verbatim; merged tolerated | `tests/audit_stub_test.rs` (11-03 T1) | filters_verbatim, empty_flag_omitted, limit_clamped_high/low, json_changes, json_all_ids, merged_action_tolerated, entity_cell_truncation + `grep -c `long = "all"` src/cli/audit.rs` == 0 (flag definition — help wording "(no --all)" is required text) | `cargo test --test audit_stub_test` |
| AUDT-02 | First-class Forbidden hints on every admin-gated surface; gate-before-validation ordering probed | `tests/audit_stub_test.rs` (11-03 T1) + trash/webhooks suites | forbidden_empty_filter (pinned), forbidden_invalid_filter_still_gate (ordering), purge_403_admin_hint (11-02), get/update/delete_foreign_403 (11-01) | `cargo test --test audit_stub_test --test trash_stub_test --test webhooks_stub_test` |

## Cross-Cutting Gates

| Concern | Gate |
|---------|------|
| Full-suite phase gate | `cargo test` green after each plan's final task (help truthfulness + docs sections + all prior suites) |
| Zero-HTTP rejections (locked codes) | 2 = unknown event / non-https url / unknown trash type / purge refusal / update-no-flags · 1 = webhook delete refusal (Validation, deliberately distinct from purge) — each proven with counter == 0 or unreachable-server runs lacking "Connection failed" |
| Grep pins | `--description` absent from cli/webhooks.rs and api-reference webhooks section (amendment) · "secret" absent from cache.rs · `Validation` absent from purge.rs · `--all` flag definition absent from cli/audit.rs (help wording "(no --all)" required) · normalize called by all three trash subcommands · forbidden_hint table untouched (hints pre-registered Phase 8) |
| Confirmation hierarchy | purge (strongest wording + exit-2 refusal) > webhook delete (standard + exit-1) > restore (none) — contracts coexist, each stub-tested |
| Conventions | No unwrap/expect in production code · hints on every error · --dry-run/--no-input/--quiet/--no-color respected (secret lines exempt from --quiet: they are the payload) · trash/audit never cached |

## Sampling Rate

- **Per task commit:** `cargo test --test <surface>_stub_test` (per-plan suite)
- **Per wave:** `cargo test` full suite (plans execute sequentially: 11-01 → 11-02 → 11-03)
- **Phase gate:** full suite green before `/gsd-verify-work`

## Wave-0 Gap Status

| Test file | Requirement coverage | Created in |
|-----------|---------------------|------------|
| tests/webhooks_stub_test.rs | WHOK-01..03 | 11-01 Task 1 (secret tests RED-first) |
| tests/trash_stub_test.rs | TRSH-01..03 | 11-02 Task 1 (refusal pinned in Task 2) |
| tests/audit_stub_test.rs | AUDT-01..02 | 11-03 Task 1 |

No framework installs needed — assert_cmd + tests/common/mod.rs helpers cover every shape (head+body-capturing stub, unreachable-server pre-HTTP probes, per-child HOME env for cache hermeticity).
