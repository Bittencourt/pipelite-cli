---
phase: 09-workflow-runs-templates-docs
verified: 2026-09-03T21:05:00Z
status: passed
score: 7/7 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run `pipelite workflows runs get <run_id> --workflow <wf_id> --watch --exit-status` against a live server while a real workflow run executes (one that ends failed, one that ends completed)"
    expected: "Transitions stream to stderr ~every 2s as the run progresses; waiting runs keep polling; on completion the detail renders and the process exits 0 (completed) / 1 (failed with --exit-status); Ctrl-C reports exit 130 in the shell"
    why_human: "Stub tests prove poll counts, transition lines, and exit codes, but real run cadence, long-run ergonomics, and shell signal reporting need a live server and a human at the terminal"
  - test: "Run `pipelite docs` and `pipelite docs --save spec.json` against the live Pipelite server"
    expected: "The real ~87KB OpenAPI 3.1 spec pretty-prints to stdout / saves as valid JSON; the live request carries no Authorization header (confirm via server access log)"
    why_human: "Fixture stubs serve a miniature spec; the real spec's size/shape and the server-side no-auth confirmation are only observable against the deployed server"
  - test: "On a live server, create a dry-run (test) workflow execution, then run `pipelite workflows runs list --workflow <wf_id>` without and with --include-dry-run"
    expected: "Without the flag the test run is hidden (server-side) with the hidden-count hint when the page is otherwise empty; with the flag it appears and rows carry dry_run=true"
    why_human: "Verifies the real server's dry-run hiding contract matches the researched wire shape the CLI was built against"
---

# Phase 9: Workflow Runs, Templates & Docs Verification Report

**Phase Goal:** Users can observe workflow executions, reuse workflow templates, and fetch the server's API contract — the observe-and-react half of automation
**Verified:** 2026-09-03T21:05:00Z
**Status:** human_needed (all machine-verifiable must-haves verified; live-server UAT items remain)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ----- | ------ | -------- |
| 1 | Runs list filtered by `--status`; test runs only via `--include-dry-run`; hidden-dry-run hint backed by EXACTLY ONE probe (SC-1 / WRUN-01) | ✓ VERIFIED | `runs/list.rs:44-68` (status hint, limit-1 `dry_run=true` probe gated on `meta.total`, quiet-suppressed); wire probes P11/P12: `dry_run=true` only with flag, probe carries `limit=1`, hint reports true count "2 test run(s) hidden"; suite 21/21 green |
| 2 | Empty `--status` result prints the five valid statuses, never a client-side rejection, exit 0 (SC-1) | ✓ VERIFIED | `runs/list.rs:45-50` hint to stderr; no client-side status validation anywhere in the handler; suite asserts exit 0 + hint |
| 3 | Run detail: every step flattened to rows (node, status, input, output, error, duration), JSON passthrough verbatim (SC-2 / WRUN-02) | ✓ VERIFIED | `runs/detail.rs:126-221` (`render_detail` seam, steps_table with `ContentArrangement::Dynamic` + width 120 non-TTY, compact_json null→empty); probe P13 rendered summary + 2 step rows with compact JSON cells; probe P14 JSON passthrough with `steps` + `depth`, no wrapper key |
| 4 | `--watch` polls until terminal; `--exit-status` maps failed→exit 1; waiting keeps polling; default exit 0 on any terminal state (SC-3 / WRUN-03) | ✓ VERIFIED | `runs/detail.rs:61-121` (2s `tokio::time::sleep`, `run_status_is_terminal`/`watch_exit_code` helpers, one stderr transition per change, quiet-suppressed, WR-02 3-failure tolerance); probes P15 (failed+flag→rc 1, 2 polls, "running → failed"), P16 (completed→rc 0, 2 polls), P17 (already-terminal→1 request); suite covers waiting non-terminality + SIGINT signal 2 |
| 5 | Templates list/get/create/delete; hidden `update` → exit 2 + delete-and-recreate hint, zero HTTP (SC-4 / TPL-01) | ✓ VERIFIED | `src/cli/templates.rs:67-69` (`hide = true` Update); probe P1 rc=2 + "delete and recreate" + no connection error; P2 help states "no template update", does not advertise update; probes P22-P25 cover list/get/delete (DELETE on wire, non-TTY refusal rc 1 pre-HTTP) |
| 6 | `templates create --workflow` maps `triggers[0]`→`trigger`, multi-trigger warning, exactly-one source enforced pre-HTTP; `--stdin` verbatim (SC-4 / TPL-01) | ✓ VERIFIED | `templates/create.rs:30-171`; probes P18 (POST body `trigger` == workflow's triggers[0], nodes copied, no "triggers" leak), P19 ("2 triggers" warning), P20 (zero triggers → rc 2 after exactly 1 fetch), P21 (no source → rc 2 naming all three); `post_workflow_template_raw` preserves stdin verbatim |
| 7 | `pipelite docs [--save FILE]` fetches OpenAPI spec WITHOUT Authorization; overwrite refusal unless `--force` pre-HTTP; parent dirs created (SC-5 / DOCS-01) | ✓ VERIFIED | `api/mod.rs` `get_docs` builds local headerless `reqwest::Client::builder()` (0 refs to `self.client` in body — grep-proven); probe P5: raw request head has no `authorization:` line, compact stub JSON rendered pretty; probes P6/P7/P8/P9: refusal rc 2 with sentinel intact + 0 requests, `--force` overwrites, nested parent dirs created; P10: 404 → locked "may not expose the docs endpoint" hint with server detail preserved, generic hint gone |

**Score:** 7/7 truths verified (all 5 ROADMAP Success Criteria + 2 plan-level truths: status-hint variant of SC-1, trigger-mapping detail of TPL-01)

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `src/api/models.rs` | Run/Step/Detail + Template models, 5/6-value status enums with `serde(other)` Unknown, duration/terminal/exit-code helpers, table configs | ✓ VERIFIED | All structs/enums/fns present (lines 478-713); unit truth tables + round-trip tests in the 146-test bin |
| `src/api/mod.rs` | `list_workflow_runs`, `get_workflow_run` (both ids in path), 4 template methods + `post_workflow_template_raw`, `get_docs` (headerless client) | ✓ VERIFIED | Lines 887-1041; envelope (`ApiSingleResponse`) unwrapping matches the server contract on every single-item fetch |
| `src/cli/workflows.rs` | Nested `Runs` group; required `--workflow`; `--status`/`--include-dry-run`; `--watch`/`--exit-status` | ✓ VERIFIED | Get args lines 268-272; after_help includes `--watch --exit-status` example (line 218) |
| `src/commands/workflows/runs/{mod,list,detail}.rs` | List hints + probe; detail render seam + watch loop | ✓ VERIFIED | 15/88/221 lines, fully wired, no stubs |
| `src/cli/templates.rs` + `src/commands/templates/*` | List/Get/Create/Delete + hidden Update; confirmation flow; batch routing | ✓ VERIFIED | All 6 files exist (20-211 lines), dispatched from `main.rs:98`; `run_batch_delete` wired (delete.rs:30) |
| `src/cli/docs.rs` + `src/commands/docs.rs` | DocsArgs `--save/--force`, format-ignored note; handler with pre-HTTP refusal + error re-wrap | ✓ VERIFIED | Dispatched at `main.rs:102`; `DOCS_ERROR_HINT` re-wraps NotFound/Api preserving detail |
| `src/cache.rs` | `KEY_TEMPLATES` + `TTL_TEMPLATES` | ✓ VERIFIED | Lines 33/35; list caches, create/delete invalidate (create.rs:177) |
| `tests/common/mod.rs` | Shared head+body-capturing stub server (4-tuple) | ✓ VERIFIED | 163 lines; used by all three phase-9 test files |
| `tests/workflow_runs_stub_test.rs` / `templates_stub_test.rs` / `docs_stub_test.rs` | WRUN/TPL/DOCS integration coverage | ✓ VERIFIED | 21 / 16 / 8 tests — all green in `cargo test` |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| runs/list.rs | api list_workflow_runs | `ctx.client.list_workflow_runs(&params)` | ✓ WIRED | list.rs:42, params built from args (34-40) |
| runs/list.rs | probe meta.total gate | empty page + no flag → one limit-1 `dry_run=true` probe | ✓ WIRED | list.rs:51-67, `probe_response.meta.total > 0` gate |
| runs/detail.rs | api get_workflow_run | both ids in path, initial fetch + every poll | ✓ WIRED | detail.rs:62, 88 |
| runs/detail.rs | models watch_exit_code | terminal check + exit from unit-tested helpers | ✓ WIRED | detail.rs:82-84 (`run_status_is_terminal`, `watch_exit_code`) |
| runs/detail.rs | comfy-table steps table | `set_content_arrangement(ContentArrangement::Dynamic)` | ✓ WIRED | detail.rs:202, width 120 non-TTY (205-207) |
| templates/create.rs | api get_workflow | `--workflow` fetch → `triggers.first()` → trigger | ✓ WIRED | create.rs:76-90, multi-trigger warning at 87-89 |
| templates/delete.rs | batch run_batch_delete | multi-ID deletes route through Phase 7 utility | ✓ WIRED | delete.rs:30 |
| commands/docs.rs | api get_docs | sole consumer of the unauthenticated fetch | ✓ WIRED | docs.rs:37; 404/Api re-wrap at 43-61 |
| api get_docs | dedicated headerless client | local `reqwest::Client::builder`, no `self.client` | ✓ WIRED | Body greps: 0 `self.client` refs, 1 local builder; wire-proven (no auth header) |
| main.rs | Commands::Docs / Templates dispatch | variant + arm | ✓ WIRED | main.rs:98, 102 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| runs list | `response.data` | `list_workflow_runs` → server envelope `{"data":[...],"meta":{...}}` | Yes (verified server shape; live rows in probes) | ✓ FLOWING |
| runs detail | `detail` | `get_workflow_run` → `{"data":{...steps}}` unwrap | Yes (probe P13/P14 real-shaped fixture parsed+rendered) | ✓ FLOWING |
| templates list/get/create/delete | template records | 4 client methods + raw POST twin, envelope-unwrapped | Yes (probes P18-P25) | ✓ FLOWING |
| docs | `spec: serde_json::Value` | `get_docs` headerless fetch | Yes (probe P5: stub spec pretty-printed; real 87KB spec pending live UAT) | ✓ FLOWING |

### Behavioral Spot-Checks

25 independent probes driven by a fresh harness (`/tmp/opencode/phase9_probes.py`) against the built debug binary + scripted stub servers — **25/25 passed**. All initial probe failures were harness fixture bugs (missing `{"data":...}` envelope, stub script exhaustion, ambient JSON config default), each corrected in the harness — notably the CLI correctly enforced the server envelope contract on every single-item fetch.

| Behavior | Command (probe) | Result | Status |
| -------- | ------- | ------ | ------ |
| templates update → rc 2 + hint, 0 HTTP | `templates update tpl_1 --name X` (unreachable server) | rc=2, "delete and recreate", no Connection error | ✓ PASS |
| help truthfulness | `templates --help`, `docs --help` | "no template update" present, update hidden; `--save` + "ignored" present | ✓ PASS |
| required --workflow pre-HTTP | `workflows runs get run_1` | rc=2, clap required-arg error, no HTTP | ✓ PASS |
| docs no-auth + pretty | `docs` vs stub | 0 `authorization:` head lines, pretty JSON, 1 request | ✓ PASS |
| docs --save contract | refusal / `--force` / parent dirs / second refusal | rc 2 + sentinel intact + 0 reqs; overwrite + confirmation; nested dirs created | ✓ PASS |
| docs 404 re-wrap | `docs` vs 404 stub | locked hint + "Not Found" detail preserved, generic hint absent | ✓ PASS |
| runs list probe discipline | empty page vs stub | 2 requests total, probe has `limit=1`+`dry_run=true`, hint "2 test run(s) hidden" | ✓ PASS |
| runs list params on wire | `--status completed --include-dry-run` | request line `?status=completed&dry_run=true&limit=50&offset=0`, no probe | ✓ PASS |
| detail table flatten | `runs get --format table` | summary block + steps table (node/status/input/output/error/duration), compact JSON cells, null→empty | ✓ PASS |
| detail JSON passthrough | `runs get --format json` | parses; run fields + 2 steps, no wrapper key | ✓ PASS |
| watch exit contract | scripted running→failed/completed | `--exit-status`: rc 1 + "running → failed"; default: rc 0; 2 polls each | ✓ PASS |
| watch single-shot terminal | already-completed + `--watch` | 1 request, renders, exits 0 | ✓ PASS |
| templates create mapping | `--workflow` one/two/zero triggers; `--stdin`; conflicts | triggers[0] mapped, nodes copied, no `triggers` leak; "2 triggers" warning; zero-trigger rc 2 after 1 fetch; no-source rc 2 | ✓ PASS |
| templates list/get/delete | vs stubs | rows rendered; DELETE on wire; non-TTY refusal rc 1 pre-HTTP | ✓ PASS |

### Probe Execution

No `scripts/*/tests/probe-*.sh` probes declared by the plans or present conventionally; verification probes were executed directly by the verifier (table above). `cargo test` full suite: **all 28 test binaries green, 0 failed** (371 tests total, including 21 runs + 16 templates + 8 docs stub tests and the SIGINT watch test).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| WRUN-01 | 09-01 | List runs with `--status` filter + test-run opt-in | ✓ SATISFIED | Truth 1-2; note: REQUIREMENTS.md's literal `--dry-run` wording was resolved to `--include-dry-run` to avoid collision with the global preview flag (documented in ROADMAP SC-1 resolution and plan locks) |
| WRUN-02 | 09-01 | Run detail with steps flattened to readable rows | ✓ SATISFIED | Truth 3 |
| WRUN-03 | 09-02 | Watch until completion, honest polling, `--exit-status` | ✓ SATISFIED | Truth 4 |
| TPL-01 | 09-02 | List/get/create/delete templates, no update | ✓ SATISFIED | Truths 5-6 |
| DOCS-01 | 09-03 | Fetch OpenAPI 3.1 spec without auth | ✓ SATISFIED | Truth 7 |

No orphaned requirements: REQUIREMENTS.md maps exactly WRUN-01/02/03, TPL-01, DOCS-01 to Phase 9; all five are claimed by plans and satisfied.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| (none) | — | No TBD/FIXME/XXX/HACK/PLACEHOLDER markers, no `unwrap()`/`expect()`, no stub phrases in any phase-modified production file | — | Clean |

Review-fix items verified landed: WR-01 (`--nodes` + `--workflow` rejection, create.rs:59-68), WR-02 (3-consecutive-poll-failure tolerance, detail.rs:18, 93-119), IN-01/IN-02 (stale `#[allow(dead_code)]` removed — grep clean; unused imports removed).

### Human Verification Required

Automated + wire-level verification is complete for all 7 truths; three items remain that only a live server + human can confirm (listed in frontmatter `human_verification`):

1. **Live `--watch` ergonomics** — real run cadence, Ctrl-C→130 in a real shell, long-run behavior
2. **Live `docs` against the real spec** — the actual ~87KB OpenAPI 3.1 document parses/renders; server log confirms no auth header
3. **Live dry-run hiding** — real server's test-run hiding matches the researched contract the opt-in flag was built against

### Gaps Summary

No gaps. All 5 ROADMAP Success Criteria and all 5 requirement IDs are implemented, wired, and proven by the full test suite (371 tests green) plus 25 independent wire-level probes run by the verifier against the built binary. The `human_needed` status reflects only live-server confirmation items — the CLI's side of every contract (query params, headers, exit codes, poll discipline, hints, envelope handling) is machine-verified.

---

_Verified: 2026-09-03T21:05:00Z_
_Verifier: the agent (gsd-verifier)_
