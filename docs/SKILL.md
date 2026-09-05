# Pipelite CLI — AI Agent Skill

> Terminal control of the Pipelite CRM: CRUD, batch automation, workflow runs,
> and the v1.1 server surfaces (notes, webhooks, trash, audit, custom fields,
> templates, OpenAPI docs). Designed to be driven headlessly by agents.

**How to read this skill:** this file is the entry point — it contains everything
you need for most tasks. Four reference files exist for depth; load one only
when its row in the router below says so:

| File | Load when |
|------|-----------|
| `docs/skill/operations.md` | You need exact flags, JSON field names, pagination caps, or format behavior for a command |
| `docs/skill/recipes.md` | You want a copy-paste recipe (first run, create-with-stage, batch, typed custom fields, webhook lifecycle, trash recovery, run watching) |
| `docs/skill/troubleshooting.md` | A command failed, output surprised you, or you're about to guess at behavior |
| `docs/skill/developing.md` | You are **modifying the CLI codebase** (patterns, build/test, conventions) — not operating it |

## 1. Connect (once per environment)

```bash
pipelite ping                                  # connectivity check — exit 0 = reachable
pipelite init                                  # or: pipelite config set server.url <URL> / set server.api_key <KEY>
```

Credentials resolve in order: env (`PIPELITE_SERVER_URL`, `PIPELITE_API_KEY`) >
`~/.pipelite/config.toml` > defaults. Headless agents: env vars are enough —
no config file needed.

## 2. The contract (true for every command)

**Exit codes** — branch on these; they are tested, not incidental:

| Code | Meaning |
|------|---------|
| `0` | Success (including dry-run previews and declined-confirmation flows where designed) |
| `1` | Item failure, user refusal, or not-found (e.g. batch partial failure, non-TTY delete refused without `--force`) |
| `2` | Structural rejection **before any HTTP** — bad enum, missing arg, mutually exclusive flags, malformed JSON. Zero mutations guaranteed. |

**Global flags** (on every command): `--format json|table|csv|plain`,
`--no-color`, `-q/--quiet`, `-v/--verbose`, `--no-input`, `--dry-run`.

**Output discipline:** data → stdout; diagnostics, per-item batch failures, and
summaries → stderr. `--format json` output is always a parseable top-level
array or object — pipe straight into `jq`.

**Errors are three lines and self-diagnosing:** category, detail, `hint:`.
The hint names the exact recovery (often a runnable command). Read it before
retrying blind.

**Agent rules of thumb:**
- Scripting mutations: always pass `--no-input`; add `--force` for deletes (non-TTY deletes refuse without it, exit 1, zero HTTP).
- Preview first: `--dry-run` shows the exact request and never touches the server.
- Deletes and `trash restore` print a plain-text confirmation (exit-code operations — don't parse their stdout; everything else honors `--format json`).
- Never use `trash purge` unless irreversible destruction is explicitly intended (admin-only, strongest confirmation, `--force` to skip).

## 3. Task router

| You need to… | Command sketch | Details |
|---|---|---|
| Check connectivity | `pipelite ping` | — |
| CRUD any of the 7 entities | `pipelite <deals\|orgs\|people\|activities\|pipelines\|stages\|workflows> list/get/create/update/delete …` | operations.md § Entities |
| Create a deal (needs a stage ID) | `stages list` → `deals create --title X --stage <id>` | recipes.md § R1 |
| Update/delete many records in one call | `deals update --stdin` / `deals delete id1 id2 --force` | recipes.md § R3 |
| Attach notes to a record | `notes add deals <id> --body "text"` | operations.md § Notes |
| Automate via webhooks | `webhooks create --url https://… --events deal.created` | recipes.md § R5 |
| Watch a workflow run until done | `workflows runs get <run> --workflow <wf> --watch --exit-status` | recipes.md § R6 |
| Recover a soft-deleted record | `trash list --type orgs` → `trash restore organizations <id>` | recipes.md § R7 |
| Store typed custom-field values | `custom-fields create …` then `deals create … --custom-field price=4` | recipes.md § R8 |
| Fetch the API contract | `pipelite docs --save openapi.json` | — |
| Page through large collections | `--limit/--offset`, `--all` (≤1000) | operations.md § Pagination |
| Inspect who changed what | `audit list --entity-type deal …` | operations.md § Audit |
| Reuse a workflow definition | `templates create --workflow <id>` | operations.md § Templates |

## 4. Core patterns (short version)

**Batch** (all 7 entities): `update --stdin` takes a JSON **array** of objects
each carrying `"id"`; `delete` takes positional IDs or a `--stdin` array of
IDs; `create --stdin` posts an array to the server's batch route (deals, orgs,
people, activities only). Items continue past per-item failures; stderr carries
`[i/n] Failed <id>: <reason>` lines and a final `N ok, M failed` summary (even
under `--quiet`); exit 0 only when every item succeeded. Malformed input exits 2
before any HTTP.

**Typed custom fields** (deals/orgs/people/activities): definitions
(`custom-fields`) are the type source. `--custom-field price=4` stores JSON
number `4` when `price` is a number definition; select options are validated
pre-HTTP; `--custom-field-json '{"k": …}'` bypasses inference entirely.
`--dry-run` types from cache only — never fetches.

**Body inputs** (notes add/edit, templates/definitions create): exactly one
source wins — `--body` (supports `@file` and `@-`) > `--stdin` > interactive
prompt. Two explicit sources → exit 2 before reading anything.

## 5. If something fails

Go to `docs/skill/troubleshooting.md` — it maps every exit code and common
error to its cause and fix. The short version: exit 2 = fix your input (the
hint lists valid values); exit 1 = read the per-item/`hint:` line (usually a
missing `--force` or a 404); connectivity problems say so explicitly.
