# Stack Research — Milestone v1.1 "Server v2 Parity" Additions

**Domain:** Rust CLI additions to an existing 7-entity CRUD tool (notes, workflow runs, webhooks, trash, custom fields, templates, audit, docs, batch ops)
**Researched:** 2026-09-02
**Confidence:** HIGH

## Headline Verdict

**Zero new crates required.** All 10 milestone features are fully covered by the existing dependency set (clap 4.6, reqwest 0.13, serde/serde_json, dialoguer 0.12, comfy-table 7.2, indicatif, chrono, csv, colored). The four specific concerns raised (multipart upload, HMAC display, nested-table rendering, streaming stdin) all resolve to "existing stack, no addition" — details below.

**The only Cargo.toml change recommended:** add the `"time"` feature to tokio (for `tokio::time::sleep` on 429 `Retry-After` backoff during batches). This is a feature flag, not a new dependency.

```toml
tokio = { version = "1", features = ["rt", "macros", "time"] }
```

---

## Recommended Stack

### Core Technologies (existing — validated, unchanged)

| Technology | Version | Purpose | Why It Already Covers the New Features |
|------------|---------|---------|----------------------------------------|
| clap | 4.6 | Arg parsing | Subcommand nesting (`workflows runs`, `notes list/add/...`), multi-value args (`deals update id1 id2 ...`), `--stdin` flags — all native clap derive. No new args crate. |
| reqwest | 0.13 | HTTP client | `PUT` (webhooks use PUT, not PATCH), arbitrary query params (audit/trash/runs filters — `"query"` feature already enabled), 204 handling (trash restore), nested paths (`/deals/{id}/notes`). `PipeliteClient` just grows more typed methods following the existing pattern (src/api/mod.rs). |
| serde + serde_json | 1.0 | Models + JSON | Typed models per new entity; `serde_json::Value` passthrough for `steps[].input/output`, audit `changes{}`, template `trigger/nodes` (existing project key decision: "server validates complex types, CLI stays flexible"). Includes **streaming deserialization** for batch stdin (see below). |
| comfy-table | 7.2 | Tables | `ContentArrangement::Dynamic` + `ColumnConstraint` (already used in src/output/table.rs) handles nested data by **flattening to rows** — run steps become one row per step; audit `changes` becomes field/from/to rows. Renderers already operate on `&[serde_json::Value]` + column projection, so no renderer architecture change. |
| dialoguer | 0.12 | Prompts | `Confirm` for trash purge, batch destructive ops, and webhook secret show-once acknowledgment. 0.12.0 is the current max stable version. |
| indicatif | 0.18 | Progress | Batch progress bar (0.18.6 current). Draw around the sequential await loop. |
| chrono | 0.4 | Dates | `NaiveDate::parse_from_str` for custom-field `date` type parsing; `DateTime` formatting for audit/run timestamps (already renders via chrono-humanize). 0.4.45 current. |
| colored | 3.1 | Output | Webhook secret warning styling, per-item batch pass/fail coloring. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| *(none — nothing to add)* | — | — | — |

### Development Tools (existing, unchanged)

| Tool | Purpose | Notes |
|------|---------|-------|
| assert_cmd / predicates / tempfile | Integration tests | Existing batch/notes/trash tests follow the `127.0.0.1:1` + `fake-test-key` fake-credential convention |

---

## Feature-by-Feature Stack Mapping

This is the section the REQUIREMENTS.md and phase plans should consume.

### 1. Batch update/delete (continue-on-error)
- **Parsing stdin — no streaming crate needed.** serde_json has both modes natively:
  - NDJSON (one JSON object per line): `std::io::BufRead::lines` + `serde_json::from_str` per line — natural fit for `... | pipelite deals update --stdin`, per-line errors carry line numbers.
  - Single JSON array streamed without full buffering: `serde_json::Deserializer::from_reader(stdin).into_iter::<serde_json::Value>()` (verified: serde.rs documents incremental array deserialization without buffering the whole sequence). Sniff first non-whitespace byte (`[` → array mode, else NDJSON) to accept both.
- **Execution model: sequential loop, deliberately NOT concurrent.** Server rate limit is 500 req/60s; batches are tens-to-hundreds of items; sequential gives deterministic output order, trivial continue-on-error (`Vec<Result>` collected per item), and simple resume semantics. **Do not add `futures`/`futures-util` for `buffer_unordered`** — if concurrency is ever proven necessary, `tokio::task::JoinSet` is already available via the existing tokio dep.
- **429 handling:** honor `Retry-After` with `tokio::time::sleep` → the one Cargo.toml change (add `"time"` feature). Without it you'd need `std::thread::sleep` inside async, which blocks the `current_thread` runtime — works but is poor practice.
- **Results rendering:** per-item (id, status, error/hint) table via existing comfy-table renderer; summary line via existing pagination-footer pattern. Progress via indicatif.

### 2. Notes (sub-resource CRUD)
Pure existing pattern. Nested list/create route `{entity}/{id}/notes` + flat `PATCH/DELETE /notes/{noteId}`. Typed `Note` model; note that list-before-edit is required by API shape (no GET on single note — PATCH body is `{content}` only). The entity restriction (deals/orgs/people/activities only — not pipelines/stages/workflows) is CLI surface design, not a stack question.

### 3. Workflow runs (list + detail with steps)
Existing client + models. `steps[]` on detail: flatten to one comfy-table row per step (`node_id`, `status`, `started_at`, truncated `error`). `input`/`output` blobs stay `serde_json::Value` — full fidelity in JSON output format, truncated in table/plain. `--dry-run` filter and `status` filter are plain query params (`"query"` feature enabled).

### 4. Webhooks CRUD (HMAC secret display)
- **No `hmac`/`sha2` crates.** The secret is generated server-side and returned **only** in the create response; the CLI's job is *display once* — read field, print with colored warning + dialoguer `Confirm` ("I've stored this secret"), refuse to re-show. Zero crypto client-side.
- Event-name validation (13 names): `const` array + membership check — no `regex`, no `strum`, no enum-with-iterator crate needed.
- URL validation: `value.starts_with("https://")` matches the server's own check. **No `url` crate** — the server is the source of truth; client check is just fast feedback.
- Update uses `PUT` — `reqwest::Client::put()` already available; the client currently only exposes typed wrappers, so add `put` alongside `patch` in the generic helper layer if one exists.

### 5. Trash (list / restore / purge)
Standard CRUD variants: list with `?type=` (plural tabs — deals/people/organizations/activities), restore = `POST .../restore` (empty body, 204), purge = `DELETE`. Admin-only purge → member keys get 403: surface via existing `CliError` detail+hint convention ("purge requires an admin API key — use `trash list` + restore instead"). Confirmation gate via dialoguer `Confirm` (skippable under `--no-input` only with an explicit `--yes`-style flag — flag design is a requirements question).

### 6. Custom field definitions CRUD + type-aware value parsing
- Definitions CRUD: standard pattern; `PUT` for updates; `position` is **float** server-side → model as `f64` (mirrors the §D fix for `Deal.position` — fix both in one pass).
- **Type-aware parsing: a pure function, not a crate.** `parse_typed_value(field_type: &str, config: &Value, input: &str) -> Result<serde_json::Value>`:
  - `number` → `input.parse::<f64>()` → `serde_json::Number`
  - `boolean` → match "true"/"false"
  - `date` → `chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d")`
  - `single_select` → validate against `config.options[]`
  - `multi_select` → split on `,` → JSON array of strings (validate each against options)
  - `url` → prefix check
  - `lookup`/`file`/`formula` → reject with hint (file upload is session-only on the server; formula is server-computed/stripped)
- Escape hatch: `--custom-field-json` accepting a raw JSON object fragment, parsed with `serde_json::from_str`, merged like the server merges (precedent: `update_activity_raw` in src/api/mod.rs already sends raw `serde_json::Value` — extend that pattern).

### 7. Workflow templates
CRUD minus update. `trigger`/`nodes` as `serde_json::Value` — matches the recorded v1.0 key decision for workflow triggers. `@filepath` JSON input pattern already exists in `pipelite workflows trigger` — reuse for `templates create @file.json`.

### 8. Audit log viewer
List with filters (`entity_type`, `entity_id`, `actor_kind`, `workflow_run_id`) — plain query params. Dynamic `changes` object (field → `{from,to}`): iterate keys and project to field/from/to rows in the table renderer; untouched in JSON output. Admin-only → same clear-403 hint treatment as trash purge. No new viewer crate; explicitly **no TUI** (Out of Scope per PROJECT.md).

### 9. Docs fetch
`GET /api/v1/docs` returns OpenAPI 3.1 JSON. Default: pretty-print via `serde_json::to_string_pretty` (pager-friendly); `--output <file>` via `std::fs::write`. **No OpenAPI-parsing crate** — the CLI is a fetch-and-save tool here, not a spec processor.

### 10. Fixes (dead flags, expand passthrough, position float, stages all-mode)
Serde/model work only: `Option<f64>` for positions, `#[serde(flatten)]` extra-args map (or `Value` passthrough) for `--expand`. No stack changes.

---

## Installation

```bash
# No new crates. The single Cargo.toml change:
# tokio = { version = "1", features = ["rt", "macros", "time"] }
cargo build
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Sequential batch loop | `futures` + `buffer_unordered` concurrency | Only if real-world batches exceed ~200 items AND users complain about wall-clock time; even then `tokio::task::JoinSet` (already available) beats adding `futures-util` |
| serde_json Deserializer stream (array) + BufRead lines (NDJSON) | `serde_jsonlines` or `jsonl` crates | Never — these wrap 20 lines of std code in a dependency |
| `starts_with("https://")` URL check | `url` crate | If requirements later demand full URL normalization/validation client-side; server rejects anything else anyway |
| comfy-table 7.2 (pinned) | comfy-table 8.0 (released 2026-08-05) | Post-milestone upgrade pass — 8.0 is a major version with unverified breaking changes, and nothing in this milestone needs it |
| Show-once secret print | `hmac` + `sha2` signature helper (`webhooks test` subcommand computing a sample `X-Signature`) | Only as a future differentiator feature; genuinely useful but not in this milestone's scope |
| `chrono::NaiveDate` for custom-field dates | `time` crate | Never for this — chrono is already the date dep; mixing two date crates is pure cost |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| reqwest `multipart` feature | File upload routes are **session-cookie-only** (SERVER-API-DIFF §B) — not API-key usable, explicitly out of milestone scope | Nothing; revisit only if the server adds API-key upload |
| `hmac` / `sha2` crates | Secret handling is display-only; server does all signing/verification | Plain print + dialoguer confirm + colored warning |
| `futures` / `futures-util` | Sequential batch is the right model (rate limits, ordered output, simple error reporting) | Existing tokio (`JoinSet`) if concurrency ever needed |
| `url` crate | One prefix check; server validates for real | `str::starts_with` |
| OpenAPI/petstore parser crates | `pipelite docs` fetches and saves the spec; it doesn't interpret it | `serde_json::to_string_pretty` + `std::fs::write` |
| `uuid` crate | Server generates all IDs; CLI passes strings through | `String`/`Option<String>` in models |
| `regex` | Event names = const list; URL = prefix check; nothing else needs pattern matching | `const &[&str]` + `contains` |
| ratatui / crossterm (TUI) | PROJECT.md Out of Scope: "GUI or TUI framework — this is a CLI tool" | comfy-table + existing plain/json/csv renderers |
| comfy-table 8.0 mid-milestone | Major-version churn with zero feature payoff for this scope | Stay on 7.2; schedule an upgrade pass after the milestone |

## Stack Patterns by Variant

**If batch stdin payload is NDJSON (recommended default for agents):**
- Read `std::io::stdin().lock().lines()`, `serde_json::from_str::<Value>` per line, report failures with line numbers
- Because it degrades gracefully (process lines before a syntax error), and matches headless/agent piping conventions

**If batch stdin payload is a single JSON array:**
- `serde_json::Deserializer::from_reader(BufReader::new(stdin)).into_iter::<Value>()` — constant-memory streaming, no full-buffer `read_to_string`
- Because some tools (jq) naturally emit arrays

**If a custom-field value must be an arbitrary/complex type:**
- `--custom-field-json` raw-JSON escape hatch, following `update_activity_raw` precedent
- Because type-aware string parsing can't express nested objects (lookup values, etc.) and hardcoding every shape is a maintenance trap

**If 429 hits mid-batch:**
- Read `Retry-After` header, `tokio::time::sleep`, retry the same item once, else record failure and continue (continue-on-error covers rate limits too)
- Requires the tokio `"time"` feature flag

## Version Compatibility

| Package | Current In Project | Latest Stable (2026-09) | Action |
|---------|-------------------|------------------------|--------|
| reqwest | "0.13" | 0.13.4 | None — semver-compatible, already current |
| comfy-table | "7.2" | 8.0.0 (2026-08-05) | **Stay on 7.x this milestone** — 8.0 is a major bump released after v1.0 shipped; nothing in scope needs it |
| dialoguer | "0.12" | 0.12.0 | None — already at max stable |
| chrono | "0.4" | 0.4.45 | None |
| indicatif | "0.18" | 0.18.6 | None |
| csv | "1.3" | 1.4.0 | None required; 1.4 is a minor bump, fine to pick up opportunistically |
| serde_json | "1.0" | 1.0.151 | None |
| tokio | "1" (rt, macros) | — | **Add `"time"` feature** — the only Cargo.toml edit this milestone |

## Sources

- `.planning/research/SERVER-API-DIFF.md` — API surface (auth model, PUT-vs-PATCH, secret-only-on-create, session-only upload routes, rate limit 500/60s) — HIGH
- `Cargo.toml` + `src/api/mod.rs` + `src/output/table.rs` — existing stack pins, client method pattern, renderer `Value`-row architecture, `update_activity_raw` raw-JSON precedent — HIGH
- crates.io API (fetched 2026-09-02): serde_json 1.0.151, comfy-table 8.0.0, reqwest 0.13.4, dialoguer 0.12.0, chrono 0.4.45, indicatif 0.18.6, csv 1.4.0 — HIGH
- serde.rs "stream-array" (via Context7 `/websites/serde_rs`) — incremental deserialization of JSON sequences without buffering — HIGH
- comfy-table docs (via Context7 `/websites/rs_comfy-table`) — `ContentArrangement::Dynamic`, `ColumnConstraint`, multi-line cells — HIGH

---
*Stack research for: Pipelite CLI v1.1 milestone additions*
*Researched: 2026-09-02*
