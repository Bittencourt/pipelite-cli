# Troubleshooting

Error → cause → fix. Read the `hint:` line first — it usually names the exact
recovery command. Then match the exit code below.

## Exit 2 — your input was wrong (zero HTTP, nothing mutated)

| Symptom | Cause | Fix |
|---------|-------|-----|
| `invalid type: map, expected a sequence` on `create/update --stdin` | Piped a single JSON object; arrays required | Wrap in `[ … ]` |
| `Missing required flags: --stage` (deals create) | Client-side requirement | `stages list` → pass `--stage <id>` (recipes.md § R1) |
| `Unknown trash type '…'` | Typo or singular-only alias | Valid: deal/deals, organization/orgs/organizations, person/people, activity/activities |
| Unknown event name on `webhooks create` | Not one of the 13 server events | Full list in the error hint / operations.md § Webhooks |
| `Multiple note body sources` | `--body` + `--stdin` both given | Pick one (`--body @-` counts as stdin) |
| `--custom-field`, `--custom-field-json`, and `--stdin` are mutually exclusive | Two value channels | Pick one |
| `Invalid JSON input` on batch | Malformed array or NDJSON | Arrays only; each update item needs `"id"` |
| Unknown select option (`Valid options: …`) | Value not in definition's `config.options` | Use a listed option, or fix the definition via `custom-fields update --config` |
| clap `unexpected argument '<x>' found` | Flag doesn't exist (e.g. `--continue-on-error` is not a flag) | Check `<command> --help`; batch continuation is built in |
| `trash purge` refusal in scripts | Non-TTY without `--force` | Intentional fail-safe. Add `--force` only if irreversible destruction is intended |

Exit 2 always fires before the first HTTP request — safe to fix and re-run.

## Exit 1 — the operation itself failed or was refused

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Refusing to delete without confirmation in non-interactive mode` | Non-TTY delete without `--force` — applies to single-ID and batch deletes alike | Add `--force` (deliberate script gate) |
| `API error (HTTP 500)` on `workflows delete` of a workflow with run history | Known server bug: `DELETE /api/v1/workflows/{id}` 500s once runs exist (works with 204 when no runs) — see pipelite issue #11 | No API-side recovery yet; the workflow stays. Track the upstream issue |
| `N ok, M failed` + `[i/n] Failed <id>: …` | Batch partial failure | Good items mutated; retry the failed IDs from stderr |
| `Not found` on get/update/delete | Wrong/typo'd ID or already deleted (soft-deleted records are gone from list) | Re-list to get the current ID; check trash (recipes.md § R7) |
| `Record not found` on `trash restore` | Not in trash — already restored or purged | Verify with a `get` |
| `Forbidden (HTTP 403)` + surface hint | Key lacks permission tier | audit/trash-purge need an admin key; notes edits need author-or-admin; foreign webhooks are owner-only (admins included) |
| `Workflow is not active. Activate…` on trigger | Workflow created inactive | `pipelite workflows update <id> --active true` |
| `API error (HTTP 429)` after a pause | Rate limited; one automatic retry already happened | Back off and re-run; batch items classify as failed with the rate-limit detail |
| `--watch` gives up after poll warnings | Server unreachable mid-watch | Re-attach with the same command; run state is server-side |

## Exit 130 — you interrupted `--watch`

Ctrl-C during watch. The run keeps executing server-side; re-attach with the
same `workflows runs get … --watch`.

## Output surprises (exit 0 but output looks wrong)

| Symptom | Explanation |
|---------|-------------|
| `deals delete` / `trash restore` printed plain text despite `--format json` | By design: deletes and restore are exit-code operations returning no body — don't parse their stdout; verify with a `get`. Everything else honors `--format json` |
| Batch summary appeared despite `--quiet` | Intentional: the `N ok, M failed` summary is the scriptable result channel (stderr) |
| JSON numbers render as `10000.0` where the server sent `10000` | Known f64 rendering nuance (documented in CHANGELOG) — numeric value is identical |
| `expanded` key appears in JSON after `--expand` | That's the inlined relation payload (Phase 8 contract) |
| Empty page got a hint line on stderr | Informational (`notes list`, batch empty input, …); suppressed by `--quiet` |
| Numbers became strings under `--dry-run` on a cold cache | Typed writing needs cached definitions; dry-run never fetches. Warm the cache once (`custom-fields list --entity-type <t>`) or accept strings in previews |
| Broken-pipe panic when piping into `head` | Exploration artifact — use `--limit`/`jq 'first'` instead of truncating pipes |

## Connectivity & auth

| Symptom | Fix |
|---------|-----|
| `Could not determine home directory` / config errors in containers | Set `PIPELITE_SERVER_URL` + `PIPELITE_API_KEY` env vars — no config file needed |
| `Auth (HTTP 401)` | Key rejected — check `PIPELITE_API_KEY` / `pipelite config show` |
| Connection refused / timeouts | Wrong `PIPELITE_SERVER_URL`, or `PIPELITE_CONFIG` points at a stale file |

## Diagnostic sequence for anything else

```bash
pipelite ping                                             # 1. server reachable?
pipelite config show                                      # 2. authenticated as whom?
<command> --dry-run                                       # 3. what WOULD it send?
<command> -v                                              # 4. verbose detail
```

Still stuck: the error's `hint:` line is contractually actionable — if it
names a command, run that command. If a hint turns out to be wrong or a
documented behavior doesn't match reality, that's a CLI bug: report it with
the exact command and the three error lines.
