# Phase 8: Foundations — Error Layer, Models & Pagination - Research

**Researched:** 2026-09-03
**Domain:** Rust CLI error handling (RFC 7807), serde model fidelity, clap flag lifecycle, pagination UX
**Confidence:** HIGH (server behavior verified directly against server source at `/home/pedro/programming/pipelite`)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Forbidden is a separate CliError variant** from Auth — 401 (bad key) ≠ 403 (insufficient permission); enables per-surface hints and distinct messaging.
- **Central `parse_rfc7807` helper** in `api/mod.rs`, called by `handle_response`/`handle_delete_response` — one place, benefits all surfaces. Parses `detail` and `errors[]` keys.
- **Per-surface Forbidden hints keyed by surface param** on API calls (e.g. audit → "audit log requires an admin key", trash purge → admin, notes → author-or-admin, webhooks → foreign webhook 403).
- **409 inactive-trigger hint**: "Workflow trigger is inactive — activate it before firing (`pipelite workflows update <id> --active true`)".
- **`Option<f64>` / `f64`** for `Deal.position`, `Stage.position`, and other position fields — server emits fractional values (FIX-03 root cause).
- **Typed models gain `Option<serde_json::Value>` expand field** (`skip_serializing_if` none) so output renders `--expand` payloads; request-only models untouched (FIX-02).
- **No client-side validation of `--expand` values** — pass through, server validates (v1.0 behavior).
- **`stages list` without `--pipeline`**: single unfiltered call to the stages endpoint; add `pipeline_id` to default columns so rows are distinguishable (FIX-04).
- **Parse-then-error for removed flags** (`people list --org/--owner`, `workflows create --active`, dead `--custom-field` on pipelines/stages): keep flags defined (hidden), command layer rejects with `CliError::InvalidInput` (exit 2) + hint naming the replacement.
- **`workflows list --active`**: client-side filter retained; one stderr warning line per invocation: `warning: --active filters client-side after fetching all records`.
- **CHANGELOG.md "v1.1 Breaking Changes" section** written in this phase — removed flag → what to use instead.
- **`--all` ceiling**: stderr warning `warning: --all stopped at 1000 records (server ceiling); results may be incomplete`; exit stays 0 (FIX-06).
- **2 plans**: (1) error layer — RFC 7807 parser, Forbidden variant, 409 hint; (2) models/pagination/dead-flags.
- **Keep `api/mod.rs` single-file this phase** (981 lines) — revisit at Phase 13.
- **Stub-server tests** (TcpListener pattern from 07-05) for 403 hints, 409 hint, 422 detail parsing, fractional-position fixture; unit tests for flag removals.
- **Error layer first** — hard gate for Phases 9–12.

### the agent's Discretion
- Exact serde mechanics for the expand field type (see Assumption A1 — `Option<serde_json::Value>` vs flatten `Map`)
- Surface param representation (enum vs `&'static str`)
- Whether to DRY the 7 identical `fetch_all` loops (config says granularity "standard"; no decision either way)

### Deferred Ideas (OUT OF SCOPE)
- `api/mod.rs` split into `api/client.rs` + `api/methods/` — only at Phase 13
- NDJSON stdin support (descoped Phase 7)
- Client-side `--expand` validation whitelist — rejected, pass-through stands
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FIX-01 | Fix or remove misleading filter flags | Dead-flag inventory verified in `src/cli/*.rs` (§ Recommended Approach: FIX-01); hidden-flag parse-then-error pattern; workflows create TTY prompt also lies |
| FIX-02 | `--expand` payloads render in output | Drop point identified (serde unknown-field discard at deserialization); flatten catch-all design (§ FIX-02) |
| FIX-03 | Position fields deserialize floats | DB column types + serializer verified from server source (§ Position Field Inventory) |
| FIX-04 | `stages list` allows omitting `--pipeline` | Server route verified: `pipeline_id` optional (§ FIX-04) |
| FIX-05 | RFC 7807 error parsing; Forbidden variant; 409 hint | Exact `ProblemDetail` shape verified from `src/lib/api/errors.ts`; 409 detail text captured verbatim (§ API Contract Details) |
| FIX-06 | `--all` ceiling warning | 7 entities share identical `fetch_all` (batch 100, cap 1000); warning text locked by CONTEXT (§ FIX-06) |
</phase_requirements>

## Summary

Phase 8 makes the CLI truthful about server errors, model shapes, and list truncation. All server-side claims were verified directly against the Pipelite server source tree (Next.js at `/home/pedro/programming/pipelite`), upgrading the milestone research from HIGH-confidence citation to primary-source verification. The critical discovery: **the current error parser reads `error`/`message` keys, which never exist in an RFC 7807 body** — every 422/409/403 today renders as bare "HTTP 422" with a generic hint. The server's `ProblemDetail` shape is `{type, title, status, detail, instance?, errors?}` with `errors[]` items shaped `{field, code, message}`, and `errors[]` appears **only on 422** where `detail` is the generic string "Request validation failed" — so the parser must join `errors[]` entries, not just prefer `detail`.

The position fix is more nuanced than "server emits floats": `deals.position` is a Postgres `numeric` serialized via `parseFloat` (fractional possible), but `stages.position` is a Postgres `integer` passed through unchanged (always integral today). The CONTEXT decision to use `f64` for both is still correct — `f64` deserializes integer JSON tokens losslessly and is forward-safe — but planners should know Stage data won't exercise the fractional path. One user-visible output change follows: serde_json renders whole `f64` as `10000.0` where the server's JavaScript emitted `10000`.

Secondary discoveries that affect planning: (1) `workflows create` also has an **interactive TTY prompt** ("Set workflow active?") that sends `active` on create and lies just as hard as the flag — removing only the clap flag leaves the lie in place; (2) `stages_table_config` **already includes `pipeline_id`** in default columns, so the CONTEXT decision "add pipeline_id to default columns" is already satisfied — the work is only in params/command layer; (3) the server can return 403 on the ping path via `check_auth_status`, a third mapping site besides `handle_response`/`handle_delete_response`.

**Primary recommendation:** Build Plan 1 (error layer) exactly as CONTEXT locks it — `parse_rfc7807` helper + `Forbidden` variant + surface param threaded from ~43 call sites inside `api/mod.rs` only, command layer untouched. Plan 2 changes models (`f64` positions, flatten expand catch-all on the 7 Base models), relaxes `StagesListParams.pipeline_id` to `Option<String>`, converts 4 flags to hidden parse-then-error, applies client-side `--active` filtering, and rewrites the ceiling warning in all 7 `fetch_all` loops.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| RFC 7807 parsing | API client (`api/mod.rs`) | — | Single chokepoint (`handle_response`/`handle_delete_response`); commands never see raw responses |
| Forbidden variant + hints | API client (variant + surface param) | Error display (`error.rs`) | Classification belongs where the status code is read; rendering is centralized in `display_error` |
| 409 inactive-trigger mapping | API client | — | Status-code routing happens in `handle_response`; hint keyed on surface (workflows) |
| Position float deserialization | Models (`api/models.rs`) | — | Pure serde concern |
| Expand payload passthrough | Models (flatten field) | Output layer (renders whatever Value carries) | Models must preserve unknown keys; renderers already operate on `serde_json::Value` |
| stages all-mode | API client params (`StagesListParams`) + `commands/stages/list.rs` | — | Params own the query string; command owns the required-flag check it currently enforces |
| Dead-flag removal | CLI defs (`cli/*.rs`) + command handlers | — | Hidden flag + parse-then-error guard in command layer (before any HTTP) |
| Pagination ceiling warning | `commands/*/list.rs` (7 files) | — | `fetch_all` lives per-command |

## API Contract Details

### RFC 7807 ProblemDetail (VERIFIED: server source `src/lib/api/errors.ts:5-12`)

```typescript
export type ProblemDetail = {
  type: string          // "https://api.pipelite.app/errors/<CODE>"
  title: string         // "Unauthorized" | "Forbidden" | "Not Found" | "Validation Error" | "Conflict" | ...
  status: number
  detail: string
  instance?: string
  errors?: Array<{ field: string; code: string; message: string }>
}
```

**Per-status construction (VERIFIED: same file, `Problems` helpers):**

| Status | type code | title | detail | errors[] |
|--------|-----------|-------|--------|----------|
| 401 | UNAUTHORIZED | Unauthorized | "Authentication required" | never |
| 403 | FORBIDDEN | Forbidden | "You don't have access to this resource" | never |
| 404 | ENTITY_NOT_FOUND | Not Found | `"{entity} not found"` | never |
| 422 | VALIDATION_ERROR | Validation Error | **"Request validation failed"** (generic) | **always — the real info** |
| 409 | CONFLICT | Conflict | real message, e.g. inactive-trigger text | never |
| 429 | RATE_LIMITED | Rate Limited | fixed text; `Retry-After` header | never |
| 500 | INTERNAL_ERROR | Internal Server Error | "An unexpected error occurred" | never |

**Key implication:** on 422 the `detail` key is worthless ("Request validation failed"); the actionable text lives only in `errors[]` items. Parser must render errors[] entries (recommended join: `"{field}: {message} ({code})"` per line or `"; "`-joined). Elsewhere `detail` is the primary text.

**409 inactive-trigger exact detail (VERIFIED: `src/app/api/v1/workflows/[id]/run/route.ts`, both pre-check and race path):**
> `"Workflow is not active. Activate the workflow before triggering a run."`

CONTEXT locks the CLI hint text; the server `detail` is threaded into the error's `detail` field, the CONTEXT hint into `hint`. Note the create→trigger trap (CONTEXT/milestone): every new workflow starts inactive, so trigger 409s until activated — the hint must name the activation command.

**Current CLI parser is wrong (VERIFIED: `src/api/mod.rs:231-234, 279-282`):**
```rust
serde_json::from_str::<serde_json::Value>(&error_body).ok()
    .and_then(|v| v.get("error").or(v.get("message")).map(|m| m.to_string()))
    .unwrap_or_else(|| format!("HTTP {}", status_code));
```
Neither `error` nor `message` exists in a ProblemDetail → every error detail today is `"HTTP {status}"`. Note `m.to_string()` on a matched Value also JSON-quotes strings (`"some text"` with literal quotes) — the new parser should use `as_str()`.

**Recommended parse order (mirrors `.planning/research/PITFALLS.md` Pitfall 2, now source-verified):**
1. `errors[]` present → join entries `"{field}: {message} ({code})"` (422 case)
2. else `detail` as string (409/404/500 case)
3. else `title` (degenerate bodies)
4. else legacy `error`/`message` keys (tolerance for non-v1 endpoints, e.g. older server builds)
5. else `"HTTP {status}"`

**Three 401/403 mapping sites (VERIFIED):** `handle_response` (mod.rs:237), `handle_delete_response` (mod.rs:285), and `check_auth_status` (mod.rs:187-198, used by ping/ping_fallback). All three must split 401→Auth, 403→Forbidden. Easy to miss the third.

**Exit codes:** current `Auth` → exit 1 (`error.rs:77-90`: only MissingInput/InvalidInput/clap get 2). `Forbidden` also maps to exit 1 — a runtime condition, not structural input. 401 and 403 are distinguished by variant/message, not exit code. 409 → `Api { status: 409 }` or a dedicated mapping — still exit 1.

### Server envelope facts relevant to this phase (VERIFIED: SERVER-API-DIFF header + route sources)
- List: `{data: [...], meta: {total, offset, limit}}` + `X-Total-Count`; `limit` default 50, **max 100**
- Stages list route reads `pipeline_id` from query; **optional** — `ownedStagesPredicate(userId, pipelineId?: string | null)` only pushes the stage filter term when present (`src/lib/api/stage-scope.ts:71-81`); route comments confirm all-stages-across-pipelines works

## Position Field Inventory (VERIFIED: server DB migrations + serializer)

| Field | Postgres type | Serializer | Wire shape | CLI today | Fix |
|-------|--------------|------------|-----------|-----------|-----|
| `deals.position` | `numeric DEFAULT '10000' NOT NULL` (drizzle/0000:136) | `parseFloat` (serialize.ts:80) | JS number → `10000` (int token) or `10010.5` (float token); `null` when falsy | `Option<i64>` (models.rs:40) — **breaks on fractional** | `Option<f64>` |
| `stages.position` | `integer NOT NULL` (drizzle/0000:122) | passthrough (serialize.ts:134) | always int token | `i64` (models.rs:327) — works today | `f64` per CONTEXT decision (forward-safe; f64 accepts int tokens) |
| `custom_field_definitions.position` | `numeric(20,10) DEFAULT '10000'` (drizzle/0003) | `parseFloat` (serialize.ts:151) | fractional possible | model doesn't exist yet (Phase 12) | note for CFLD-01: must be f64 from birth |
| `deals.value` | text column | `parseFloat` (serialize.ts:75) | number or null | `Option<f64>` | none — already correct |
| dates (`expected_close_date`, `*_at`) | timestamp | ISO string | string | `Option<String>`/`String` | none |

No other numeric fields risk float emission: organizations/people/activities/pipelines/workflows carry no position-like numerics.

**JS serialization nuance (important for tests):** `JSON.stringify(parseFloat("10000"))` emits `10000` — an **integer token**. serde_json parses that into `f64` fine. But Rust `serde_json::to_string(&10000.0f64)` emits `10000.0`. So after the fix, CLI JSON output shows `position: 10000.0` where the raw server body had `10000`. This is correct-but-cosmetically-different — call it out in CHANGELOG and pin it in test expectations. (Custom serializers to strip `.0` would violate "models match server output" simplicity — don't hand-roll one.)

## Recommended Approach (per requirement)

### FIX-05 — Error layer (Plan 1, hard gate for Phases 9–12)

1. **`Forbidden` variant** in `src/error.rs`:
```rust
#[error("Forbidden")]
Forbidden { detail: String, hint: String },
```
Add match arms in `display_error` (error.rs:47-60). `exit_code` needs **no change** (falls to 1). Suggested title with status context, matching the `Api` variant's pattern: `#[error("Forbidden (HTTP 403)")]` so batch per-item lines carry the code.

2. **Surface key.** Recommend a plain `&'static str` (or small enum) threaded as a parameter:
```rust
async fn handle_response<T: DeserializeOwned>(
    &self, response: reqwest::Response, surface: &'static str,
) -> Result<T>
```
Each of the ~43 internal call sites in `api/mod.rs` passes its entity name (`"deals"`, `"workflows"`, ...). **No command-layer changes** — the threading is entirely inside `api/mod.rs`, keeping the single-file decision comfortable. New surfaces (audit/trash/notes/webhooks in Phases 9–12) pass their own key and add hint entries.

3. **`parse_rfc7807` helper** (free function in `api/mod.rs`):
```rust
/// Extract a human-readable message from an RFC 7807 problem body.
fn parse_rfc7807(body: &str, status: u16) -> String {
    serde_json::from_str::<serde_json::Value>(body).ok()
        .and_then(|v| {
            if let Some(errs) = v.get("errors").and_then(|e| e.as_array()) {
                if !errs.is_empty() {
                    return Some(errs.iter().filter_map(|e| {
                        let f = e.get("field")?.as_str()?;
                        let m = e.get("message")?.as_str()?;
                        let c = e.get("code").and_then(|c| c.as_str()).unwrap_or("invalid");
                        Some(format!("{f}: {m} ({c})"))
                    }).collect::<Vec<_>>().join("; "));
                }
            }
            v.get("detail").and_then(|d| d.as_str()).map(str::to_string)
                .or_else(|| v.get("title").and_then(|t| t.as_str()).map(str::to_string))
                .or_else(|| v.get("error").and_then(|e| e.as_str()).map(str::to_string))
                .or_else(|| v.get("message").and_then(|m| m.as_str()).map(str::to_string))
        })
        .unwrap_or_else(|| format!("HTTP {status}"))
}
```
Both `handle_response` and `handle_delete_response` call it; legacy `error`/`message` keys stay in the fallback chain (only legacy non-7807 servers produce them). Use `as_str()` — not `.to_string()` on Value — to avoid JSON-quoted detail text.

4. **409 mapping** in `handle_response` (and delete handler for symmetry):
```rust
409 => Err(CliError::Api {
    status: 409,
    detail: error_detail, // from parse_rfc7807
    hint: if surface == "workflows" {
        "Workflow trigger is inactive — activate it before firing (`pipelite workflows update <id> --active true`)".into()
    } else {
        "Resolve the conflict and try again.".into()
    },
}.into()),
```
409 is only known to fire on the trigger path today (VERIFIED: server route); the surface guard keeps the specific hint honest.

5. **Forbidden hint table** (default for existing surfaces; phases 9–12 extend):
```rust
let hint = match surface {
    "audit" => "The audit log requires an admin API key.".to_string(),          // Phase 11 surface, defined now for completeness
    "trash" => "Permanent purge requires an admin API key.".to_string(),        // Phase 11
    "notes" => "You can only modify your own notes (or use an admin key).".to_string(), // Phase 10
    "webhooks" => "This webhook belongs to another user.".to_string(),          // Phase 11; note: foreign webhook → 403, not 404
    _ => "Your API key doesn't have permission for this action.".to_string(),
};
```
Thread the server's `detail` ("You don't have access to this resource") into the error `detail` field so the server's voice is preserved.

6. **`check_auth_status`** (mod.rs:187): split — 401 keeps Auth hint; 403 emits Forbidden with the generic permission hint (ping has no meaningful surface; pass `"general"`).

### FIX-03 — Position floats (Plan 2)

- models.rs:40 → `pub position: Option<f64>`; models.rs:327 → `pub position: f64`.
- Update the three model tests that assert position (`deal_deserializes_from_json_with_nulls` uses `"position": null` — fine; `api_list_response_deserializes_with_meta` uses `"position": 1`; `stage_deserializes_from_json` asserts `stage.position == 2` → becomes `2.0`).
- Add a fractional fixture: `"position": 10010.5` deserializes; assert `serde_json::to_value` round-trips as `10010.5`.
- Deal create/update models carry no position (server auto-assigns max+10000 — VERIFIED: deals/route.ts:291) — no request-model changes.

### FIX-02 — Expand passthrough (Plan 2)

**Where payloads die (VERIFIED):** serde silently discards unknown fields during deserialization of the 7 Base models; expand payloads (`owner`, `organization`, `person`, `stage`, `type`, `deal`, `stages`, `pipeline` — SERVER-API-DIFF §D-6) are sibling top-level keys, so they're gone before `serde_json::to_value` runs in `*_to_values()`.

**Mechanism:** flatten catch-all on each Base model (the only mechanically correct reading of "expand field" — expands are arbitrary sibling keys, so a fixed named field can't capture them):
```rust
/// Captures `--expand` relation payloads (owner, organization, ...) and any
/// other server-emitted keys the typed fields don't model.
#[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
pub expanded: serde_json::Map<String, serde_json::Value>,
```
(`use serde_json::Map;`). **Zero command-layer or renderer changes**: `*_to_values()` re-serialization now emits expand keys; JSON format includes them automatically; table/csv/plain show them only when the user names the key via `--fields` (object values render as compact JSON via `format_value`'s fallback — format.rs:32).

Apply to all 7 Base models (Deal, Organization, Person, Activity, Pipeline, Stage, Workflow). Create/Update models untouched (request-only, per CONTEXT).

**Caveats:** flatten forbids `deny_unknown_fields` (not used anywhere — VERIFIED); adds a small serde slowdown (irrelevant at CLI scale); any *new* server-emitted key also passes through, which is exactly the "models match server output" goal. Test: round-trip `{"id":..., "owner": {...}}` → expand survives; empty case → no `expanded` key in output (skip_serializing_if).

See Assumption A1 regarding `Option<serde_json::Value>` wording in CONTEXT.

### FIX-04 — stages list all-mode (Plan 2)

1. `StagesListParams.pipeline_id: String` → `Option<String>`; `to_query_pairs` pushes the pair only when `Some` (mirrors every optional filter in the same file).
2. `commands/stages/list.rs:15-18`: delete the `--pipeline` required check; thread `args.pipeline.clone()` through `fetch_page`/`fetch_all` instead of `&str`.
3. `pipeline_id` in default columns: **already present** (`stages_table_config` — models.rs:361: `["id", "name", "pipeline_id", "position"]`). Nothing to do; add a test asserting it stays.
4. Update `after_help` examples (cli/stages.rs:41) to show `pipelite stages list` (all-mode) and `--pipeline` as optional.
5. Note: `list_deals` and the dashboard also fetch stages by pipeline (4 call sites of `list_stages` — VERIFIED) — they always pass Some; unaffected.

### FIX-01 — Dead flags (Plan 2, breaking)

**Removal inventory (all VERIFIED in `src/cli/*.rs`):**

| Flag | Definition site | Why dead | Replacement hint |
|------|----------------|----------|------------------|
| `people list --org` | people.rs:73-74 | server route reads neither param (SERVER-API-DIFF §D-1) | "people list does not support server-side org filtering; pipe through `jq` or fetch all and filter" |
| `people list --owner` | people.rs:77-78 | same | same style |
| `workflows create --active` | workflows.rs:113-115 (+ handler workflows/create.rs:24, 66-76) | create schema has no `active`; server always creates inactive | "new workflows start inactive; activate with `pipelite workflows update <id> --active true`" |
| `pipelines create --custom-field` | pipelines.rs:104-105 | pipelines have no custom fields; flag never parsed/sent | "pipelines do not support custom fields" |
| `stages create --custom-field` | stages.rs:135-136 | same | "stages do not support custom fields" |

**Keep:** `workflows list --active` (workflows.rs:63-65) — retained as a client-side filter; **live `--custom-field` on deals/orgs/people/activities create+update is NOT touched** (those work).

**Parse-then-error pattern:**
```rust
/// [REMOVED v1.1] Server ignores --org on people list.
#[arg(long, hide = true)]
pub org: Option<String>,
```
then at the top of `commands/people/list.rs::run` (before any HTTP):
```rust
if args.org.is_some() || args.owner.is_some() {
    return Err(CliError::InvalidInput {
        detail: "--org/--owner were removed: the server ignores them and returns unfiltered data".into(),
        hint: "Fetch people (`pipelite people list`) and filter client-side, e.g. with jq.".into(),
    }.into());
}
```
`hide = true` keeps clap accepting the flag (parse-then-error) while hiding it from `--help`. `InvalidInput` → exit 2 (error.rs:80-85 — already correct, no exit_code change).

**Workflows create specifics (easy to miss):**
- Remove the **interactive dialoguer Confirm** "Set workflow active?" (workflows/create.rs:66-76) — it also sends `active` on create and lies on TTY. After removal, `active` is never part of `WorkflowCreate` construction in the create handler.
- Update the `has_flags` mutual-exclusion check (create.rs:22-26) which references `args.active`.
- Consider removing `active` from the `WorkflowCreate` model too — server ignores it (VERIFIED: milestone §D-4); leaving it lets stdin JSON carry a dead key silently. Removing it changes stdin error behavior for users who pass `active` in stdin JSON (serde would reject unknown field? No — serde ignores unknown fields by default; it would just drop silently, same as today). Safe to remove; flag as planner choice.
- Update after_help examples (workflows.rs:37-39 has none referencing --active create; list after_help at :26 references `--active true` — that's the list flag, kept).

**Docs surface in this phase:** CHANGELOG.md (new, "v1.1 Breaking Changes" per CONTEXT). `docs/api-reference.md` (stages list `--pipeline` required row at :266; error-code table at :381 saying 401,403→Auth) and `docs/SKILL.md` are cross-checked in Phase 13 per milestone plan, but if touched incidentally, keep consistent.

### FIX-06 — `--all` ceiling warning (Plan 2)

7 entities have `fetch_all` with identical shape (VERIFIED: deals, stages, workflows, pipelines, activities, orgs, people list.rs — batch 100, cap 1000). Current message `Showing {max} of {total}. Use --limit/--offset for more.` (e.g. deals/list.rs:83-88) is informational, not the locked warning.

Replace (or precede) with the CONTEXT-locked line in all 7:
```rust
eprintln!("warning: --all stopped at 1000 records (server ceiling); results may be incomplete");
```
Exit stays 0 — tests must assert success exit code with the warning on stderr. Plumbing to honor `--quiet`/`--no-color`: check how other stderr lines handle ctx.quiet — if a quiet flag is reachable in these handlers, respect it; otherwise note that decisions lock the exact line (stderr warnings already exist elsewhere in the codebase, e.g. batch summary "survives --quiet" is stdout; verify quiet semantics before shipping).

Discretion opportunity: the 7 loops are copy-paste identical — a shared `paginate_all` generic helper would collapse them, but CONTEXT sets no decision; keeping the mechanical per-file edit is lower-risk and honors "standard" granularity.

## Pitfalls

### Pitfall 1: Missing the third 401/403 site
**What:** `check_auth_status` (api/mod.rs:187-198) also maps 403→Auth on the ping path. Fixing only `handle_response`/`handle_delete_response` leaves `pipelite init`/`ping` misreporting 403 as bad-key.
**Avoid:** grep `StatusCode::FORBIDDEN` after the change — must appear only in Forbidden mappings.

### Pitfall 2: `.to_string()` on matched JSON values
**What:** current code does `v.get("error")...map(|m| m.to_string())` — for a JSON string this yields quoted text (`"Authentication required"` with literal quotes).
**Avoid:** `as_str()` + `str::to_string` in `parse_rfc7807`. Add a fixture asserting no surrounding quotes in rendered detail.

### Pitfall 3: errors[] is 422-only; detail is generic there
**What:** a parser that prefers `detail` unconditionally renders "Request validation failed" for every 422 — technically true, uselessly vague.
**Avoid:** errors[]-first ordering (see helper above); only fall back to detail when errors[] is absent/empty.

### Pitfall 4: Removing the workflows create flag but not the TTY prompt
**What:** create.rs:66-76 prompts "Set workflow active?" on TTY and sends the answer — server ignores it; every workflow starts inactive regardless. Removing only the clap flag leaves the interactive lie.
**Avoid:** delete the prompt block and the `args.active` reference in `has_flags` in the same task.

### Pitfall 5: `f64` output formatting surprises
**What:** `serde_json` renders whole f64 as `2.0`; server JS emits `2`. JSON-diffing users see changed output.
**Avoid:** accept it (matches "server emits floats" truth), pin it in tests, note in CHANGELOG. Do not hand-roll integer-collapsing serializers.

### Pitfall 6: flatten field changes every Base-model fixture
**What:** adding `#[serde(flatten)] expanded: Map<String, Value>` to Base models affects all existing `serde_json::from_value`/`to_value` tests — structs no longer round-trip to exactly their typed keys when the fixture carries unknown keys; also `#[serde(default)]` is required or missing-key deserialization fails (flatten always matches).
**Avoid:** run the full model test module; add `default`; verify no `expanded` key appears in output when empty (skip_serializing_if).

### Pitfall 7: parse-then-error flags must reject before HTTP
**What:** the point of exit 2 + InvalidInput is "nothing ran". If the guard sits after cache-refresh or any HTTP call, the contract breaks.
**Avoid:** guard as the first statements of the command handler; test with unreachable server URL (127.0.0.1:1 pattern) asserting exit 2 and no connection attempt.

### Pitfall 8: Stages all-mode pagination identity
**What:** in all-mode, stages from different pipelines interleave; offset pagination is stable per-request but rows must remain attributable. `pipeline_id` column already exists — the risk is a test fixtures omitting it.
**Avoid:** stub-server fixture for all-mode includes `pipeline_id` on each stage row; assert rows distinguishable.

### Pitfall 9: --active client-side filter changes workflows list totals
**What:** today `workflows list --active` sends `active` as a query param the server ignores (VERIFIED: WorkflowsListParams.to_query_pairs:969-971 pushes it). Making it a real client-side filter changes result counts (honest) — scripts relying on the old silently-unfiltered behavior will see different output, plus a stderr warning.
**Avoid:** implement filter after fetch (per page or after --all), emit the locked warning once per invocation, update CHANGELOG (behavioral, not removed-flag).

### Pitfall 10: Deleting `active` from WorkflowsListParams but not the model
**What:** if the server-side query pair push is removed, the `WorkflowsListParams.active` field becomes dead code — compiler won't flag a pub struct field.
**Avoid:** single task updates params struct + both list handlers together; `cargo build` warnings for unused imports will catch stragglers.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON error-body extraction | Custom per-status parsing in each client method | One `parse_rfc7807` helper called from both handlers | PITFALLS.md "Never" table: copying extraction into new methods fossilizes the bug |
| HTTP stub for error tests | A test HTTP framework | TcpListener scripted-response pattern from `tests/batch_error_test.rs:42-119` | Already handles request framing, Content-Length, counters; zero new deps |
| Unknown-key capture in serde | Manual post-deserialization raw-body merge | `#[serde(flatten)] Map<String, Value>` | serde-native; hand-rolled merging breaks on nested/absent keys |
| Exit-code plumbing | New exit-code scheme for Forbidden | Existing `exit_code()` (error.rs:77-90) | InvalidInput→2 / everything-else→1 already implements the contract |
| Float parsing tolerance | Custom int-or-f64 enum | `f64` directly | serde_json parses int tokens into f64 losslessly |

## Common Pitfalls (codebase-specific quick list)

- `WorkflowCreate.active` is serialized into request bodies — server ignores it (create schema has no active). Removing the model field + flag + prompt together is the only complete truth-telling fix.
- `docs/api-reference.md:381` documents "401, 403 → Auth" — stale after this phase; Phase 13 cross-verifies docs but CHANGELOG lands now.
- Server limit cap is 100/request; `fetch_all` batches at 100 — correct today; don't "optimize" batch size upward.

## Code Examples

### Stub-server test for 403 Forbidden hint
```rust
// Source: tests/batch_error_test.rs:42 (existing pattern) + api/mod.rs status mapping
#[test]
fn forbidden_on_deal_get_shows_permission_hint() {
    let (url, _counter) = spawn_stub_server(&[(403, None)]); // extend script to carry bodies
    // stub body: {"type":"...FORBIDDEN","title":"Forbidden","status":403,"detail":"You don't have access to this resource"}
    let mut cmd = cmd_with_server(&url);
    cmd.args(["deals", "get", "deal_1"]);
    cmd.assert().failure().code(1)
        .stderr(predicates::str::contains("Forbidden").and(
               predicates::str::contains("permission")));
}
```
(The existing helper scripts `(status, Option<&str>)` for Retry-After; Plan 1 extends the tuple to carry a response body — small, mechanical.)

### Fractional position fixture
```rust
// Source: verified server behavior — serialize.ts:80 parseFloat, drizzle numeric column
#[test]
fn deal_position_deserializes_fractional_float() {
    let deal: Deal = serde_json::from_value(json!({
        "id": "d1", "title": "t", "value": null, "stage_id": "s1",
        "organization_id": null, "person_id": null, "owner_id": "u1",
        "position": 10010.5, "expected_close_date": null, "notes": null,
        "custom_fields": null,
        "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"
    })).unwrap();
    assert_eq!(deal.position, Some(10010.5));
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `error`/`message` key extraction | RFC 7807 `detail`/`errors[]` parsing | This phase (server has always spoken 7807) | All error details become real server text |
| 403 → Auth | 403 → Forbidden with surface hints | This phase | Phases 9–12 admin/ownership surfaces get honest errors |
| `workflows list --active` as (ignored) server param | Real client-side filter + warning | This phase | Honest results; stderr warning line |
| Silent 1000-record `--all` cap | Loud ceiling warning | This phase | FIX-06 |

**Deprecated/outdated:**
- `check_auth_status` conflating 401/403 — split in this phase.
- i64 position typing — replaced by f64 (server `numeric` columns + parseFloat make fractional canonical).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | CONTEXT's "Typed models gain `Option<serde_json::Value>` expand field" is best implemented as `#[serde(flatten)] serde_json::Map<String, Value>` — expands are sibling keys with arbitrary names, so no fixed-named `Option<Value>` field can capture them; flatten-Map is the only mechanically correct reading and satisfies the decision's stated intent (output renders expand payloads, skip-when-empty) | FIX-02 | Low — alternative (per-relation named Option<Value> fields for owner/organization/person/stage/type/deal/stages/pipeline) is possible but brittle if server adds relations; surface at discuss-phase only if user objects |
| A2 | `Forbidden` exit code 1 (same as Auth) — CONTEXT doesn't specify an exit code; existing contract puts runtime failures at 1 | FIX-05 | Low — scripts keying on exit codes see no change |
| A3 | Removing `active` from the `WorkflowCreate` model (not just the flag) — server ignores it; stdin JSON carrying `active` continues to deserialize (unknown fields ignored by default) | FIX-01 | Very low — pure dead-weight removal; planner may keep the field to minimize diff |
| A4 | orgs list `--owner` is also dead (SERVER-API-DIFF §D-2) but is **not** in FIX-01 text or CONTEXT decisions — treated as out of scope | Dead-flag inventory | None if truly out of scope; flag raised in Open Questions |
| A5 | Ceiling warning respects `--quiet` if a quiet flag is reachable in list handlers — CONTEXT locks the warning text but not quiet semantics; existing codebase precedent suggests stderr warnings always print | FIX-06 | Low — worst case warning prints under --quiet, which matches "warn loudly" intent |

## Open Questions

1. **Should `orgs list --owner` join the removal list?** (RESOLVED — out of Phase 8 scope; CHANGELOG carries a known-limitations note)
   - What we know: SERVER-API-DIFF §D-2 says server ignores it; FIX-01 names only `people list --org/--owner`.
   - Recommendation: leave out of Phase 8 scope (not in locked decisions); note in CHANGELOG under "known limitations" or defer to Phase 13 docs pass.
   - Resolution: locked as recommended — `orgs list --owner` is out of scope (08-02 "Do NOT touch" list); the CHANGELOG "Known limitations" note is in 08-02 Task 3 step 7.
2. **Does the ceiling warning need to replace or supplement the existing "Showing 1000 of N" line?** (RESOLVED — replace, inside the kept guard)
   - What we know: both convey the cap; CONTEXT locks the exact warning text.
   - Recommendation: replace the old line with the locked warning in all 7 files — one consistent voice; the `meta` footer ("Showing 1-1000 of 1243") already carries counts.
   - Resolution: locked as recommended — keep the `if total > max_records` guard, replace only the message inside it, with a negative under-ceiling test (08-02 Task 2 step 5 + behavior list).
3. **`errors[]` join format** (`"{field}: {message} ({code})"` proposed) is unspecified by CONTEXT. (RESOLVED — format locked)
   - Recommendation: keep the proposed format; it's display-only and trivially adjustable at review.
   - Resolution: locked as proposed — `"{field}: {message} ({code})"`, entries `"; "`-joined, `(invalid)` code default, items with missing field/message skipped (08-01 Task 1 action).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo/rustc | build + tests | ✓ | 1.94.0 / 1.94.0 | — |
| assert_cmd, predicates, tempfile | stub-server + CLI tests | ✓ (dev-deps pinned) | 2.0 / 3.0 / 3.0 | — |
| Live Pipelite server | (not required — stub servers suffice) | n/a | — | TcpListener stubs |

**Missing dependencies with no fallback:** none.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust 1.94, built-in) + assert_cmd integration tests |
| Config file | none needed (Cargo conventions) |
| Quick run command | `cargo test --quiet` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FIX-05 | 422 RFC 7807 body with errors[] renders server text | stub-server integration | `cargo test --test error_layer_stub_test` | ❌ Wave 0 (new file) |
| FIX-05 | 403 → Forbidden variant + surface hint (deals surface default) | stub-server integration | same file | ❌ Wave 0 |
| FIX-05 | 409 on workflows trigger → activation hint | stub-server integration | same file | ❌ Wave 0 |
| FIX-05 | parse_rfc7807 unit: errors[]-first, detail fallback, no JSON quotes, legacy keys | unit | `cargo test --lib api::tests` | ❌ Wave 0 (in api/mod.rs) |
| FIX-05 | ping-path 403 → Forbidden (check_auth_status) | unit/integration | same | ❌ Wave 0 |
| FIX-03 | fractional position deserializes; whole f64 serializes as N.0 | unit (models.rs tests) | `cargo test --lib models` | ❌ Wave 0 (extend existing mod) |
| FIX-02 | expand payload survives round-trip; absent → no `expanded` key | unit (models.rs tests) | same | ❌ Wave 0 |
| FIX-04 | stages list without --pipeline issues no pipeline_id param; rows show pipeline_id | stub-server integration | `cargo test --test stages_integration` | ✅ (extend) |
| FIX-01 | removed flags → InvalidInput, exit 2, hint text, zero HTTP | integration | `cargo test --test dead_flags_test` | ❌ Wave 0 (new file; 127.0.0.1:1 unreachable pattern) |
| FIX-01 | workflows list --active filters client-side + warning | stub-server integration | `cargo test --test workflows_integration` | ✅ (extend) |
| FIX-06 | --all at 1000 ceiling prints locked warning, exit 0 | stub-server integration | `cargo test --test deals_integration` (extend) | ✅ (extend) |

### Sampling Rate
- **Per task commit:** `cargo test --quiet` (full suite is fast enough at this codebase size)
- **Per wave merge:** `cargo test` + `cargo build` warning check
- **Phase gate:** full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/error_layer_stub_test.rs` — RFC 7807 fixtures (422 errors[], 403, 409, 401), extends the `spawn_stub_server` pattern (needs body-carrying script tuples)
- [ ] `tests/dead_flags_test.rs` — parse-then-error unit/integration for the 5 removed flag paths
- [ ] models.rs test additions (fractional position, flatten round-trip) — inside existing `#[cfg(test)] mod tests`
- [ ] No framework install needed

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | API key provided via config/env; no auth logic added |
| V3 Session Management | no | stateless bearer key |
| V4 Access Control | yes (classification only) | 403 → Forbidden variant; CLI surfaces server's decision, never makes its own |
| V5 Input Validation | yes | parse-then-error guards reject before HTTP; serde typed models |
| V6 Cryptography | no | none |
| V7 Error/Logging | yes | error details echo server text; API key never appears in error detail (verified: details use URL + server text only) |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Error text echoing untrusted server input to terminal | Tampering/Information Disclosure | Accept — rendering server-authored detail is the feature; no ANSI-escape sanitization exists today, out of scope |
| Verbose 403 messages enabling resource enumeration (webhooks 403 vs workflows 404) | Information Disclosure | Hints stay accurate but minimal; no existence-probing added (PITFALLS.md security note) |
| API key leakage via error detail | Information Disclosure | Keep detail sourced from response body/URL only — never interpolate credentials |

## Sources

### Primary (HIGH confidence)
- `/home/pedro/programming/pipelite/src/lib/api/errors.ts` — exact ProblemDetail shape, per-status helpers, 403 detail text, 409 construction
- `/home/pedro/programming/pipelite/src/app/api/v1/workflows/[id]/run/route.ts` — exact inactive-trigger 409 detail (both check and race path)
- `/home/pedro/programming/pipelite/src/lib/api/serialize.ts` — parseFloat on deal.value/deal.position/cfd.position; stage position passthrough
- `/home/pedro/programming/pipelite/drizzle/0000_large_nick_fury.sql`, `0003_futuristic_lord_tyger.sql` — column types: stages.position integer, deals.position numeric, cfd.position numeric(20,10)
- `/home/pedro/programming/pipelite/src/lib/api/stage-scope.ts` + `src/app/api/v1/stages/route.ts` — pipeline_id optional, ownedStagesPredicate accepts null
- Local source: src/api/mod.rs, src/error.rs, src/api/models.rs, src/cli/*.rs, src/commands/*/list.rs, src/output/*, tests/batch_error_test.rs
- `.planning/research/SERVER-API-DIFF.md`, `.planning/research/PITFALLS.md` — milestone-level synthesis (now spot-verified against server source)

### Secondary (MEDIUM confidence)
- docs/api-reference.md, docs/SKILL.md — CLI-documented behavior (stale spots identified: api-reference.md:266, :381)

### Tertiary (LOW confidence)
- none — all load-bearing claims verified against source

## Metadata

**Confidence breakdown:**
- API contract (RFC 7807, 403/409 texts): HIGH — read from server source directly
- Position field inventory: HIGH — DB migrations + serializer read directly
- Dead-flag inventory: HIGH — definitions and handlers read directly
- Architecture (flatten expand, surface param): MEDIUM-HIGH — serde flatten mechanics are standard-library behavior; A1 flagged for transparency
- Pitfalls: HIGH — derived from code reads, not speculation

**Research date:** 2026-09-03
**Valid until:** 2026-10-03 (stable domain; server source static within milestone)
