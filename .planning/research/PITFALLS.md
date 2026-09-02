# Pitfalls Research

**Domain:** Adding new feature surfaces (batch ops, notes, workflow runs, webhooks, trash, custom fields, templates, audit, docs) to an existing Rust CRM CLI (pipelite v1.0 → v1.1) against a live server API
**Researched:** 2026-09-02
**Confidence:** HIGH — every pitfall below is grounded in two verified sources: (a) the actual v1.0 source code (read directly: `src/api/mod.rs`, `src/error.rs`, `src/cache.rs`, `src/dry_run.rs`, `src/prompt.rs`, command implementations), and (b) `.planning/research/SERVER-API-DIFF.md` (verified against server source, §A–§E).

> The single most important meta-finding: **the v1.0 error layer and pagination helper were built before the server quirks were known, and every new feature in this milestone flows through them.** Fixing the shared layers is not a "cleanup phase" — it is a prerequisite for notes, trash, audit, webhooks, and batch being correct. Phase ordering should reflect this.

---

## Critical Pitfalls

### Pitfall 1: 403/409 conflated into "Auth" — misleading hints on every admin-gated and foreign-resource path

**What goes wrong:**
`src/api/mod.rs` `handle_response()` (line 197) and `check_auth_status()` (line 149) map **both 401 and 403** to `CliError::Auth` with hint *"Check your API key. Run `pipelite init` to reconfigure."* The 409 status has no case at all — it falls through to generic `Api` with detail *"HTTP 409"* and hint *"Try again later."*

Every new feature has a 403 path where this hint is actively wrong:
- `trash purge` by a member (admin-only) → user is told to reconfigure their key, which won't help
- `audit list` by a non-admin → same
- `notes patch/delete` by a non-author member → same
- `webhooks delete <foreign-id>` → same (and note the asymmetry: workflows return 404 on foreign, webhooks return **403** — so code/scripts keyed on "404 = not mine" breaks)

And the 409 path: `workflows trigger` on an **inactive** workflow → 409 with no hint. This combines with §D bug 4 (server has no `active` on create; `workflows create --active` is silently ignored and every new workflow starts inactive) into a guaranteed trap: **create → trigger always 409s** for a new workflow unless the user separately activated it.

**Why it happens:**
The error enum was designed when 403 only ever meant "bad key." The new server surfaces introduce permission-tier 403s (admin), ownership 403s (author/webhook owner), and business-state 409s (inactive trigger), which were never in scope for v1.0.

**How to avoid:**
- Split the mapping: 401 → `Auth`; 403 → a **new `Permission` variant** whose hint is supplied by the calling command ("requires an admin API key", "you can only modify your own notes (or ask an admin)", "webhook belongs to another user").
- Add a `409` case to `handle_response` with route-aware hints; for workflow trigger specifically: *"Workflow is inactive — activate it first: `pipelite workflows update <id> --active true`"*.
- Keep one parse site (see Pitfall 4) so `detail` from the RFC 7807 body feeds these hints.

**Warning signs:**
- Any new command's test asserting `CliError::Auth` on a 403 fixture.
- The string "run `pipelite init`" appearing in any hint for trash/audit/notes/webhooks.

**Phase to address:**
**Foundation phase (error-layer fixes) — before any new surface ships.** Batch (Pitfall 2's error classification) and trash/audit/notes/webhooks all consume this.

---

### Pitfall 2: RFC 7807 error bodies parsed with the wrong keys — every server detail message is discarded

**What goes wrong:**
The server returns RFC 7807: `{type, title, status, detail, errors[]}` (SERVER-API-DIFF header). But `handle_response` and `handle_delete_response` extract detail via `v.get("error").or(v.get("message"))` — **neither key exists in an RFC 7807 body**. Result: today every 422 renders as *"HTTP 422 — Check the request parameters and try again"* with zero server-provided information. The `errors[]` array (field-level validation detail) is never surfaced.

This predates the milestone but becomes critical now: the new surfaces fail with rich server messages (invalid trash type → 422, immutable definition fields → 422, webhook url not https → 422, inactive workflow → 409 `detail`), and users can't see any of it.

**Why it happens:**
v1.0 was written against a guessed error shape before the server diff research existed. Both parse sites (`handle_response`, `handle_delete_response`) duplicated the same wrong extraction.

**How to avoid:**
- Parse `detail` (string) and `title` first, fall back to `errors[]` joined entries, fall back to `error`/`message` (keep for tolerance), fall back to `HTTP {status}`. One shared helper — do **not** copy the extraction into the new webhook/notes/trash client methods.
- Add a test fixture with a real RFC 7807 body per status class (422 with `errors[]`, 409, 403).

**Warning signs:**
- Any new command handler doing its own `response.text()` + JSON key fishing instead of going through `handle_response`.
- Screenshots/manual tests showing "HTTP 422" with no server message.

**Phase to address:**
**Foundation phase**, same PR/change as Pitfall 1 — they're the same function.

---

### Pitfall 3: Batch continue-on-error exit-code and summary semantics defined implicitly (or not at all)

**What goes wrong:**
`--continue-on-error` batch operations fail scripts in two opposite ways: (a) partial failure exits **0**, so `pipelite deals batch-delete --stdin ids.json && echo done` reports success while 12 of 50 deletes failed; or (b) the summary goes through `--quiet` suppression or into stdout, breaking `--format json` machine consumption. A third variant: the loop stops at the first failure anyway because "continue-on-error" was only implemented at the HTTP layer, not the loop layer.

Current `exit_code()` (src/error.rs) only knows: 2 = clap usage / MissingInput, 1 = everything else. There is no partial-success concept.

**Why it happens:**
Exit codes feel like a detail to defer, but scripts are written against them the day the feature ships — changing semantics later is a breaking change.

**How to avoid:**
Define and test the contract explicitly, up front, in the shared batch utility (the milestone already plans one — put the contract there, not in per-entity handlers):
- **0** = every item succeeded
- **1** = ≥1 item failed (including when `--continue-on-error` was set — the flag means "keep going", not "ignore failures")
- **2** = structural/validation failure before any HTTP call (bad stdin, missing ids) — consistent with existing MissingInput=2
- Summary line (e.g., `12 ok, 3 failed, 35 skipped`) goes to **stderr always**, even under `--quiet` (quiet silences per-item chatter, not the outcome). `--format json` emits a structured per-item result array: `[{id, ok, status, error}]`, preserving input order.
- Default is fail-fast (abort on first error, report what was done); `--continue-on-error` opts into full traversal.
- Decide and document 404-during-batch-delete: recommend **failure** (consistent with single delete), not silent idempotent success.

**Warning signs:**
- Per-entity batch handlers that each roll their own loop instead of calling the shared utility.
- Tests only covering the all-success path.

**Phase to address:**
**Batch utility phase (first batch phase)** — contract + tests land with the utility, before per-entity coverage.

---

### Pitfall 4: Batch stdin JSON parsing edge cases — empty input, object-vs-array, and validate-after-first-request

**What goes wrong:**
`--stdin` batch payloads hit a long tail of parsing traps: empty stdin (or `echo |`), a single JSON **object** where an array is expected, NDJSON streams, UTF-8 BOM, trailing garbage after the array, duplicate ids, per-item missing `id` on batch update, and huge arrays held in memory. The worst variant: the implementation validates items **lazily**, so item 47's malformed record fails after 46 HTTP mutations already ran — with no `--continue-on-error`, leaving a partially applied batch the user can't see.

**Why it happens:**
`serde_json::from_reader` handles the happy path; everything else is manual, and the `@filepath` precedent in `workflows trigger --data` (inline string OR @file) invites re-implementing input handling per command.

**How to avoid:**
- **Parse and structurally validate 100% of input before issuing the first HTTP request**: valid JSON array, each item an object, `id` present and non-empty (update/delete), reject unknown top-level shapes. Report the offending **array index** in the error ("item 47: missing `id`"), exit 2 (no mutation happened).
- Accept: array of objects; reject with a targeted message: empty input (`stdin was empty`), single object (hint: "wrap in an array"), NDJSON (hint: "one JSON array expected"). Strip BOM.
- Reuse one `read_json_stdin_or_file` helper (support `@filepath` for consistency with `workflows trigger --data`).
- Duplicate ids on batch update: dedupe-with-warning or error — pick one, test it.
- Size guard: warn (or hard-error) above a sane item count relative to the rate limit (see Pitfall 14) rather than silently chewing for 10 minutes.

**Warning signs:**
- Any `for item in items { client.update(item).await? }` where `items` wasn't fully validated first.
- Integration tests only feed well-formed arrays.

**Phase to address:**
**Batch utility phase** — stdin parsing and pre-validation live in the shared utility alongside the exit-code contract.

---

### Pitfall 5: Webhook secret destroyed by the output layer on its single appearance

**What goes wrong:**
The webhook `secret` is returned **only on create** (server excludes it from list/get, and PUT can never set it). v1.0's table renderer truncates long values via `truncate_with_ellipsis` (src/output/format.rs:96) and the command layer converts typed models → `serde_json::Value` → generic renderers. If the create response's secret flows through that path in table format, the user's one chance to copy it is rendered as `whsec_abc123...` — **permanently lost**; the only remedy is delete-and-recreate the webhook.

Secondary leaks: piping create output through `--format json` into a log file is fine (user asked), but the secret must never (a) be accepted as a CLI flag (shell history), (b) be written into `~/.pipelite/cache/`, or (c) be fabricated in `--dry-run` output.

**Why it happens:**
The generic output pipeline is convenient — webhook create "just works" with zero rendering code, and the truncation bug is invisible until someone actually loses a secret.

**How to avoid:**
- Webhook create is the one command that bypasses generic single-item rendering for the secret field: print the secret **full and untruncated** on its own line in human formats, with an explicit warning ("Save this now — it cannot be retrieved later"). JSON format includes the full object (including secret) — that's the machine contract.
- Never cache the create response. Never add a `--secret` flag. Dry-run previews the request body (url/events) only — the body contains no secret, keep it that way; do not print a placeholder secret.

**Warning signs:**
- Webhook create rendering through `output::render_single` with default column truncation.
- Any test asserting the secret is absent from create output.

**Phase to address:**
**Webhooks phase.** Write the truncation test first: create a webhook with a 64-char secret in table format and assert the full value appears.

---

### Pitfall 6: Trash singular/plural type mismatch — scripting round-trips 422, and purge is irreversible

**What goes wrong:**
Two landmines in one surface:
1. **Type vocabulary mismatch:** `trash list` rows carry `entity_type` **singular** (`deal`) while restore/purge path segments require the **plural** tab (`deals`, plus invalid → 422). A user scripting `trash list --format json | jq ... | xargs pipelite trash restore` pipes the singular value from the row into the path → 422. Users will also type `--type deal` naturally.
2. **Destructiveness asymmetry:** restore is recoverable-ish (record still exists), purge is **permanent and admin-only** (member → 403; live/unknown record → 404). Treating purge like every other delete command in v1.0 — which have **no confirmation at all** (see `deals/delete.rs`: dry-run check, HTTP call, done) — is a one-keystroke data-destruction footgun.

**Why it happens:**
The singular/plural split lives server-side (serializer field vs. route tab) and is invisible until you script against it. And v1.0's no-confirm delete pattern is the obvious template to copy.

**How to avoid:**
- **Normalize client-side:** accept singular or plural (and the other entity names) for `--type` and positional type args; canonicalize to plural before building the path. Internally, map from a row's `entity_type` to the plural segment so CLI-generated values always round-trip.
- Purge gets the strongest confirmation in the codebase: interactive (TTY) → `dialoguer::Confirm` showing the record name; headless (`--no-input`) → **require `--force`** (the existing convention from `workflows delete`, not a new `--yes`); neither present → exit 2 with MissingInput. Never auto-proceed.
- Restore returns **204 on a POST** — `handle_delete_response` is the existing 204 handler but is delete-shaped; generalize an `expect_empty_success` helper rather than abusing the delete one.
- On restore, invalidate the matching entity cache key (`KEY_DEALS` etc. per type) — the restored record is now live but `~/.pipelite/cache/deals.json` still doesn't know it (TTL_ENTITY_LIST is 5 min).
- 404 on purge/restore of a live record deserves a specific hint ("record is not in trash") rather than the generic not-found text.

**Warning signs:**
- Any `format!("{}/api/v1/trash/{}/{}", base, entity_type, id)` using the row field verbatim.
- A purge handler with no `--force`/confirm branch.

**Phase to address:**
**Trash phase** (with the `expect_empty_success` helper possibly landing in Foundation — coordinate with notes' 204-less POST shape).

---

### Pitfall 7: Type-aware `--custom-field` value parsing — the key=value grammar is ambiguous in five ways

**What goes wrong:**
Today `--custom-field key=value` always stores strings ("4", not 4 — SERVER-API-DIFF §C). "Type-aware" parsing sounds trivial but has repeated failure modes:
1. **Split position:** values containing `=` (`url=http://x?a=b`) — split on the **first** `=` only.
2. **Type source:** to parse "4" as number you need the definition's `type`, which means a definitions lookup (extra HTTP call per command, or cached definitions — stale cache → wrong type → server stores or rejects wrongly).
3. **Boolean/string collision:** for a `text` field, the user's literal "true" must stay a string; for a `boolean` field it must become `true`. Same input, two valid outputs — only the definition disambiguates.
4. **Array syntax for multi_select:** `[a,b]`? JSON `["a","b"]`? comma-split? Whatever is chosen must be documented and tested; JSON-array syntax is the least ambiguous.
5. **Silent no-ops:** writing to a **formula** key is stripped server-side (computed), and keys **cannot be deleted** via the API (merge-only blob) — an empty `key=` must not be presented as deletion.

**Why it happens:**
key=value flags invite regex-parse-and-go. The type information lives one API call away, and the string-typed v1.0 behavior "works" until someone filters `custom_fields.rating > 3` server-side and nothing matches.

**How to avoid:**
- Look up the definition by key (cached with a short TTL; invalidated by definitions CRUD) → parse per its `type` enum (text/number/date/boolean/single_select/multi_select/file/url/lookup/formula). Unknown key → warn loudly but still send as string (server accepts), so the escape hatch exists.
- Date: accept ISO 8601 only, pass through as string (server stores strings in jsonb) — don't invent timezone conversions.
- Warn specifically when the target key is a **formula** field ("server-computed; value will be ignored").
- Ship `--custom-field-json` in the same phase as the typed parser — raw JSON object merged into the blob — so any ambiguity has a documented out.
- Explicitly error (not silently send) on `key=` empty value: "custom field values cannot be deleted via the API".

**Warning signs:**
- A parser with no access to definitions (guessing types from value shape — "looks like a number" is wrong for text fields).
- No tests for `text` fields receiving "true"/"4" staying strings.

**Phase to address:**
**Custom fields phase** — and the definitions CRUD must land in the same phase *before* the typed parser (it's the type source).

---

### Pitfall 8: Pagination cap violations — server max limit=100, CLI silently caps at 1000, trash offsets capped at 10,000

**What goes wrong:**
Four distinct cap traps:
1. Server rejects/clamps `limit` > 100 (default 50). Naive new list commands that pass `--limit 500` through get silent truncation or an opaque 422.
2. The v1.0 `--all` implementation **silently stops at 1000 records** ("auto-paginates in batches of 100 up to 1000 records" — every list command doc comment). Copying this into `audit list --all` (logs grow unboundedly) or `workflows runs --all` produces quietly incomplete exports that users trust as complete.
3. **Trash-specific:** list `offset` is capped ≤ 10,000 server-side — a naive `--all` loop walking offset past 10,000 breaks mid-loop.
4. Interactive pickers (`prompt.rs` `get_pipelines_cached` etc.) fetch `limit: u64 = 100` — beyond 100 records, options silently vanish from interactive selection.

**Why it happens:**
The 1000 cap was a sensible v1.0 guard for entity lists; it was never surfaced to the user, so it's invisible. New surfaces have larger datasets (audit) and new caps (trash), so inherited assumptions fail in new places.

**How to avoid:**
- Clamp `--limit` to ≤100 client-side at the CLI layer with a visible warning, or pass through and surface server clamping.
- Build **one shared paginator** used by all new surfaces: loop on `offset += page_size` until `offset >= meta.total` (meta is already deserialized in `ApiListResponse`), with a hard safety ceiling **plus a stderr warning when any ceiling is hit** ("stopped at 1000 of 4,231 — refine filters"). Consider `--max N` to make the ceiling explicit.
- Trash `--all`: stop cleanly at the 10,000 offset cap with a message; never send offset > 10,000.
- Interactive pickers: bump pages or paginate; at minimum warn when the pick list is truncated.

**Warning signs:**
- Any new list command with its own hand-rolled loop instead of the shared paginator.
- `audit list --all` tests using <1000 fixtures only.

**Phase to address:**
**Foundation phase** (shared paginator) — audit/trash/runs phases consume it; existing `--all` commands migrate opportunistically.

---

### Pitfall 9: Float `position` deserialization failures — Deal, Stage, and definitions all emit floats

**What goes wrong:**
Server emits `Deal.position` via `parseFloat` — fractional positions (`1012.5`, the whole point of fractional insert-between) **fail deserialization** into `Option<i64>` (models.rs:40), killing `deals list`/`get` entirely for affected records. `Stage.position` is `i64` (models.rs:327) and custom-field-definition `position` is documented float (auto-assigned max+10000). This breaks not just reads but the batch round-trip: `deals get --format json` can't even produce output for a float-position deal, and `--stdin` batch updates carrying `"position": 1012.5` fail to parse into `DealUpdate`.

**Why it happens:**
Positions were integers when v1.0's models were written; the server moved to fractional ordering later (the diff doc flags it as bug §D-7).

**How to avoid:**
- Change all three to `f64` (or `serde_json::Number`). f64 represents integers exactly up to 2⁵³ — the max+10000 assignment scheme is safe.
- Audit every match on `position` for exhaustiveness after the type change (compiler will catch most).
- Accept the cosmetic change: JSON output shows `100.0` where it showed `100`. Test that `jq` consumers and the table renderer handle it; don't hand-roll int-vs-float display logic unless the table looks broken.
- Fixture test with `"position": 1012.5` at the **model deserialization** level, not just the command level.

**Warning signs:**
- `serde_json::from_str` tests still using `"position": 1` only.
- Any `as i64` / `as f64` casts introduced to "handle both".

**Phase to address:**
**Foundation phase (model fixes)** — must land before batch stdin testing and before any user with fractional positions can use the new commands.

---

### Pitfall 10: Expand-payload loss through typed models — and the `#[serde(flatten)]` re-serialization trap when fixing it

**What goes wrong:**
The loss mechanism is confirmed in code: `deals get` does `serde_json::to_value(&deal)` (deals/get.rs:17) after deserializing into typed `Deal` — serde silently drops unknown keys (the expanded `owner`/`organization`/`stage` objects) at **deserialization**, so `--expand` payloads never survive to output. §D-6.

The trap is in the obvious fix: adding `#[serde(flatten)] pub extra: Map<String, Value>` to entity structs "captures" the keys — but the **same structs feed request serialization** (create/update `.json(data)`), so captured expand keys get re-sent in PUT/POST bodies → server 422s (or silently ignores, which is worse: round-tripping stale expanded `owner` objects as update fields). A second variant of the same disease: user pipelines like `deals get <id> --expand owner --format json | jq '.title = "x"' | pipelite deals update <id> --stdin` now transmit expanded keys back.

**Why it happens:**
One struct, two serialization directions (parse responses, build requests) with opposite requirements. serde's flatten is invisible at the type level — nothing warns you that the extra map will serialize.

**How to avoid:**
- Decide the design **before** implementing: preferred option is output-path passthrough — when `--expand` is active, render from the raw `serde_json::Value` (the client methods already return typed models; add raw-returning variants or a `get_deal_value` that keeps the response Value for rendering) and keep typed models exclusively for request construction.
- If flatten is chosen: `#[serde(flatten, skip_serializing)]`-style separation is **not enough by itself** — request structs must be distinct from response structs, or request serialization must explicitly omit the extra map. Add a round-trip test: `get --expand` → parse → serialize → assert no expanded keys in the request body.
- Whatever the mechanism: `--expand` + output must show the expanded objects in all four formats, and there must be a documented statement about update-stdin ignoring (or rejecting) expanded keys.

**Warning signs:**
- A PR touching `models.rs` adding `flatten` without a corresponding change to request-building code.
- Manual test: `deals get --expand owner` still shows flat ids only.

**Phase to address:**
**Foundation phase (model/expand fixes)** — the design decision belongs in planning; implementation before or with the notes phase (notes on a deal are easiest to test against expanded deal output).

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Per-entity batch loops instead of shared utility | Faster to ship first entity | Exit codes, stdin parsing, rate-limit, and summary behavior drift per entity; 7 entities × bugs | Never — the utility is already milestone-planned |
| Copying `handle_response`'s error extraction into new client methods | One less refactor | Pitfall 2 fossilizes; 403/409 fixes miss the copies | Never |
| Keeping `--all`'s silent 1000 cap | Zero work | Users export audit logs believing they're complete | Only if a loud stderr warning is added in the same change |
| String-typed `--custom-field` values (v1.0 behavior) | Ships today | Server-side filters/comparisons on numbers/booleans silently fail | Never for new work — this is the bug being fixed |
| Typed models for run `steps[].input/output` and audit `changes` | Compile-time safety | Arbitrary server JSON breaks deserialization (from/to can be null/objects) — same class as Pitfall 9 | Never — use `serde_json::Value` for these fields by design |
| `--force` for destructive ops without dry-run support | Simpler flags | Scripts can't preview batch/purge effects | Never — `--dry-run` is a stated project convention |
| Accepting secrets as CLI flags (e.g., hypothetical `--secret`) | Test convenience | Secrets leak into shell history and ps output | Never |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Webhooks API | Reusing the PATCH mental model; assuming 404 for foreign ids | Update is **PUT** (v1.0 already uses `.put()` everywhere — keep it); foreign webhook → **403**, not 404 (opposite of workflows' 404) |
| Workflow templates | Building an `update` command by copying other entities | There is **no update endpoint** — only list/get/create/delete; don't ship a command that can only 404/405 |
| Custom-field definitions | Round-tripping a fetched definition back through PUT (get → jq → update --stdin) | `entity_type` and `type` are **immutable** — update model must exclude them or the round-trip 422s; position is float |
| Definitions list | Assuming list = live definitions only | List **includes soft-deleted** definitions — filter or visibly mark them; DELETE on already-deleted → 404 with a hint that should say "already deleted", not "not found" |
| Notes API | Adding `--author` flag; offering `notes get <id>`; allowing notes on pipelines/stages/workflows | Author is **forced** to the key owner (flag would be dead); **no GET single note** (edit/delete by id from list output); only deals/orgs/people/activities support notes — reject others client-side with a hint |
| Workflow runs list | Assuming empty result = nothing ran | Dry-run runs are **hidden** unless `?dry_run=true` — show a hint on empty results; `status` values are **not validated server-side** — validate client-side (pending/running/completed/failed/waiting) or typos silently return empty |
| Trash API | Using `entity_type` (singular) from rows in path segments | Path segments are **plural tabs** (deals/people/organizations/activities); normalize client-side (Pitfall 6) |
| Restore endpoint | Routing POST-204 through the delete-shaped 204 handler | Generalize an `expect_empty_success` helper; restore success prints nothing from the server — the CLI must synthesize the confirmation message |
| Audit API | Typing `changes.field.from/to` as strings | Values are arbitrary JSON (can be null, objects) — `serde_json::Value`; same for run step `input`/`output` |
| `pipelite docs` | Deserializing OpenAPI 3.1 into a typed model | Passthrough `serde_json::Value`; spec may be large — offer `--output <file>`; endpoint is public (no auth needed; the default header is harmless) |
| Rate limiting | Treating 429 as a permanent item failure in batch loops | 429 carries `Retry-After` — parse it; batch should back off and retry a bounded number of times before classifying the item as failed (Pitfall 14) |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Unthrottled batch loops | 429 errors partway through large batches; items classified as failures that were only rate-limited | Server allows **500 req/60s** (~8.3 req/s); throttle or back off on 429 with `Retry-After`; classify exhausted-retry 429 as failure with a "rate limited" detail | Batches ≳400–450 items; also any script combining `--all` export + batch mutate |
| `--all` on audit/workflow-runs surfaces | Minutes-long commands, silent 1000-item truncation, memory growth from full typed vectors | Shared paginator with streaming/early-exit filters; warn at ceiling (Pitfall 8) | Audit logs beyond a few thousand entries |
| Definitions lookup per `--custom-field` flag | One extra HTTP round-trip per flag per invocation | Resolve all flags against one cached definitions fetch per invocation | Every typed custom-field write; painful in batch-with-custom-fields scripts |
| Parallelizing batch requests for speed | Rate-limit storms, non-deterministic partial-failure state, unordered results | Keep it sequential — input-order results and simple error attribution are worth more than speed at 8 req/s | Any batch > ~50 items |
| Interactive pickers fetching unpaginated | Slow startup, incomplete lists | Cap at existing limit=100 but warn on truncation | orgs/deals >100 records |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Webhook secret persisted anywhere (cache file, debug log, error context) | Secret persists on disk past its one-show lifetime; webhook can be spoofed | Only ever held in the create response, rendered once (Pitfall 5); never in `~/.pipelite/cache/` |
| Echoing secrets through `--dry-run`/preview rendering | Dry-run bodies are meant to be pipeable | Create request bodies contain no secret — keep placeholder-free; audit this when adding dry-run to webhook create |
| Accepting secrets/tokens as flags | Shell history + `ps` leakage | No such flags; server generates the secret |
| Displaying audit `changes` values (from/to) by default in shared environments | Sensitive field values (comp values, PII) scroll into terminal scrollback/logs | Render fully in `--format json` (machine contract); consider whether human table truncates value bodies — document the choice |
| 403-message information leakage asymmetry | Webhooks 403 on foreign ids (confirms existence) vs workflows 404 — a CLI that "helpfully" distinguishes "does not exist" from "not yours" in hints is fine, but don't add enumeration-friendly behaviors like id existence pre-checks before delete | Keep hints accurate but minimal; no existence-probing endpoints |
| Purge confirmation skipped in scripts via `--no-input` without `--force` | Irreversible data loss from a fat-fingered script | `--no-input` without `--force` on purge → exit 2, never proceed (Pitfall 6) |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Empty `workflows runs` list with no explanation | User concludes their trigger failed; actually dry-run runs are hidden | On empty result, hint: "dry-run runs are hidden — pass `--dry-run` to include them" |
| `status:completed` typo returning empty results | Silent nothing; user thinks no runs completed | Client-side enum validation with "valid values: pending, running, completed, failed, waiting" in the error |
| 403 rendered as "check your API key / run pipelite init" | Member users run `pipelite init` repeatedly, then file bugs | Permission-specific hints naming the actual requirement (admin key, record ownership) |
| Dead flags "fixed" by client-side emulation (fetch-all + filter) | Commands get slow and silently capped without explanation | Remove the flag with `after_help` guidance, or implement server-side-equivalent behavior with explicit pagination + warning; never silently emulate |
| `deals get`/`list` failing wholesale on one float-position record | One bad record takes down whole workflows | Fixed by Pitfall 9 at model level — verify with a fractional fixture |
| Table-rendered secret (truncated) on webhook create | Secret unrecoverable; user must delete/recreate webhook | Untruncated one-line secret display with save-it-now warning (Pitfall 5) |
| Notes longer than a terminal width clipped in table view | Users can't verify note content | Table truncates (fine); `--format json`/plain show full 1–200,000-char content; `notes get`-style full view via list + `--fields content` |

## "Looks Done But Isn't" Checklist

- [ ] **Batch ops:** Often missing partial-failure exit code + stderr summary — verify: script with 1-of-3 failing ids exits 1 and prints `2 ok, 1 failed` even under `--quiet`; `--format json` emits per-item results.
- [ ] **Batch ops:** Often missing dry-run for the whole batch — verify: `--dry-run` previews **all** items with zero HTTP calls.
- [ ] **Webhooks:** Often missing secret-untruncated display + no-cache guarantee — verify: 64-char secret in table format is fully visible; nothing under `~/.pipelite/cache/` contains it after create.
- [ ] **Webhooks:** Often missing foreign-403 handling — verify: deleting another user's webhook id yields a permission hint, not "check your API key".
- [ ] **Trash:** Often missing plural normalization — verify: `trash list --format json | jq -r '.[0].entity_type'` piped back into `trash restore` succeeds (CLI maps singular→plural internally).
- [ ] **Trash:** Often missing `--no-input` guard on purge — verify: `--no-input` purge without `--force` exits 2 with a MissingInput error, performs no HTTP call.
- [ ] **Workflow runs:** Often missing empty-result hint and status validation — verify: `--status completd` errors client-side; empty list on a dry-run-only history prints the `--dry-run` hint.
- [ ] **Custom fields:** Often missing formula-field warning and immutable-field exclusion — verify: writing to a formula key warns; definitions update round-trip (get → update) doesn't resend `entity_type`/`type`.
- [ ] **Custom fields:** Often missing soft-deleted filtering in definitions list — verify: after a definition delete, list marks or omits it.
- [ ] **Error layer:** Often missing RFC 7807 `detail`/`errors[]` rendering — verify: a 422 fixture with `detail` + `errors[]` renders server text in the CLI error, not "HTTP 422".
- [ ] **Pagination:** Often missing cap warnings — verify: `--all` against a >1000-row fixture warns "stopped at ceiling" on stderr.
- [ ] **Position:** Often missing fractional fixture — verify: model test with `"position": 1012.5` deserializes.
- [ ] **Audit:** Often missing admin-403 distinct handling — verify: member key produces "requires admin" permission error.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Webhook secret lost (truncated/never shown) | MEDIUM | Delete webhook, recreate, save secret from the new create output; update the webhook URL in the emitting system |
| Trash purged by mistake | HIGH | **Unrecoverable** — permanent server-side; only mitigation is the confirmation UX + audit trail (check audit log for the delete actor) |
| Partial batch applied with no per-item record | MEDIUM | Re-run list/get to diff intended vs actual state; re-run batch with corrected stdin (operations are idempotent-ish for updates; deletes of already-deleted → 404s to reconcile) |
| Batch aborted mid-way by 429 storm | LOW | Wait `Retry-After`; re-run remaining items (per-item results make the remainder computable — hence Pitfall 3's JSON contract) |
| String-typed custom field values already stored ("4") | MEDIUM | Server merge accepts corrected typed writes; write a one-off script re-sending values parsed per definitions; formula keys self-heal (server-computed) |
| Expand keys sent back in an update body (422 or stale write) | LOW–MEDIUM | If 422: strip keys, retry. If silently written: re-update with correct values; the audit log shows what changed |
| Float-position deserialization in the wild (pre-fix release) | LOW | Ship the f64 model fix; no data migration needed (server data was always fine) |

## Pitfall-to-Phase Mapping

> Phase names are functional labels for the roadmap creator; exact numbering TBD. **Ordering rationale:** Pits 1, 2, 8, 9, 10 all live in shared infrastructure (error layer, paginator, models) that every subsequent phase consumes — they must precede or accompany the first new surface, not trail at the milestone's end (the milestone doc currently lists fixes last; this research recommends promoting the error/model/pagination subset to the front).

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. 403/409 conflation, misleading hints | Foundation (error layer) | Unit tests: 403 fixture → Permission variant with route-specific hint; 409 fixture on trigger → "activate first" hint |
| 2. RFC 7807 keys not parsed | Foundation (error layer) | Fixture with `detail` + `errors[]` renders server text in CLI error output |
| 3. Batch exit-code/summary contract | Batch utility phase | Integration: mixed-success batch exits 1, stderr summary survives `--quiet`, JSON per-item results |
| 4. stdin parsing/pre-validation | Batch utility phase | Edge-case tests: empty, object-not-array, NDJSON, BOM, missing id at index N — all exit 2 with zero HTTP calls |
| 5. Secret show-once vs truncation | Webhooks phase | Table-format create test with 64-char secret fully visible; cache-dir scan shows no secret |
| 6. Trash plural/singular + purge confirmation | Trash phase | Round-trip test (list JSON → restore); `--no-input` purge w/o `--force` exits 2; restore invalidates entity cache |
| 7. Typed custom-field parsing | Custom fields phase (definitions CRUD first) | Text-field-receives-"true" stays string; number/bool/date/multi_select parse per definition; formula warning; `=`-in-value test |
| 8. Pagination caps | Foundation (shared paginator) | >1000-row fixture warns at ceiling; trash --all stops cleanly at offset 10,000; no request ever sends limit>100 |
| 9. Float position | Foundation (models) | Deserialization fixture `"position": 1012.5` for Deal/Stage/definitions |
| 10. Expand loss / flatten trap | Foundation (models, design decision in planning) | `get --expand` shows expanded objects in all 4 formats; round-trip test asserts request bodies contain no expanded keys |
| 429 mid-batch handling | Batch utility phase | Simulated 429+Retry-After → backs off, retries boundedly, then classifies as failure with "rate limited" detail |
| PUT/immutable-fields/templates-no-update | Webhooks / Definitions / Templates phases | Definitions round-trip test; templates command set has no `update`; webhook update uses PUT |
| dry_run-hidden + status validation | Workflow runs phase | Typo'd status errors client-side; empty-list hint test |
| Dead-flag removal (people --org/--owner, workflows --active, create --active, dead --custom-field) | Foundation or a dedicated fixes phase | `--help` no longer lists removed flags; removal noted in changelog (breaking-change callout) |
| Notes constraints (no author flag, no single GET, entity allowlist) | Notes phase | Client-side rejection of pipelines/stages/workflows notes with hint; no `--author` flag exists |

## Sources

- `.planning/research/SERVER-API-DIFF.md` — server quirks verified against server source (§A surfaces, §D bugs 1–9, rate limit 500/60s, RFC 7807 envelope, pagination caps) — HIGH confidence
- `src/api/mod.rs` — `handle_response`/`handle_delete_response` error-key parsing (lines 190–218, 230–253), 401/403→Auth mapping (149–160), PUT usage, per-entity params/limit plumbing — read directly
- `src/error.rs` — CliError variants, `exit_code()` semantics (65–73) — read directly
- `src/api/models.rs` — `Deal.position: Option<i64>` (line 40), `Stage.position: i64` (line 327), `ApiListResponse` meta (17–27) — read directly
- `src/commands/deals/get.rs` + `src/commands/deals/delete.rs` — typed→Value output conversion (expand loss site), delete-without-confirmation + cache invalidation pattern — read directly
- `src/cache.rs` — TTLs, per-key invalidation, `invalidate_prefix` — read directly
- `src/dry_run.rs`, `src/prompt.rs`, `src/output/format.rs` (`truncate_with_ellipsis`), `src/cli/workflows.rs` (`--force` convention, `--data` @filepath), list-command `--all` 1000-cap doc comments — read directly
- `.planning/PROJECT.md` — milestone scope, conventions (`--dry-run`, `--no-input`, `--quiet`, `--no-color`) — read directly

---
*Pitfalls research for: pipelite CLI v1.1 — Server v2 Parity milestone*
*Researched: 2026-09-02*
