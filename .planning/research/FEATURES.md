# Feature Research

**Domain:** Rust CRM CLI (`pipelite`) — v1.1 "Server v2 Parity" milestone: new command surfaces (notes, workflow runs, webhooks, trash, custom fields, templates, audit, docs) + batch operations
**Researched:** 2026-09-02
**Confidence:** HIGH (core UX patterns verified against live official docs of gh CLI, Stripe CLI, Helm, kubectl; API surface taken as fact from `.planning/research/SERVER-API-DIFF.md`)

---

## Research Question

How do mature CLIs (gh, stripe-cli, sentry-cli, slack-cli, kubectl) handle: notes sub-resource UX, workflow run inspection, webhook secret show-once, trash/soft-delete confirmations, typed custom-field input, batch continue-on-error output, and audit log filtering? What is table stakes vs differentiator for pipelite v1.1?

## Verified Precedents (what mature CLIs actually do)

| Pattern | Precedent (verified) | Implication for pipelite |
|---------|---------------------|--------------------------|
| Sub-resource comments | `gh issue comment 12 --body "..."` — nested under parent entity, positional parent ID first, `--body` flag or `--body-file -` for stdin, **interactive prompt when no body given** (HIGH — cli.github.com/manual/gh_issue_comment) | Notes: parent-scoped list/add; content via flag, `@file`, stdin, or interactive prompt (dialoguer already in stack) |
| Run inspection | `gh run list --status=X`, `gh run view <id>` (summary + steps), `gh run watch --compact --interval 3 --exit-status` — non-zero exit if run fails (HIGH — cli.github.com/manual/gh_run_watch) | `workflows runs` list/detail table stakes; `--watch` + `--exit-status` is the differentiator pattern |
| Secret show-once | `stripe listen` prints `> Ready! Your webhook signing secret is whsec_abcdefg1234567` once, prominently; `--print-secret` flag for scriptability ("only print the webhook signing secret and exit") (HIGH — docs.stripe.com/cli/listen) | `webhooks create` renders the one-time secret with a warning block; JSON output carries it for scripts; never cached locally |
| Destructive delete | `gh repo delete` — prompts by default, `--yes` skips the prompt, and `--yes` is **ignored** in the maximum-danger case (no explicit target → always prompt) (HIGH — cli.github.com/manual/gh_repo_delete) | `trash purge` (permanent, admin-only): confirm prompt + `--yes` for scripts; `--no-input` without `--yes` must refuse, not silently proceed |
| Typed key=value input | Helm `--set a=b` (type-inferred), `--set-string` (force string), `--set-json '{"k":[1,2]}'` (escape hatch), `--set-file` — documented as the answer to "deeply nested data is hard on the command line" (HIGH — helm.sh/docs/intro/using_helm) | `--custom-field key=value` gains type inference from cached definitions; add `--custom-field-string` and `--custom-field-json` escape hatches |
| Batch error handling | `kubectl apply` processes each item independently, prints one result line per item, continues past individual failures, exits non-zero at the end (MEDIUM — kubectl reference partially retrieved; behavior well-established in widespread use) | Batch: per-item status lines + final `N ok, M failed` summary + aggregate exit code |
| Audit filtering | GitHub audit log API: composable query filters (action, actor, date); Slack audit API similar (action/actor/target/before-after) (MEDIUM — docs pages partially loaded; pipelite server filter shape verified in API diff) | `audit list` with enum-validated filter flags mirroring server params exactly — never client-side-only filters (v1.0's dead-flag lesson) |
| Documented exit codes | `gh help exit-codes` is a first-class help topic (HIGH — present in gh manual nav) | Document batch/watch exit codes in help text |

---

## Feature Landscape

### Table Stakes (Users Expect These)

Missing any of these makes the CLI feel incomplete against the server's own capabilities. All inherit existing v1.0 conventions (4 output formats, `--fields`, `--dry-run`, `--no-input`, `--quiet`, `--no-color`, hint-bearing errors) — for THIS project, honoring those flags on every new command is itself table stakes.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Notes: parent-scoped list/add (deals/orgs/people/activities) | Every mature CRM CLI exposes comments/notes on records; API supports it natively | LOW | `GET/POST /{entity}/{id}/notes`. CRUD-pattern replication. Content via `--content`, `--file`/`@path`, stdin, or interactive prompt |
| Notes: edit/delete by note ID | Server exposes global `PATCH/DELETE /notes/{noteId}`; asymmetric API needs asymmetric commands | LOW | No GET-single endpoint — edit/delete confirmations must show content from list context. Surface author-or-admin 403 with hint |
| Workflow runs list with `--status`/`--dry-run` filters | `gh run list --status=` is the mental model; server supports both filters | LOW | `dry_run` runs hidden by default — `--dry-run` flag opts in (document this; it will surprise users otherwise). Client-side enum validation for status (server doesn't validate — CLI should, per v1.0 dead-flag lesson) |
| Workflow run detail with steps | gh `run view` shows jobs/steps; debugging failed automations requires step-level `error` visibility | LOW-MEDIUM | `GET .../runs/{runId}` adds `steps[]`. Table: node_id, status, started/completed, error. `input`/`output` are arbitrary JSON → show in `--json` fully; truncated in table; `--step <nodeId>` drill-down optional |
| Webhooks CRUD with validation | Standard integration-management surface; server accepts ANY event name → client-side validation is the CLI's job | LOW-MEDIUM | 13-event enum validated in clap (arg_enum) → good clap errors for typos. `url` must be https (pre-validate). Update uses **PUT** not PATCH (full object required — differs from every other entity in the CLI; document in help) |
| Webhook secret shown once with warning | Stripe pattern (verified); users who miss the secret are locked out of signature verification forever | LOW | Warning block on create: "Store this secret now — it cannot be retrieved again." JSON format includes it for headless use; list/get never shows it (server excludes it) |
| Trash: list + restore | Soft-delete visibility is expected hygiene once the server has a trash tab | LOW-MEDIUM | `trash list [--type deals\|people\|organizations\|activities]`; rows show `name`, `deleted_at`, `deleted_by`, `linked_parents[]`. Plural-type positional matches server routes (`/trash/{type}/{id}`); restore = owner-or-admin (403 → hint) |
| Trash: purge with confirmation | Permanent, admin-only destruction — the `gh repo delete` case | LOW | Confirm prompt default; `--yes` skips; **`--no-input` without `--yes` = refuse with hint** (never default-confirm in headless mode). Non-admin 403 → "requires an admin API key" hint |
| Custom field definitions CRUD | Can't write typed values reliably without knowing field IDs/types; definitions are the schema | LOW | list (`--entity-type` filter) / get / create / delete. PUT semantics: `entity_type`+`type` immutable (client-side hint if user tries). Soft-delete → 404 on re-delete (hint: "already deleted"). `position` is server-assigned float — don't accept it on create |
| Type-aware `--custom-field` writing | Current CLI stores `"4"` not `4` — silently wrong data for number/boolean/date fields | MEDIUM | Coerce per definition type: number→f64, boolean→bool, date→string, single_select→string, multi_select→array. Definitions fetched via existing TTL cache (no extra HTTP per update). Unknown field ID → error listing valid IDs for that entity |
| `--custom-field-json` escape hatch | Helm `--set-json` pattern; multi_select/lookup/formula configs don't fit key=value | LOW | Raw JSON merged into the blob. Independent of definitions lookup — can ship before type inference |
| Batch update/delete with continue-on-error | v1.0 has batch create via `--stdin` on 3 entities; users expect update/delete parity with the same input model | MEDIUM | Per-item result lines (`<id> ok` / `<id> FAILED: <msg>`), final summary (`N succeeded, M failed`), exit 0 all-ok / 1 any-failed. Failed IDs printed last in a copy-pasteable block (json format: structured results array) |
| Audit log list with filters | GitHub audit log model; admin keys need change history from the terminal | LOW-MEDIUM | Flags mirror server exactly: `--entity-type --entity-id --actor-kind --workflow-run-id` + offset/limit. All enums client-validated. **403 must render the admin-key hint prominently** — this is the primary non-obvious failure mode |
| Workflow templates list/get/create/delete | CRUD parity expectation; no update endpoint exists (server) → don't invent one | LOW | Create reuses v1.0 patterns: `--trigger @file`/`--nodes @file` with `serde_json::Value` (established Key Decision). Name 1–200, description ≤2000, category ≤100 — pre-validate with good errors |
| `pipelite docs` | Self-documentation against the live server version; trivial to build | LOW | `GET /api/v1/docs` (no auth) → stdout as JSON (pipeable to jq), `--output <file>` optional. Not table stakes vs competitors (no major CLI has this) but effectively free |
| All new commands respect v1.0 global flags | Project's own bar: `--dry-run` (no HTTP), `--no-input`, `--quiet`, `--no-color`, 4 formats | LOW | Cross-cutting; fold into each feature, not a separate phase |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| `workflows runs --watch` (+ `--exit-status`) | Converts v1.0's fire-and-forget trigger into observe-and-react: `pipelite workflows run X --watch && notify` — the gh run watch loop, verified pattern. HIGH value because the server has no push/websocket | MEDIUM | Poll loop, `--interval` (default 3–5s), redraw or delta-print step statuses, stop on completed/failed, `--exit-status` → non-zero if failed. Must respect `--no-input` trivially (it's read-only). Watch on a completed run = single print |
| Batch failed-ID summary in pipeable form | Turns partial failure into a one-liner retry: `... \| pipelite deals update --stdin`. No comparable CRM CLI does this well | LOW | Almost free once batch exists: print failed IDs as the last line in plain format; include in JSON results array |
| Audit changes diff rendering (`field: from → to`) | GitHub shows change diffs; raw `changes{field:{from,to}}` JSON is unreadable in a table | LOW-MEDIUM | Human formats render compact diffs; `--json` keeps raw shape. Truncate long values in table view |
| Webhook event-name shell completions | 13 enum values → dynamic completion via existing clap_complete unstable-dynamic infrastructure | LOW | Same mechanism as entity-ID completion. Nice polish on an integration-critical surface |
| `--custom-field-string` force-string flag | Helm `--set-string` analog: opt-out when inference guesses wrong | LOW | Tiny addition once inference exists; covers select fields whose values look numeric |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Live-streaming run logs | `gh run view --log` familiarity | Server has NO log endpoint — only step `input`/`output`/`error` JSON snapshots. Anything "streaming" would be fake polling theater | Poll-based `--watch` on run/step status (honest, matches API) |
| Local storage/retrieval of webhook secrets ("show it again later") | Users lose the secret | Server never returns it again — caching it in `~/.pipelite/` (0600 or not) creates a secret-hoarding footgun the project never audited | Show-once warning + `--print-secret`-style quiet path at create time; document rotation-by-recreate |
| Full-replacement custom-fields blob write (`/api/custom-fields/save` model) | Seems simpler than merge | Session-only route (unusable with API keys) AND full-replace semantics silently drop fields you didn't send | Merge via normal entity update (server already merges `{...existing, ...updates}`) |
| Trash pagination beyond offset 10,000 | "I want everything" | Server hard-caps offset ≤10,000 — paginating past it returns errors | `--type` filter narrows results; document that huge trashes should be purged by type |
| Client-side-only filters that look server-side (`audit --action`, date ranges) | Feature completeness | This is exactly how v1.0 got dead flags (`people --org`, `workflows --active`) — filters that silently do nothing | Only ship filters the API diff verifies; note `created_at` ranges as a future server request |
| Deleting custom-field keys via update flags | Blob merge "can't delete" feels limiting | Server has no key deletion on v1 entity updates; faking it (sending null) depends on server coercion behavior — unverified | Set the key to a deliberate empty value; file a server request for key deletion |
| Interactive TUI for run steps / audit browsing | Visual appeal | PROJECT.md explicitly excludes TUI frameworks | Rich table + `--json` output; editors/jq handle the rest |

## Feature Dependencies

```
[Custom field definitions CRUD]
    └──requires──> (nothing new; TTL cache, CRUD pattern)
    └──enables──> [Type-aware --custom-field writing]   (coercion needs definitions)
                      └──enhanced by──> [--custom-field-string] (needs inference to opt out of)

[Workflow runs list + detail]
    └──requires──> (nothing new)
    └──enables──> [--watch + --exit-status]              (polls list/detail endpoints)

[Batch shared utility]
    └──requires──> existing --stdin plumbing (v1.0)
    └──enables──> [failed-ID pipeable summary]           (rendering on top of results)

[Webhooks CRUD]
    └──enhanced by──> [event-name completions]           (needs the enum to exist first)

[All new surfaces]
    └──requires──> AppContext / CliError hint convention / output renderers (v1.0, exist)

[Notes edit/delete] ──conflicts-with──> interactive assumptions:
    no GET-single endpoint → cannot show content before confirm; must state that in prompt text

[Type-aware writing] ──conflicts-with──> [batch --stdin] (resolved):
    batch payloads embed custom_fields as raw JSON → JSON path bypasses inference inside batch (document)
```

### Dependency Notes

- **Typed writing requires definitions CRUD:** coercion maps field-ID → type; without the definitions list endpoint shipped first there is nothing to coerce against. Phase order: definitions before typed writing. `--custom-field-json` is deliberately independent and can land either side.
- **Watch requires runs list/detail:** trivial dependency but real — build runs first, watch as a follow-up command in the same group.
- **Completions require the webhook enum:** the 13-event validation enum is the single source of truth; completion derives from it.
- **Notes edit/delete conflict with confirm UX:** no GET-single note route means a delete confirmation cannot display current content — the prompt should say so explicitly (mature CLIs show what they're about to destroy; when they can't, they say why).

## MVP Definition

### Launch With (v1.1 — this milestone)

All of it is already milestone-scoped in PROJECT.md; this ordering minimizes risk:

- [ ] Notes group (list/add/edit/delete) — highest-use surface, pure CRUD-pattern replication, fast win
- [ ] Workflow runs list + detail-with-steps — closes the observability gap left by fire-and-forget trigger
- [ ] Webhooks CRUD (event validation, https check, PUT semantics, show-once secret) — integration-critical
- [ ] Trash list/restore/purge (confirm + `--yes`, admin-403 hints) — safety-critical, needs careful confirm gating
- [ ] Custom field definitions CRUD + `--custom-field-json` — unblocks typed writing
- [ ] Type-aware `--custom-field` writing — depends on definitions + cache
- [ ] Batch update/delete via shared utility, continue-on-error, per-item + summary output, aggregate exit codes — 4 drafted plans already exist
- [ ] Workflow templates (list/get/create/delete via @file) — small CRUD surface
- [ ] Audit list with enum-validated filters + prominent admin-403 hint
- [ ] `pipelite docs` — near-zero cost
- [ ] §D fixes (dead filters, --expand passthrough, position float, stages all-mode) — trust repair; dead flags actively lie to users and are cheaper to fix than to document around

### Add After Validation (v1.1.x / follow-up)

- [ ] `workflows runs --watch --exit-status` — trigger after runs list/detail ships; poll loop needs terminal-behavior testing across TTY/non-TTY
- [ ] Webhook event-name completions — after enum exists; low-risk polish
- [ ] Audit changes diff rendering (`from → to`) — after audit list proves useful; pure rendering
- [ ] `--custom-field-string` — after inference ships and misfires are observed
- [ ] Batch `--delay`/throttle flag — only if real users hit the 500 req/60s rate limit on large batches

### Future Consideration (v2+, needs server work first)

- [ ] Global search via API key — server route is session-only (API diff §B); blocked server-side
- [ ] File upload/download via API key — session-only today; blocked server-side
- [ ] Sort/date-range/custom-field filters on lists — no server support (API diff §E); request upstream before building any client-side facade
- [ ] Docs-driven dynamic completions (parse the OpenAPI spec to generate flag completions) — clever but fragile; only if server spec drift becomes a real pain

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Notes CRUD | HIGH | LOW | P1 |
| Workflow runs list + detail | HIGH | LOW-MEDIUM | P1 |
| Webhooks CRUD + show-once secret | HIGH | LOW-MEDIUM | P1 |
| Trash list/restore/purge | HIGH | MEDIUM | P1 |
| Batch update/delete + summary/exit codes | HIGH | MEDIUM | P1 |
| Custom field definitions CRUD | MEDIUM-HIGH | LOW | P1 |
| Type-aware `--custom-field` + `--custom-field-json` | MEDIUM-HIGH | MEDIUM | P1 |
| §D dead-flag fixes | HIGH (trust) | MEDIUM | P1 |
| Audit list + 403 hint | MEDIUM | LOW-MEDIUM | P2 |
| Workflow templates CRUD | MEDIUM | LOW | P2 |
| `pipelite docs` | LOW-MEDIUM | LOW | P2 |
| Runs `--watch` + `--exit-status` | HIGH | MEDIUM | P2 |
| Event-name completions | MEDIUM | LOW | P3 |
| Audit diff rendering | MEDIUM | LOW-MEDIUM | P3 |
| `--custom-field-string` | LOW-MEDIUM | LOW | P3 |
| Batch retry/`--delay` tooling | LOW-MEDIUM | LOW | P3 |

**Priority key:** P1 = must land this milestone · P2 = should land, natural follow-up in same milestone if capacity · P3 = polish, deferrable

## Competitor Feature Analysis

| Feature | gh CLI | stripe-cli | kubectl / helm | pipelite Approach |
|---------|--------|------------|----------------|-------------------|
| Sub-resource notes/comments | Nested under entity: `gh issue comment <n>`; prompt when body omitted; `--body-file -` for stdin | (n/a) | (n/a) | Top-level `notes` group with type positional (`notes list deals <id>`) — notes have global noteId ops + 4 parent types; one clap definition instead of 4 duplicated groups. Content via flag/`@file`/stdin/prompt |
| Run inspection | `run list --status`, `run view` (+steps), `run watch --interval --compact --exit-status` | (n/a) | `kubectl get --watch` | `workflows runs <wf>` + `workflows runs <wf> <run>`; watch pattern adopted for follow-up; no fake streaming |
| Secret show-once | `gh secret set` (write-only; values never echoed back) | `stripe listen` prints secret once + `--print-secret` (verified) | k8s secrets via files | Stripe pattern: prominent one-time display + JSON-for-scripts; never cached, never in list output |
| Destructive confirm | `gh repo delete` prompts; `--yes` skips; `--yes` ignored in max-danger case (verified) | prompts for destructive ops | `kubectl delete` prompts; `--force` | `trash purge` + `webhooks delete` confirm by default; `--yes` for scripts; `--no-input` without `--yes` refuses with hint |
| Typed CLI input | (flag-typed) | (flag-typed) | Helm `--set` / `--set-string` / `--set-json` (verified) | Inference from cached definitions + string-force + JSON escape hatch |
| Batch errors | `gh api` is single-call; `gh pr list --json` for bulk reads | (n/a) | `kubectl apply`: per-item lines, continue past failures, non-zero exit (MEDIUM conf.) | Per-item lines + final summary + aggregate exit code + pipeable failed-ID list |
| Audit/filter UX | `gh api` audit-log w/ composed query filters | (n/a) | (n/a) | Flags mirror server params 1:1, client-validated enums, admin-403 as first-class hint |
| Self-docs | `gh help exit-codes`, extensive manual | `stripe openapi` (spec push/pull — MEDIUM confidence, current docs stub didn't confirm) | `kubectl explain` | `pipelite docs` fetches live OpenAPI 3.1 — differentiator for an API-first tool |

## Confidence Assessment

| Area | Confidence | Basis |
|------|------------|-------|
| Notes / runs / watch / secret / confirm / `--set` patterns | HIGH | Fetched from cli.github.com, docs.stripe.com, helm.sh on 2026-09-02 |
| kubectl batch continue-on-error | MEDIUM | Reference page partially retrieved (nav-heavy); behavior is well-established in widespread use but not verified verbatim in fetched text |
| Audit filter conventions (GitHub/Slack) | MEDIUM | Slack docs page redirected; GitHub audit-log shape inferred from API diff + training data — pipelite's server params are the ground truth anyway |
| `stripe openapi` command existing | MEDIUM | Training data; Stripe docs returned a stub page |
| API shapes, limits, permissions | HIGH | Taken as given from SERVER-API-DIFF.md (researched directly against server code) |

## Gaps / Open Questions

- `notes edit` UX without GET-single: consider having `notes list --json` be the documented "get the note first" path
- Does `webhooks update` (PUT) require sending the full object (url+events+active) or does the server treat omitted keys as no-op? PUT semantics suggest full-object — needs one-phase verification against server code before planning
- Batch + custom fields interaction: confirm batch `--stdin` payloads accept raw `custom_fields` JSON and document that inference is bypassed there
- Rate limiting on batch (500 req/60s): sequential per-item calls at >400 items will 429; decide whether v1.1 honors Retry-After automatically in batch mode or errors per-item

## Sources

- gh CLI manual — gh_run_watch, gh_issue_comment, gh_repo_delete (cli.github.com) — fetched 2026-09-02 — HIGH
- Stripe CLI — `stripe listen` reference (docs.stripe.com/cli/listen) — fetched 2026-09-02 — HIGH
- Helm — "Using Helm" (`--set` format and limitations; `--set-string`/`--set-json`/`--set-file`) (helm.sh/docs/intro/using_helm) — fetched 2026-09-02 — HIGH
- kubectl apply reference (kubernetes.io) — partially retrieved — MEDIUM
- Slack developer docs / audit API — redirected to docs home, not usable — attempt logged
- `.planning/research/SERVER-API-DIFF.md` — pipelite server API surface, permissions, limits — HIGH (project-internal, code-verified)

---
*Feature research for: pipelite CLI v1.1 Server v2 Parity milestone*
*Researched: 2026-09-02*
