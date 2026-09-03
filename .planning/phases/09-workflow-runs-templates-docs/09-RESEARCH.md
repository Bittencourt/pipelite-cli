# Phase 9: Workflow Runs, Templates & Docs - Research

**Researched:** 2026-09-03
**Domain:** Pipelite server v2 workflow-observability surfaces (runs, templates, OpenAPI docs) mapped onto the existing pipelite-cli Rust patterns
**Confidence:** HIGH — every endpoint verified by reading server source at `/home/pedro/programming/pipelite` (same authoritative source validated in Phase 8 research); every client pattern verified by reading `src/`

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Runs List & Detail UX**
- **Nested command placement**: `pipelite workflows runs list --workflow <id>`; detail via `pipelite workflows runs <id>` (runs are workflow-scoped resources).
- **`--status` passes through** — server validates; empty result with `--status` set → hint listing valid statuses.
- **Test-run opt-in flag is `--include-dry-run`** (NOT `--dry-run`, which is the global preview flag). Without it, test runs are hidden; when results are empty and dry-run runs exist server-side, hint: "N test run(s) hidden — pass --include-dry-run".
- **Run detail = one table row per step** (node, status, input, output, error, duration), truncating long cells in table mode; `--format json` passes the run through verbatim.

**--watch Semantics**
- **2s fixed poll interval**, no flag (documented in help).
- **No timeout** — watch until terminal state or Ctrl-C (interrupt → exit 130).
- **Exit codes**: default exit 0 when run reaches a terminal state (even failed); `--exit-status` maps failed/cancelled → 1; Ctrl-C → 130.
- **Silent polling**: one stderr line per state change (`run <id>: running → completed`); final state renders like normal detail; `--quiet` suppresses stderr progress lines.

**Templates**
- **Top-level `pipelite templates`** (list/get/create/delete); after_help notes templates instantiate workflows.
- **Create**: `templates create --name X --workflow <id>` + optional flags, plus `--stdin` JSON body for full control (mirrors existing create patterns).
- **No update subcommand** — `templates --help` after_help states: "The server exposes no template update — delete and recreate to change a template"; `pipelite templates update` → exit 2 InvalidInput with that hint.
- **Delete uses the confirm_destructive pattern**: TTY prompt, `--force` opt-out, `--dry-run` preview (consistent with Phase 7 batch deletes).

**docs Command**
- **Raw OpenAPI JSON to stdout** (pretty-printed); `--save FILE` writes and prints one-line confirmation; `--format` ignored (spec is not tabular).
- **No Authorization header** — fetches without authentication per DOCS-01.
- **`--save` refuses to overwrite an existing file** unless `--force`; creates missing parent dirs.
- **Docs-endpoint errors flow through the Phase 8 error layer** with hint: "the server may not expose the docs endpoint — check server version".

### the agent's Discretion
- Plan ordering inside the phase is free (ROADMAP marks Phase 9 parallelizable with Phase 10).
- Implementation details not pinned above (exact flag placement for detail's workflow id, cache TTLs, renderer internals) follow existing codebase patterns.

### Deferred Ideas (OUT OF SCOPE)
- WRUN-04 (batch failed-ID pipeable summary for retries) — deferred to future milestones per REQUIREMENTS.md.
- Run cancellation/termination commands — not in server API scope for this milestone.
- `docs` endpoint summary table rendering — rejected (raw spec only).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WRUN-01 | List runs of a workflow with `--status` filter and `--dry-run` opt-in for test runs | Server contract §1: `GET /api/v1/workflows/{id}/runs?offset&limit&status&dry_run=true`; hiding is server-side; CLI flag is `--include-dry-run`; status enum verified |
| WRUN-02 | View a run's detail including its steps (node, status, input, output, error, timing) flattened to readable rows | Server contract §2: detail response shape with `steps[]`; exact step field names verified in `serializeRunStep` |
| WRUN-03 | Watch a run until completion with `--watch [--exit-status]` via honest polling | Terminal states verified (`completed`/`failed` only); poll loop pattern; exit-code mapping; SIGINT behavior verified against tokio feature set |
| TPL-01 | List, get, create, and delete workflow templates (no update — server has none) | Server contract §3: all four routes verified; create schema verified (no update route exists — confirmed); delete is hard delete |
| DOCS-01 | Fetch the server's OpenAPI 3.1 spec (`pipelite docs [--save FILE]`) without authentication | Server contract §4: `/api/v1/docs` has NO `withApiAuth` wrapper — verified public; Content-Type verified; unauthenticated client path designed |
</phase_requirements>

## Summary

Phase 9 adds three read-mostly surfaces. All five server endpoints were verified by reading the server's Next.js route handlers, Drizzle schema, and serializers directly — the contract in `SERVER-API-DIFF.md` is confirmed accurate with two additions it missed: (1) run rows carry `depth` for nested-workflow executions and the serializer **excludes** the DB columns `context` and `replayed_from_run_id`; (2) the `dry_run=true` opt-in param is an exact-string match (`"true"` only) and hiding is done **server-side**, so the CLI never filters test runs client-side.

Two findings change task shapes: **(a) Contract gap in the locked UX wording** — the server requires BOTH the workflow id and the run id in the detail path (`/workflows/{id}/runs/{runId}`), and there is no way to resolve a run's workflow from the run id alone, so `pipelite workflows runs <run_id>` detail/watch MUST also carry the workflow id (recommended: required `--workflow` flag, matching list's flag). **(b) Template create needs a mapping step** — the server's create schema is `{name, description, category, trigger (single object), nodes}` and has no workflow reference, so `templates create --workflow <id>` must fetch the workflow and map `triggers[0]` → `trigger` (workflows store an array; templates store one object).

Zero new crates are required. The watch loop, the unauthenticated docs fetch, and the steps table all fit existing patterns (scripted TcpListener stub tests, `send_with_retry`, comfy-table Dynamic arrangement). Ctrl-C → exit 130 falls out of default SIGINT termination (tokio has no `signal` feature today; adding it is optional).

**Primary recommendation:** Build in three independent slices — runs (list/detail/watch sharing one `WorkflowRunsListParams` + `WorkflowRun`/`WorkflowRunStep` models), templates (top-level command group, cache-backed completions, delete via the workflows single-delete confirmation pattern), and docs (unauthenticated client method + `--save` file logic) — each with stub-server tests using a request-head-capturing variant of the existing stub helper.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Runs list + status/dry-run filtering | Server (query params) | CLI (flags → params, empty-result hints) | Server filters `status` and hides `dry_run` server-side; CLI only forwards flags and renders hints |
| Run detail steps flattening | CLI (presentation) | Server (returns steps) | Server returns the full steps array; flattening to rows is pure presentation |
| Watch polling | CLI (loop) | Server (stateless GETs) | No logs/stream endpoint exists (REQUIREMENTS Out-of-scope table); polling is the honest pattern |
| Template CRUD | Server | CLI (mapping workflow→template on create) | Server owns storage; CLI translates `--workflow <id>` into a fetch-and-map create payload |
| Template delete confirmation | CLI | Server (204/404) | Destructive-action UX is a CLI responsibility; server just deletes |
| OpenAPI docs fetch | Server (public route) | CLI (pretty-print, --save) | Server serves the spec unauthenticated; CLI formats/persists |
| Caching | CLI | — | Runs/docs never cached (ephemeral/large); templates list cacheable for completions |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap (derive) | 4.6 | New `runs` subcommand group, `templates` + `docs` commands | Already the CLI framework; hidden-flag + after_help patterns established [VERIFIED: Cargo.toml] |
| reqwest | 0.13 (json, rustls, query) | All HTTP incl. new unauthenticated docs path | Existing client; `default_headers` carries auth, so docs needs a bare client instance [VERIFIED: Cargo.toml] |
| tokio | 1 (rt, macros, time) | `time::sleep` for the 2s poll interval | `time` feature already enabled — no new feature needed for polling [VERIFIED: Cargo.toml] |
| serde/serde_json | 1.x | Run/step/template models (`serde_json::Value` for trigger/input/output) | Existing convention for complex workflow fields [VERIFIED: Cargo.toml] |
| comfy-table | 7.2 | Steps detail table (Dynamic arrangement truncates long cells) | Existing renderer base; `ContentArrangement::Dynamic` + fixed width when not a TTY already implemented [VERIFIED: src/output/table.rs] |
| chrono + chrono-humanize | 0.4 / 0.2 | Parse ISO timestamps, humanize step duration | Already used by output/formatters [VERIFIED: Cargo.toml, src/output/format.rs] |
| dialoguer | 0.12 | Templates delete confirmation | Same confirm flow as workflows delete [VERIFIED: src/commands/workflows/delete.rs] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| assert_cmd + predicates + tempfile | 2.x / 3.x / 3.x | Stub-server integration tests | All new test files [VERIFIED: Cargo.toml dev-dependencies] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Default SIGINT kill for Ctrl-C → 130 | `tokio::signal::ctrl_c()` graceful handler | Signal handler requires adding the `signal` feature to tokio; default SIGINT already yields shell exit code 130 with line-buffered stdout flushed. Only add the feature if graceful cleanup is wanted — not needed per locked decisions |
| Typed `WorkflowRunStep` struct | Raw `serde_json::Value` steps | Typed model gives field-name safety for the table renderer and unit-testable duration math; `input`/`output` stay `Value` (server stores arbitrary jsonb) |
| New dependency for polling/backoff | `tokio::time::sleep` fixed 2s | Locked decision: 2s fixed, no backoff, no flag |

**Installation:**
```bash
# No new crates. Everything below is already in Cargo.toml.
cargo build
```

**Version verification:** No registry lookups needed — this phase installs nothing new. All versions above were read from the project's `Cargo.toml` [VERIFIED: Cargo.toml].

## Package Legitimacy Audit

**No external packages are installed in this phase.** All functionality uses crates already pinned in `Cargo.toml` (verified by reading the file). The one candidate change — enabling tokio's `signal` feature — is a feature flag on an existing pinned crate, not a new package; the recommended design avoids even that by relying on default SIGINT termination. slopcheck run skipped per protocol (nothing to install).

## Server Contract (verified against server source)

All claims in this section were verified by reading the server's route handlers, schema, and serializers on 2026-09-03 [VERIFIED: /home/pedro/programming/pipelite source files, paths cited per line].

### 1. Runs — list and detail

**List:** `GET /api/v1/workflows/{id}/runs?offset&limit&status&dry_run=true`
- Source: `src/app/api/v1/workflows/[id]/runs/route.ts`
- Auth: `withApiAuth` (Bearer API key). Foreign or missing workflow → **404** (never 403 — deliberate anti-enumeration; byte-identical bodies) [VERIFIED: route.ts:52-56]
- `status` query param: forwarded into the WHERE clause via a TypeScript cast — **NOT validated**. An invalid status yields HTTP 200 with an empty `data` array (locked CONTEXT hint covers this) [VERIFIED: route.ts:66]
- `dry_run=true` (exact string) opts IN to test runs; **test runs are hidden by default server-side** (`eq(dryRun, false)` appended when the param is absent or not exactly `"true"`) [VERIFIED: route.ts:21-22, 67]
- Ordering: `created_at DESC`. Pagination: `offset` (default 0), `limit` (default 50, max 100) via shared `parsePagination` [VERIFIED: route.ts:76-78, src/lib/api/pagination.ts]
- Run row shape (from `serializeRun`, `src/lib/api/serialize.ts:207-221`):

```json
{
  "id": "uuid",
  "workflow_id": "uuid",
  "status": "pending|running|completed|failed|waiting",
  "trigger_data": { "envelope of the record that triggered the run": "..." },
  "error": "string or null",
  "depth": 0,
  "dry_run": false,
  "current_node_id": "string or null",
  "started_at": "ISO or null",
  "completed_at": "ISO or null",
  "created_at": "ISO"
}
```
- **Serializer exclusions** (DB columns that never appear on the wire): `context` (jsonb) and `replayed_from_run_id` [VERIFIED: schema/workflows.ts:48,57 vs serialize.ts:207-221]
- `depth` semantics: nested-workflow executions. When a workflow's CRM actions fire another workflow, the child run is a **separate row on the CHILD workflow** at `depth + 1`; runs at `depth >= MAX_RECURSION_DEPTH` are created immediately as `failed` [VERIFIED: src/lib/execution/engine.ts:107-124, src/lib/triggers/create-run.ts:34-35]. Consequence: `workflows runs list --workflow <parent>` never shows child runs — they live under the child workflow.

**Detail:** `GET /api/v1/workflows/{id}/runs/{runId}`
- Source: `src/app/api/v1/workflows/[id]/runs/[runId]/route.ts`
- **Requires BOTH ids in the path.** The workflow is resolved owner-scoped first (404 if not owner), then the run is matched with `workflow_id = id AND id = runId` (404 "Workflow run" if no match) [VERIFIED: route.ts:34-52]
- Response: single envelope `{data: { ...run, steps: [...] }}` — run fields as above plus `steps` ordered by `created_at ASC` [VERIFIED: route.ts:55-64]
- Step shape (from `serializeRunStep`, serialize.ts:226-240):

```json
{
  "id": "uuid",
  "run_id": "uuid",
  "node_id": "string",
  "status": "pending|running|completed|failed|skipped|waiting",
  "input": { "arbitrary jsonb": "or null" },
  "output": { "arbitrary jsonb": "or null" },
  "error": "string or null",
  "resume_at": "ISO or null",
  "started_at": "ISO or null",
  "completed_at": "ISO or null",
  "created_at": "ISO"
}
```
- Step status enum has **six** values — note `skipped`, absent from the run-level enum [VERIFIED: schema/workflows.ts:68]

### 2. Dry-run / test runs — the mechanism

- Test runs are created **only by server-internal paths** (`createWorkflowRun(..., {dryRun: true})` — the UI test-run action). The REST trigger `POST /api/v1/workflows/{id}/run` never creates them [VERIFIED: src/lib/triggers/create-run.ts:21,50; src/app/api/v1/workflows/[id]/run/route.ts has no dry_run param]
- `workflow_runs.dry_run` is `NOT NULL DEFAULT false`, so all pre-column runs are real runs [VERIFIED: schema/workflows.ts:50-53]
- **Filtering is server-side.** The CLI sends `dry_run=true` when `--include-dry-run` is passed and omits the param otherwise. No client-side filtering.
- **The "N test run(s) hidden" hint requires a probe:** the locked UX wants the hint only "when dry-run runs exist server-side", which the CLI can only learn by making a second request with `dry_run=true` and reading `meta.total`. Recommended: on an empty first-page result without the flag, issue exactly one probe request (`limit=1, dry_run=true`); if `meta.total > 0`, print the hint to stderr (suppressed under `--quiet`, consistent with the quiet convention for non-essential output).

### 3. Templates

Routes (source: `src/app/api/v1/workflow-templates/route.ts`, `[id]/route.ts`):
- `GET /api/v1/workflow-templates?offset&limit` — paginated list, `created_at DESC` [VERIFIED: route.ts:13-20; mutations listWorkflowTemplates]
- `POST /api/v1/workflow-templates` — 201 `{data: {...}}` via `createdResponse` [VERIFIED: route.ts:22-48]
- `GET /api/v1/workflow-templates/{id}` — single envelope; 404 "Workflow template" if missing [VERIFIED: [id]/route.ts:15-24]
- `DELETE /api/v1/workflow-templates/{id}` — **204 No Content**; 404 if missing [VERIFIED: [id]/route.ts:26-34]
- **NO update route exists** — confirmed: no PUT/PATCH handler in `[id]/route.ts` [VERIFIED]

Create schema (zod, `src/lib/mutations/workflow-templates.ts`):
```
name:        string, 1..200, required
description: string, ≤2000, optional/nullable
category:    string, ≤100, optional/nullable
trigger:     object (record), REQUIRED — no inner shape validation
nodes:       array of objects, optional, server defaults to []
```
Validation failure → 422 RFC 7807 with `errors[]` entries (flows through the Phase 8 parser untouched — the errors-array branch wins) [VERIFIED: route.ts:32-40].

Item shape (`serializeWorkflowTemplate`, serialize.ts:245-255):
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string or null",
  "category": "string or null",
  "trigger": { "object": "..." },
  "nodes": [ "array, default []" ],
  "created_at": "ISO"
}
```

Delete semantics: **hard delete** (`db.delete(...)`), **no soft-delete marker** (the table has no `deleted_at` column — schema/workflows.ts:95-108), no linked-object checks, no reference counting. The Phase 12 custom-field-definitions soft-delete concern does **not** apply to templates. Double-delete → second DELETE 404s. **No ownership scoping** — templates are global; any valid API key can read or delete any template (getWorkflowTemplate takes no userId) [VERIFIED: mutations/workflow-templates.ts].

Workflow→template mapping (for `templates create --workflow <id>`): `workflows.triggers` is an **array** of objects; `template.trigger` is a **single required object**. The canonical round-trip (how a template instantiates a workflow: `POST /workflows {name, triggers: [template.trigger], nodes: template.nodes}`) implies `template.trigger = workflow.triggers[0]`. There is no server-side instantiate endpoint [VERIFIED: absence of any non-CRUD route under workflow-templates/].

### 4. Docs

`GET /api/v1/docs` (source: `src/app/api/v1/docs/route.ts`):
- **No auth** — the handler has no `withApiAuth` wrapper at all; doc comment states it is deliberately public [VERIFIED: route.ts:7-14]
- Reads `public/openapi.yaml` (exists, 87 KB, `openapi: 3.1.0`), parses YAML→JSON, returns `Content-Type: application/json` with `Cache-Control: public, max-age=3600` and CORS `*` [VERIFIED: route.ts:16-27]
- Error case (spec file missing): **500 with legacy body `{"error": "OpenAPI specification not available"}`** — NOT RFC 7807. `parse_rfc7807` handles this via its legacy `error`-key fallback (step 4), rendering "OpenAPI specification not available" [VERIFIED: route.ts:28-34; src/api/mod.rs:75]
- 404 (older server without the route): standard Next.js 404 — the CONTEXT-mandated hint ("the server may not expose the docs endpoint — check server version") belongs here
- **Staleness note:** the served spec documents `/workflows/{id}/runs` and `/workflows/{id}/runs/{runId}` but does **NOT** document `/workflow-templates` or `/docs` itself [VERIFIED: grep of public/openapi.yaml]. Irrelevant to the CLI (raw passthrough) but worth knowing if anyone greps the spec for endpoint truth.

## How the Unauthenticated Client Path Works

`PipeliteClient::from_credentials` installs `AUTHORIZATION` in `default_headers`, which apply to **every** request from that client instance — there is no per-request way to remove a default header [VERIFIED: src/api/mod.rs:113-131]. Recommended approach for docs:

```rust
// Source: pattern derived from src/api/mod.rs client construction (same timeouts, no default headers)
pub async fn get_docs(&self) -> Result<serde_json::Value> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;
    let url = format!("{}/api/v1/docs", self.base_url);
    let response = client.get(&url).send().await.map_err(|e| self.map_request_error(e))?;
    // Reuse the Phase 8 status mapping via surface "docs" — but see Pitfall 5
    // for the hint override the CONTEXT locks for the docs surface.
    let spec: serde_json::Value = ...; // parse like handle_response success branch
    ...
}
```

As a method on `PipeliteClient` it reuses `self.base_url` and `map_request_error`; the throwaway client keeps the same 5s/30s timeouts. Asserting "no Authorization header" in tests is easy: the stub server captures the full request head (existing pattern reads it for Content-Length — extend the helper to also record it).

## Recommended Approach per Requirement

### WRUN-01 — Runs list
- `WorkflowsCommands::Runs(WorkflowsRunsArgs)` nested subcommand enum with `List` and a detail variant (see Open Question 1), matching the one-CLI-file/one-command-dir convention. New file `src/cli/workflows_runs.rs` (or a `runs` variant block inside `workflows.rs`) + `src/commands/workflows/runs/`.
- `WorkflowRunsListParams { workflow_id, status: Option<String>, include_dry_run: bool, limit, offset }` → query pairs `status`, `dry_run=true` (only when opted in), `limit`, `offset`.
- Empty result + `--status` set → stderr hint listing valid statuses `pending|running|completed|failed|waiting` (server does not validate). Empty result + no `--include-dry-run` → the probe-request hint from §2 above.
- Render via the standard `output::render_list`; `--fields` works free. Runs are **never cached**.

### WRUN-02 — Run detail
- Fetch run + steps via the detail endpoint; `--format json` passes the raw single envelope data through verbatim (FIX-02 passthrough convention).
- Table mode: bespoke steps table (comfy-table, `ContentArrangement::Dynamic`, fixed width 120 when not a TTY — same as `format_list`): columns `node` (`node_id`), `status`, `input`, `output`, `error`, `duration` (computed client-side from `started_at`/`completed_at` via chrono + chrono-humanize; empty while running). Render run-level summary fields (id, status, dry_run, depth, error, started/completed) as a compact header above the steps table.
- `input`/`output` render as compact JSON strings; Dynamic arrangement + the non-TTY width cap provide the locked truncation behavior.

### WRUN-03 — Watch
- Loop on the detail command: fetch → if `status ∈ {pending, running, waiting}` print one stderr transition line on change (`run <id>: X → Y`, suppressed by `--quiet`), `tokio::time::sleep(Duration::from_secs(2))`, repeat. On `completed | failed`: render like normal detail, exit per mapping — default 0 even on failed; `--exit-status` maps `failed` → 1.
- **Terminal states are exactly `{completed, failed}`.** There is **no `cancelled` status in the server enum** [VERIFIED: schema/workflows.ts:33]; the CONTEXT's "cancelled → 1" is satisfied vacuously — map it defensively anyway (`"cancelled" => 1` in the match) so a future server addition can't flip exit codes silently.
- `waiting` is NOT terminal — keep polling (steps' `resume_at` shows why it waits). A run stuck in `waiting` polls indefinitely; that is the locked no-timeout decision, but document it in after_help.
- Ctrl-C: rely on default SIGINT termination — the process dies mid-poll, the shell reports **130** (128+SIGINT=2), and Rust's line-buffered stdout has already flushed every prior render. Zero code, zero new features. (Alternative if graceful shutdown is ever wanted: add `signal` to tokio features and `tokio::signal::ctrl_c()` in a `select!` — not recommended now.)

### TPL-01 — Templates
- Top-level `Templates(TemplatesCommands)` in `Commands` enum; subcommands List / Get / Create / Delete; NO Update variant. A hidden `#[arg(long, hide = true)] update`-style parse-then-error is not needed for a missing *subcommand* — instead the after_help carries the locked sentence, and `templates update` naturally fails with clap's unrecognized-subcommand error. **Correction to the locked wording:** clap cannot emit the custom exit-2 InvalidInput for a nonexistent subcommand; to honor "pipelite templates update → exit 2 InvalidInput with that hint", add a hidden `Update` variant (hide = true, like `workflows create --active`) whose handler returns `CliError::InvalidInput` with the delete-and-recreate hint — the established Phase 8 hidden-flag pattern [VERIFIED: src/cli/workflows.rs:113-117, src/commands/workflows/create.rs:25-30].
- Create: `--name` + `--workflow <id>` (fetch workflow, map `triggers[0]` → `trigger`, `nodes` → `nodes`; stderr warning if the workflow has >1 trigger — only the first is captured), optional `--description`, `--category`, `--nodes <json>`, `--trigger <json>` (escape hatch when not snapshotting a workflow), `--stdin` raw-JSON body with full control (verbatim passthrough like other `--stdin` creates). Trigger must resolve from exactly one source → otherwise exit 2 InvalidInput.
- Delete: mirror `workflows` single-delete exactly — dry-run intercept first, TTY `dialoguer::Confirm`, non-interactive without `--force` → exit 2 with the `--force` hint [VERIFIED: src/commands/workflows/delete.rs:42-76]. Multiple IDs route through `batch::run_batch_delete` (free consistency with Phase 7). Cache invalidation: `KEY_TEMPLATES` on create/delete success.
- Completion candidates: template ids from cache (KEY_TEMPLATES) for get/delete; existing `workflow_id_candidates` for create's `--workflow`.

### DOCS-01 — docs command
- Top-level `Docs(DocsArgs)` with `--save FILE` + `--force`. Fetch via the unauthenticated client method above; pretty-print with `serde_json::to_string_pretty` to stdout; ignore `--format` (document in help).
- `--save`: create parent dirs (`fs::create_dir_all` on the parent), refuse to overwrite an existing file unless `--force` (exit 2 InvalidInput with the `--force` hint), print one-line confirmation (`Wrote OpenAPI spec to FILE (N bytes)` or similar), respecting `--quiet`.
- Errors: 404/500 on the docs surface get the locked hint. Because `handle_response` hardcodes generic 404 hints, catch the error in the command handler and re-wrap `CliError::NotFound`/`CliError::Api` preserving `detail` but substituting the CONTEXT hint (see Pitfall 5).

## Architecture Patterns

### System Architecture Diagram

```
                        pipelite CLI (Phase 9 surfaces)
                                      │
        ┌─────────────────────────────┼──────────────────────────────┐
        │                             │                              │
  workflows runs                templates                        docs
        │                             │                              │
  list / detail / watch       list/get/create/delete          GET (no auth header)
        │                             │                        │
        │  GET /api/v1/workflows/{wf}/runs[/{run}]            │  bare reqwest::Client
        │  ?status=&dry_run=true&offset&limit                 │  (no default_headers)
        │                             │                        │
        │                      POST/GET/DELETE                 │
        │                      /api/v1/workflow-templates      │
        │                             │                        │
        ▼                             ▼                        ▼
  ┌─────────────────────────────────────────────────────────────────┐
  │  Pipelite server (Next.js /api/v1)                              │
  │  withApiAuth (Bearer pk_live_...)   docs route: NO auth         │
  │  runs: owner-scoped 404s, server-side dry_run hiding            │
  │  templates: global, hard delete, zod create validation          │
  └─────────────────────────────────────────────────────────────────┘
```

Watch data flow: `GET detail` → status terminal? ──no──► stderr transition line ─► sleep 2s ─► (loop)
                                                │yes
                                                ▼
                                    render final detail; exit 0, or 1 with `--exit-status` + failed

### Recommended Project Structure
```
src/
├── cli/
│   ├── workflows.rs         # extend: Runs variant on WorkflowsCommands
│   ├── templates.rs         # NEW: TemplatesCommands enum + args
│   └── docs.rs              # NEW: DocsArgs
├── commands/
│   ├── workflows/
│   │   └── runs/            # NEW: list.rs, detail.rs (detail+watch), mod.rs
│   ├── templates/           # NEW: list.rs, get.rs, create.rs, delete.rs, mod.rs
│   └── docs.rs              # NEW
├── api/
│   ├── mod.rs               # add: list_workflow_runs, get_workflow_run, template CRUD, get_docs
│   └── models.rs            # add: WorkflowRun, WorkflowRunStep, WorkflowTemplate(+Create), params
└── cache.rs                 # add: KEY_TEMPLATES + TTL_TEMPLATES
tests/
├── workflow_runs_stub_test.rs   # NEW
├── templates_stub_test.rs       # NEW
└── docs_stub_test.rs            # NEW
```

### Pattern 1: Scripted stub server for polling tests
**What:** The existing `spawn_body_stub_server(script: &[(u16, &str)])` serves one scripted response per connection, in order [VERIFIED: tests/error_layer_stub_test.rs:54-127]. A watch test scripts `[(200, running), (200, running), (200, completed)]` and the CLI's successive polls consume them naturally.
**When to use:** WRUN-03 state-transition and exit-code tests; WRUN-01/02 and TPL-01/DOCS-01 response rendering tests.
**Extension needed:** a variant that also records each request head (request line + headers) so tests can assert `?status=failed&dry_run=true` query strings and the **absence** of `Authorization:` on docs requests. The current helper reads the head but discards it.

### Pattern 2: Hidden-flag parse-then-error
**What:** `#[arg(long, hide = true)]` + handler rejection before any HTTP, exit 2 InvalidInput with an actionable hint [VERIFIED: src/cli/workflows.rs:113-117].
**When to use:** `templates update` (hidden variant) per the locked after_help + exit-2 decision.

### Pattern 3: Single-delete confirmation flow
**What:** dry-run intercept → TTY dialoguer Confirm → non-interactive refusal exit 2 with `--force` hint [VERIFIED: src/commands/workflows/delete.rs:42-76].
**When to use:** `templates delete`.

### Anti-Patterns to Avoid
- **Client-side dry-run filtering:** never fetch all runs and filter `dry_run` in the CLI — the server already hides them; sending `dry_run=true` unconditionally would silently include test runs everywhere.
- **Caching runs:** run state changes between polls; any cache read would make `--watch` show stale states.
- **Validating `--status` client-side with a rejection:** the locked decision is pass-through + empty-result hint, not a client-side enum rejection.
- **`Value::to_string()` on error/detail strings:** JSON-quotes them (Pitfall 2 in Phase 8 research; `parse_rfc7807` already handles this).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| 429 handling on new endpoints | Custom retry logic | `send_with_retry` (existing) | Retry-After clamping + single retry already tested [VERIFIED: src/api/mod.rs:206-232] |
| Error status mapping | Per-command match on status codes | `handle_response` / `handle_delete_response` | RFC 7807 parsing + hint table centralised |
| Truncating long step cells | Manual string cutting | comfy-table `ContentArrangement::Dynamic` + width cap | Already implemented and TTY-aware [VERIFIED: src/output/table.rs:46-52] |
| Humanized durations | Custom relative-time math | `chrono-humanize::HumanTime` | Already a dependency and used in formatters |
| Confirmation prompts | Raw stdin y/n | `dialoguer::Confirm` + existing flow | TTY/non-TTY semantics already worked out |
| ID shell completion | Manual cache reads in handlers | `ArgValueCandidates` + cache candidates helpers | Pattern exists for workflow ids |

**Key insight:** this phase is almost entirely composition of existing utilities; the only genuinely new mechanics are the poll loop and the headerless HTTP client.

## Runtime State Inventory

**Skipped — not a rename/refactor/migration phase.** No stored data, live service config, OS registrations, secret keys, or build artifacts carry names this phase changes. (New cache files `templates.json` are created, none renamed.)

## Common Pitfalls

### Pitfall 1: Detail command missing the workflow id
**What goes wrong:** The locked UX reads `pipelite workflows runs <id>`, inviting a run-id-only detail command. The server requires the workflow id in the path and offers no run→workflow lookup (audit's `workflow_run_id` filter is admin-only and Phase 11).
**Why it happens:** The CONTEXT wording was written before the double-id path requirement was re-verified.
**How to avoid:** Detail/watch carry a required `--workflow` flag (same flag name as list). Flag this to discuss/planner — see Open Question 1.
**Warning signs:** Any plan task that calls `GET /workflows/{runId}/runs/...` or invents a lookup endpoint.

### Pitfall 2: Multi-trigger workflows silently truncated on template create
**What goes wrong:** `templates create --workflow <id>` maps `triggers[0]` → `trigger`; workflows with 2+ triggers lose triggers silently.
**How to avoid:** Warn on stderr when `triggers.len() > 1`; `--stdin` is the documented full-control path.
**Warning signs:** Template instantiated later produces a workflow with fewer triggers than the source.

### Pitfall 3: Treating `waiting` as terminal
**What goes wrong:** Watch exits on `waiting`, or `--exit-status` maps it to failure; runs waiting on a `resume_at` are mid-flight.
**How to avoid:** Terminal set is exactly `{completed, failed}`; document `waiting` semantics in help.
**Warning signs:** Watch tests that script a `waiting` response must expect continued polling, not exit.

### Pitfall 4: Sending the auth header on docs / asserting the wrong thing
**What goes wrong:** Reusing `self.client` for docs leaks the API key to a (public) endpoint — harmless today but violates the locked decision and sets a precedent; conversely, tests that only assert 200 miss the actual contract.
**How to avoid:** Dedicated headerless client instance; stub test asserts the request head contains NO `authorization:` line.
**Warning signs:** `get_docs` implemented as a method that calls `self.client.get(...)`.

### Pitfall 5: Docs error hints swallowed by generic 404 mapping
**What goes wrong:** `handle_response`'s 404 arm hardcodes "The requested resource was not found." — the locked docs hint never appears.
**How to avoid:** In the docs command, match the error and re-wrap NotFound/Api preserving `detail`, substituting the CONTEXT hint. (Alternatively add a docs-specific branch, but the command-level re-wrap keeps the shared mapper untouched.)
**Warning signs:** Stub test with a 404 docs response asserting the generic hint instead of the docs hint.

### Pitfall 6: The hidden-runs probe firing on every list
**What goes wrong:** Naively probing `dry_run=true` on every empty list doubles request volume; probing on non-empty lists is pure waste.
**How to avoid:** Probe only when: first page empty AND `--include-dry-run` absent AND `--status` absent (a status-filtered empty result says nothing about test runs of that status — probe with the same status or skip; recommend probing with the same `status` so the hint stays truthful).
**Warning signs:** N+1 request patterns in logs; hint appearing when test runs exist but the filter itself caused the empty page.

### Pitfall 7: `templates delete` on someone else's template
**What goes wrong:** Templates are global — no ownership check server-side; a valid key deletes any template, affecting other users.
**How to avoid:** Nothing to fix server-side (out of CLI scope), but the delete prompt should state the template name, and docs/help can note templates are shared across the deployment.
**Warning signs:** Assumptions in plan tasks that 403 is possible on template delete (it effectively isn't; only 404).

## Code Examples

### Run detail table rendering (client-side duration)
```rust
// Source: pattern from src/output/table.rs + chrono-humanize usage in src/output/format.rs
fn step_duration(started_at: &Option<String>, completed_at: &Option<String>) -> String {
    match (started_at, completed_at) {
        (Some(s), Some(c)) => {
            let (s, c) = (parse_iso(s), parse_iso(c)); // chrono::DateTime<Utc>
            HumanTime::from(c - s).to_string()          // e.g. "3 seconds ago" style — trim to "3s" if desired
        }
        _ => String::new(), // pending/running steps have no duration yet
    }
}
```

### Watch loop skeleton
```rust
// Source: locked CONTEXT semantics + tokio::time (feature already enabled)
let mut last_status = String::new();
loop {
    let run = ctx.client.get_workflow_run(wf_id, run_id).await?; // detail incl. steps
    if run.status != last_status && !ctx.quiet && !last_status.is_empty() {
        eprintln!("run {run_id}: {last_status} → {}", run.status);
    }
    last_status = run.status.clone();
    if matches!(run.status.as_str(), "completed" | "failed") {
        render_like_detail(&run)?;                                  // final render
        let code = if args.exit_status && run.status == "failed" { 1 } else { 0 };
        std::process::exit(code);
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
    // Ctrl-C: default SIGINT kills the process → shell reports 130. No handler needed.
}
```

### Templates create via --workflow (mapping step)
```rust
// Source: verified shapes — workflows.triggers: Vec<Value> (schema/workflows.ts:12),
// template.trigger: single required object (create schema)
let wf = ctx.client.get_workflow(&args.workflow, None).await?; // reuse existing method
let trigger = wf.triggers.as_ref().and_then(|t| t.first()).cloned().ok_or_else(|| CliError::InvalidInput {
    detail: format!("Workflow {} has no triggers to snapshot", args.workflow),
    hint: "Add a trigger to the workflow first, or create the template with --stdin.".to_string(),
})?;
if wf.triggers.as_ref().map_or(false, |t| t.len() > 1) && !ctx.quiet {
    eprintln!("warning: workflow has {} triggers; only the first was captured", wf.triggers.as_ref().unwrap().len());
}
let body = WorkflowTemplateCreate {
    name: args.name.clone().expect("clap-required"),
    description: args.description.clone(),
    category: args.category.clone(),
    trigger,
    nodes: wf.nodes.clone().unwrap_or_default(),
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Runs list with no test-run hiding | Server-side `dry_run` hiding + `?dry_run=true` opt-in | Server WR-02 (post-Phase-50 source) | CLI must NOT filter client-side; the flag only toggles the query param |
| Any-key-can-read-any-workflow's-runs | Owner-scoped 404 on both runs routes | SEC-48-A (server comments) | Foreign workflow → 404 like everything else under workflows/; no new error branch needed |
| OpenAPI spec as static docs | Served live at `/api/v1/docs`, CORS `*`, no auth | Server docs route | `pipelite docs` is a plain GET; spec itself omits templates/docs paths (server-side staleness, not CLI's problem) |

**Deprecated/outdated:**
- `SERVER-API-DIFF.md` §5's "`--dry-run` opt-in" flag name: superseded by the locked `--include-dry-run` (CLI-side naming only; the wire param stays `dry_run=true`).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Default SIGINT termination yields shell-visible exit 130 for the watch command (no handler installed) | Watch / Recommended Approach | Low: POSIX-standard 128+N; if any wrapper eats SIGINT, fallback is adding tokio `signal` feature |
| A2 | Template instantiation round-trip is client-side (`triggers: [template.trigger]`), justifying `triggers[0]` → `trigger` mapping | Templates / Pitfall 2 | Medium: if the server later adds an instantiate endpoint with different semantics, the mapping still round-trips validly (same object shape) |
| A3 | `meta.total` from a `dry_run=true` probe reliably reports hidden test-run count | WRUN-01 hint | Low: total is computed with the same WHERE clause server-side [VERIFIED: runs route.ts:80-82] |
| A4 | Templates list TTL of ~1h (mirroring `TTL_WORKFLOWS`) is acceptable for completions | Cache guidance | Low: stale completion candidates degrade gracefully (they are only suggestions) |

## Open Questions

1. **Detail/watch invocation shape (needs planner confirmation against CONTEXT wording)**
   - What we know: The locked wording says "detail via `pipelite workflows runs <id>`" but the server path needs the workflow id too, and no run→workflow resolution exists.
   - What's unclear: Whether `<id>` was meant as run id (with `--workflow` flag) or the wording simply elided the second id.
   - Recommendation: `pipelite workflows runs <run_id> --workflow <wf_id> [--watch] [--exit-status]` — required `--workflow` flag, consistent with list's flag name. Low-cost to change later if discuss-phase overrides.

2. **Should the hidden-runs hint probe carry the user's `--status` filter?**
   - What we know: Probing without the status could report hidden test runs that the filtered view would never show, making the hint misleading.
   - Recommendation: Probe with the same `status` param; hint says "N test run(s) hidden — pass --include-dry-run".

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo | Build + tests | ✓ | 1.94.0 | — |
| Existing test suite compiles | Baseline | ✓ | `cargo test --no-run` passes (1 pre-existing dead-code warning: `get_workflows_cached`) | — |
| Live Pipelite server | Manual UAT only | Available at `/home/pedro/programming/pipelite` (docker-compose.yml present) | — | Stub-server tests cover CI; live UAT needs a running server with an active workflow to observe runs |
| tokio `signal` feature | Only if graceful Ctrl-C chosen | ✗ (not enabled) | — | Default SIGINT behavior (recommended path needs nothing) |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** none blocking.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` + `assert_cmd` CLI integration tests (existing) |
| Config file | none needed (tests/ convention + `Command::cargo_bin`) |
| Quick run command | `cargo test --quiet workflow_runs_stub templates_stub docs_stub` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WRUN-01 | Runs list renders rows; `--status` and `--include-dry-run` produce correct query params (stub asserts request line) | integration (stub) | `cargo test --quiet --test workflow_runs_stub` | ❌ Wave 0 |
| WRUN-01 | Empty + `--status` → valid-statuses hint; empty without opt-in + dry runs exist → hidden-runs hint (stub scripts probe sequence) | integration (stub) | `cargo test --quiet --test workflow_runs_stub hidden` | ❌ Wave 0 |
| WRUN-02 | Detail flattens steps into rows with node/status/error/duration; `--format json` passes run verbatim | integration (stub) + unit (duration math) | `cargo test --quiet --test workflow_runs_stub detail` | ❌ Wave 0 |
| WRUN-03 | Watch polls scripted running→running→completed; exit 0; `--exit-status` + failed → exit 1; transition lines on stderr; `--quiet` suppresses | integration (stub, scripted multi-response) | `cargo test --quiet --test workflow_runs_stub watch` | ❌ Wave 0 |
| WRUN-03 | Ctrl-C → exit 130 | integration (Unix-only, `#[cfg(unix)]:` spawn child, send SIGINT) or manual UAT | `cargo test --quiet --test workflow_runs_stub sigint` | ❌ Wave 0 (manual UAT fallback acceptable) |
| TPL-01 | list/get render; create posts mapped payload (+ warning on multi-trigger); create 422 passthrough; delete confirms, `--force` bypasses, non-interactive without `--force` → exit 2 zero HTTP | integration (stub) | `cargo test --quiet --test templates_stub` | ❌ Wave 0 |
| TPL-01 | `pipelite templates update` → exit 2 with delete-and-recreate hint | integration (no HTTP) | `cargo test --quiet --test templates_stub update` | ❌ Wave 0 |
| DOCS-01 | Docs GET sends NO Authorization header (stub asserts request head); pretty JSON on stdout; `--save` writes + creates parent dirs; refuses overwrite without `--force`; 404 → docs-specific hint | integration (stub) | `cargo test --quiet --test docs_stub` | ❌ Wave 0 |
| all | after_help examples in help output stay truthful | integration | extend existing `help_examples_test.rs` | ✅ (extend) |

### Sampling Rate
- **Per task commit:** `cargo test --quiet` (targeted new test files)
- **Per wave merge:** `cargo test` (full suite)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/workflow_runs_stub_test.rs` — WRUN-01/02/03 (needs the request-head-capturing stub helper variant)
- [ ] `tests/templates_stub_test.rs` — TPL-01
- [ ] `tests/docs_stub_test.rs` — DOCS-01
- [ ] Stub helper extension (record request heads) — shared, put in a `tests/fixtures/` module or duplicate per established convention

*(Existing infrastructure otherwise sufficient: no framework install needed.)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes (existing) | Bearer API key via `default_headers` on the shared client; docs deliberately bypasses it (public endpoint, locked decision) |
| V3 Session Management | no | API-key CLI, no sessions |
| V4 Access Control | yes (server-side) | Runs are owner-scoped server-side (404 anti-enumeration); CLI renders 404 verbatim. Templates are global by server design (no 403 possible) |
| V5 Input Validation | yes | clap flag typing + server zod/422 passthrough via `parse_rfc7807`; no client-side enum rejection per locked decision |
| V6 Cryptography | no | Nothing new to hash/encrypt; signing secrets are Phase 11 (webhooks) |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API key leaked to public endpoint | Information Disclosure | Headerless client for docs + stub test asserting no `authorization:` header |
| Test-run failures misread as production failures | Repudiation (ops) | Server-side hiding by default; CLI opt-in flag naming (`--include-dry-run`) keeps intent explicit |
| `trigger_data`/step `input`/`output` contain CRM record payloads on screen | Information Disclosure | Table-mode truncation + `--quiet`/piped-output conventions; JSON passthrough only on explicit `--format json` (same posture as all entities) |
| Template deleted affecting other users (global resource) | Tampering | Confirmation prompt shows template name; help notes templates are deployment-global |

## Sources

### Primary (HIGH confidence)
- `/home/pedro/programming/pipelite/src/app/api/v1/workflows/[id]/runs/route.ts` — list contract, dry_run hiding, 404 scoping
- `/home/pedro/programming/pipelite/src/app/api/v1/workflows/[id]/runs/[runId]/route.ts` — detail contract, steps inclusion
- `/home/pedro/programming/pipelite/src/app/api/v1/workflow-templates/route.ts` + `[id]/route.ts` — templates CRUD, no-update confirmation
- `/home/pedro/programming/pipelite/src/lib/mutations/workflow-templates.ts` — create schema, hard delete
- `/home/pedro/programming/pipelite/src/app/api/v1/docs/route.ts` + `public/openapi.yaml` — docs contract, no-auth, content-type
- `/home/pedro/programming/pipelite/src/db/schema/workflows.ts` — status enums, dry_run NOT NULL, depth, template table
- `/home/pedro/programming/pipelite/src/lib/api/serialize.ts` — exact wire field names
- `/home/pedro/programming/pipelite/src/lib/triggers/create-run.ts`, `src/lib/execution/engine.ts` — dry-run + depth provenance

### Secondary (HIGH — local codebase)
- `pipelite-cli`: src/api/mod.rs, src/api/models.rs, src/cli/*.rs, src/commands/workflows/*.rs, src/batch.rs, src/cache.rs, src/output/*.rs, src/error.rs, src/context.rs, Cargo.toml, tests/error_layer_stub_test.rs, docs/SKILL.md

### Tertiary
- `.planning/research/SERVER-API-DIFF.md` — cross-checked; accurate on all Phase 9 items; flag name superseded by CONTEXT decision; missed serializer exclusions + spec staleness (documented above)

## Metadata

**Confidence breakdown:**
- Server contract: HIGH — read directly from server source with file:line citations
- CLI integration patterns: HIGH — read from current post-Phase-8 source
- Watch/SIGINT behavior: HIGH on contract, MEDIUM-HIGH on exit-130 mechanism (A1: standard POSIX behavior, not empirically tested in this session)
- Template create mapping: MEDIUM-HIGH (A2: round-trip reasoning, no server instantiate endpoint exists to contradict)

**Research date:** 2026-09-03
**Valid until:** 2026-10-03 (server source verified today; stable milestone-scoped contract)
