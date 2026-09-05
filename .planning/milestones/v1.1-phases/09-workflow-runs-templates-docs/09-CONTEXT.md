# Phase 9: Workflow Runs, Templates & Docs - Context

**Gathered:** 2026-09-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can observe workflow executions, reuse workflow templates, and fetch the server's API contract — the observe-and-react half of automation. Covers WRUN-01..03 (runs list/detail/watch), TPL-01 (templates CRUD minus update), DOCS-01 (OpenAPI fetch). Mutually independent of Phase 10 (notes); both consume the Phase 8 error layer.

</domain>

<decisions>
## Implementation Decisions

### Runs List & Detail UX
- **Nested command placement**: `pipelite workflows runs list --workflow <id>`; detail via `pipelite workflows runs <id>` (runs are workflow-scoped resources).
- **`--status` passes through** — server validates; empty result with `--status` set → hint listing valid statuses.
- **Test-run opt-in flag is `--include-dry-run`** (NOT `--dry-run`, which is the global preview flag). Without it, test runs are hidden; when results are empty and dry-run runs exist server-side, hint: "N test run(s) hidden — pass --include-dry-run".
- **Run detail = one table row per step** (node, status, input, output, error, duration), truncating long cells in table mode; `--format json` passes the run through verbatim.

### --watch Semantics
- **2s fixed poll interval**, no flag (documented in help).
- **No timeout** — watch until terminal state or Ctrl-C (interrupt → exit 130).
- **Exit codes**: default exit 0 when run reaches a terminal state (even failed); `--exit-status` maps failed/cancelled → 1; Ctrl-C → 130.
- **Silent polling**: one stderr line per state change (`run <id>: running → completed`); final state renders like normal detail; `--quiet` suppresses stderr progress lines.

### Templates
- **Top-level `pipelite templates`** (list/get/create/delete); after_help notes templates instantiate workflows.
- **Create**: `templates create --name X --workflow <id>` + optional flags, plus `--stdin` JSON body for full control (mirrors existing create patterns).
- **No update subcommand** — `templates --help` after_help states: "The server exposes no template update — delete and recreate to change a template"; `pipelite templates update` → exit 2 InvalidInput with that hint.
- **Delete uses the confirm_destructive pattern**: TTY prompt, `--force` opt-out, `--dry-run` preview (consistent with Phase 7 batch deletes).

### docs Command
- **Raw OpenAPI JSON to stdout** (pretty-printed); `--save FILE` writes and prints one-line confirmation; `--format` ignored (spec is not tabular).
- **No Authorization header** — fetches without authentication per DOCS-01.
- **`--save` refuses to overwrite an existing file** unless `--force`; creates missing parent dirs.
- **Docs-endpoint errors flow through the Phase 8 error layer** with hint: "the server may not expose the docs endpoint — check server version".

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 8 error layer: parse_rfc7807, Forbidden + per-surface hints, 409 activation hint (src/api/mod.rs, src/error.rs)
- Phase 7 batch utility: confirm_destructive pattern, run_batch_delete, collect_delete_ids (src/batch.rs) — for templates delete
- send_with_retry on all API calls (429 handling free of charge)
- Stub-server TcpListener test pattern + hermetic env helpers (tests/batch_error_test.rs, tests/error_layer_stub_test.rs)
- output/ renderers; workflow models already use serde_json::Value for complex fields
- `#[serde(flatten)] expanded` pattern from Phase 8 for any extra payload fields

### Established Patterns
- One CLI definition file per entity (src/cli/), one command dir per entity (src/commands/)
- after_help examples on every subcommand; hints on every error; exit 2 = InvalidInput
- Cache invalidation on mutation (workflows use `workflows_`-prefixed keys; runs likely uncached — confirm in research)
- Hidden-flag parse-then-error for impossible commands (Phase 8 pattern)

### Integration Points
- workflows trigger (existing) is the producer of runs — runs list/detail/watch read what it creates; 409 inactive-trigger hint from Phase 8 lands here
- templates create instantiates from workflows — needs workflow id completion (clap_complete cache pattern)
- docs is auth-free — bypasses the authenticated client constructor or uses a raw request path

</code_context>

<specifics>
## Specific Ideas

- ROADMAP marks Phase 9 parallelizable with Phase 10 — plan ordering inside this phase is free
- Criterion 1 wording "test runs appear only with --dry-run opt-in" resolved to `--include-dry-run` to avoid colliding with the global preview flag (recorded above)
- Criterion 3: `--exit-status` gates exit codes; default watch exits 0 on any terminal state

</specifics>

<deferred>
## Deferred Ideas

- WRUN-04 (batch failed-ID pipeable summary for retries) — deferred to future milestones per REQUIREMENTS.md
- Run cancellation/termination commands — not in server API scope for this milestone
- `docs` endpoint summary table rendering — rejected (raw spec only)

</deferred>
