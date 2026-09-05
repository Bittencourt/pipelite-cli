# Recipes

Copy-paste task walks. Every mutation here is either dry-run first or uses
throwaway `e2e-`/`agentsim-`-prefixed records — adapt prefixes to your
convention. Headless assumptions: `PIPELITE_SERVER_URL` + `PIPELITE_API_KEY`
set, `--no-input` everywhere, `--force` on deletes.

## R0 — First contact with a server

```bash
pipelite ping                          # exit 0 = reachable; latency printed
pipelite config show                   # what am I authenticated as? (no secret echoed)
pipelite deals list --limit 3 --format json | jq '.[] | {id, title}'
```

If `ping` fails: check `PIPELITE_SERVER_URL` reachability, then
`PIPELITE_API_KEY` (401 → key rejected; 403 → key lacks permission).
`pipelite init` is the interactive alternative; skip it in headless envs.

## R1 — Create a deal (stage ID required)

Deals must hang off a pipeline stage. `stages list` requires `--pipeline`,
so chain from the pipeline list:

```bash
PIPE=$(pipelite pipelines list --format json | jq -r '.[0].id')
STAGE=$(pipelite stages list --pipeline "$PIPE" --format json | jq -r '.[0].id')
pipelite deals create --title "acme-onboarding" --stage "$STAGE" \
  --value 1500 --format json --no-input
```

To find a specific stage by name across all pipelines:

```bash
pipelite stages list --format json | jq -r '.[] | select(.name | test("Cold";"i")) | .id'
```

## R2 — Find records before mutating

```bash
pipelite deals list --stage "$STAGE" --format json | jq -r '.[] | select(.title=="acme-onboarding") | .id'
pipelite orgs list --format json | jq -r '.[] | select(.name=="Acme") | .id'
```

IDs are UUIDs — always capture them from list/get output, never hand-type.

## R3 — Batch update + batch delete

```bash
# Update 2 deals in one invocation (array of objects, each with "id"):
echo '[{"id":"ID_A","title":"Renamed A"},{"id":"ID_B","value":2500}]' \
  | pipelite deals update --stdin --format json --no-input

# Delete many, non-interactively:
pipelite deals delete ID_A ID_B --force --no-input

# Delete from a pipeline of IDs:
cat ids.txt | pipelite deals delete --stdin --force --no-input
```

Partial-failure shape (expect it, don't fear it): failed items print
`[i/n] Failed <id>: <reason>` on stderr, a `N ok, M failed` summary follows,
exit code is 1 if anything failed / 0 only if all succeeded. Good items were
still mutated — collect the failed IDs from stderr and retry them.

Dry-run the exact payload first by adding `--dry-run` (zero HTTP).

## R4 — Create deals in bulk

```bash
cat <<'JSON' | pipelite deals create --stdin --format json --no-input
[{"title":"bulk-1","stage_id":"STAGE_ID"},
 {"title":"bulk-2","stage_id":"STAGE_ID"}]
JSON
```

Array required (a single object is rejected). Server batch route covers
deals/orgs/people/activities; other entities process per-item.

## R5 — Webhook lifecycle (secret handled safely)

```bash
OUT=$(pipelite webhooks create --url "https://example.com/hook" \
  --events deal.created,deal.updated --format json --no-input)
echo "$OUT" | jq -r '.secret'    # ⚠ shown EXACTLY ONCE — store it now; never in list/get
WH=$(echo "$OUT" | jq -r '.id')

pipelite webhooks update "$WH" --events deal.created,deal.deleted,activity.created --no-input
pipelite webhooks get "$WH" --format json | jq '{url, events, active}'
pipelite webhooks delete "$WH" --force --no-input
```

## R6 — Trigger a workflow and watch the run

```bash
WF=$(pipelite workflows list --format json | jq -r '.[] | select(.name=="My Flow") | .id')
pipelite workflows update "$WF" --active true --no-input      # trigger 409s until active
RUN=$(pipelite workflows trigger "$WF" --format json | jq -r '.run_id')

pipelite workflows runs get "$RUN" --workflow "$WF" --watch --exit-status --format json
# exit 0 = run completed, exit 1 = run failed (scriptable), Ctrl-C = 130
```

Without `--watch`, `runs get` is a snapshot; re-run to refresh. `runs list
--workflow "$WF"` shows history with `--status completed|failed|running|pending|waiting`.

## R7 — Recover a soft-deleted record

```bash
pipelite orgs delete "$ORG" --force --no-input               # soft delete → trash
pipelite trash list --type organizations --format json       # ⚠ --type is required to see this tab
pipelite trash restore organizations "$ORG" --no-input       # no confirmation needed
pipelite orgs get "$ORG" --format json                       # verify it's back
```

`trash restore` prints a plain-text confirmation — verify via `get`, not by
parsing restore output. `trash purge` is irreversible and admin-only; do not
use it in cleanup scripts.

## R8 — Typed custom fields end-to-end

```bash
pipelite custom-fields create --entity-type deals --key eval_score \
  --type number --format json --no-input

pipelite deals create --title "scored-lead" --stage "$STAGE" \
  --custom-field eval_score=7 --format json --no-input

pipelite deals get "$DEAL" --format json \
  | jq '{value: .custom_fields.eval_score, type: (.custom_fields.eval_score | type)}'
# → {"value":7,"type":"number"}   ← JSON number, not "7"
```

Selects validate options pre-HTTP; multi-select takes comma-separated values
(`tags=a,b` → `["a","b"]`); use `--custom-field-json '{"k": …}'` to bypass
inference. Re-typing an existing definition? Definition mutations invalidate
the cache automatically; force a fresh read with `pipelite cache clear` if a
stale type seems to be applied (rare — TTL 1h).

## R9 — Page through a big collection

```bash
# All deals up to 1000 in one call:
pipelite deals list --all --format json | jq 'length'

# Manual paging beyond that:
pipelite deals list --limit 100 --offset 0 --format json
pipelite deals list --limit 100 --offset 100 --format json
```

Caps differ per surface (audit ≤100, notes ≤100, trash offset ≤10,000 — see
operations.md § Pagination). Empty page + exhausted data = stop condition.
