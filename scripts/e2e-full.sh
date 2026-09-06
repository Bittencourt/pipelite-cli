#!/usr/bin/env bash
# scripts/e2e-full.sh — FULL-SUITE live-server E2E for pipelite.
#
# Covers every CLI surface against a REAL Pipelite server:
#   S01 preflight (ping, version, docs/OpenAPI, completions)
#   S02 CRUD lifecycle for orgs, people, deals, activities, pipelines, stages
#   S03 notes lifecycle
#   S04 webhooks (secret shown exactly once, update merge, pre-HTTP validation)
#   S05 trash round trip (soft delete -> list -> restore -> verify)
#   S06 typed custom fields (number/select/multi_select/verbatim JSON)
#   S07 workflows, runs (trigger/watch) and templates
#   S08 batch operations and their exit-code contract
#   S09 pagination, field projection, list filters
#   S10 output formats and global flags (quiet, no-color, dry-run, no-input)
#   S11 exit-code contract and error UX (2 pre-HTTP, 1 runtime, hints)
#   S12 audit log
#   S13 config file round trip + cache refresh/clear (fully isolated HOME)
#   S14 dashboard (slow on large datasets; opt out with E2E_SKIP_DASHBOARD=1)
#
# EXECUTION CONTRACT (same safety rules as scripts/e2e-v1.1.sh):
#   * Credentials come ONLY from the session environment: export
#       PIPELITE_SERVER_URL and PIPELITE_API_KEY (an ADMIN key — audit is
#       admin-gated) before running. The script refuses to run otherwise.
#     Credentials are NEVER read from files, never given defaults, never echoed.
#   * Only throwaway records prefixed "e2e-full-" are created, mutated, deleted.
#   * "trash purge" is NEVER invoked (permanent, admin-only) — enforced by the
#     pl() guard below, not by discipline.
#   * The trash round trip and soft-deleted cleanup records leave harmless
#     trash entries behind (soft delete is by design; never purged here).
#   * Results print to stdout AND append to $E2E_RESULTS_FILE
#     (default: ./e2e-full-results.txt — a local scratch file; do NOT commit).
#   * Exit status: 0 when every scenario PASSes, 1 otherwise.
set -euo pipefail

# ---------------------------------------------------------------------------
# Guard 1 — environment credentials, checked FIRST, with a usage message.
# ---------------------------------------------------------------------------
if [[ -z "${PIPELITE_SERVER_URL:-}" || -z "${PIPELITE_API_KEY:-}" ]]; then
  cat >&2 <<'USAGE'
e2e-full.sh: missing credentials.

This suite drives REAL mutations against a live Pipelite server and needs
session-environment credentials (never committed, never read from files):

    export PIPELITE_SERVER_URL   (your live server URL)
    export PIPELITE_API_KEY      (an ADMIN key — audit is admin-gated)

Then re-run:  bash scripts/e2e-full.sh
Optional:     E2E_SKIP_DASHBOARD=1  (skip the slow full-table dashboard scan)
USAGE
  exit 2
fi

# ---------------------------------------------------------------------------
# Guard 2 — the no-purge rule, ENCODED as code: every pipelite invocation
# MUST go through pl(), which refuses the permanent-destroy subcommand.
# ---------------------------------------------------------------------------
pl() {
  local arg
  for arg in "$@"; do
    if [[ "$arg" =~ ^purge$ ]]; then
      echo "FATAL: 'trash purge' is forbidden in this script (permanent, admin-only)." >&2
      exit 70
    fi
  done
  "$BIN" "$@"
}
readonly -f pl

# Binary resolution: build release once and run that binary directly.
cargo build --release --quiet
readonly BIN="$(pwd)/target/release/pipelite"

RESULTS_FILE="${E2E_RESULTS_FILE:-e2e-full-results.txt}"
: > "$RESULTS_FILE"
SCRATCH="$(mktemp -d /tmp/e2e-full.XXXXXX)"
trap 'rm -rf "$SCRATCH"' EXIT

# ---------------------------------------------------------------------------
# Assertion helpers — every failure prints observed vs expected.
# ---------------------------------------------------------------------------
PASSCount=0
FAILCount=0

pass() { echo "PASS $1: $2" | tee -a "$RESULTS_FILE"; PASSCount=$((PASSCount+1)); }
fail() { echo "FAIL $1: $2" | tee -a "$RESULTS_FILE" >&2; FAILCount=$((FAILCount+1)); }
assert_eq() { # assert_eq <scenario> <what> <expected> <observed>
  if [[ "$4" == "$3" ]]; then pass "$1" "$2 = $3"; else fail "$1" "$2 — expected [$3], observed [$4]"; fi
}
assert_contains() { # assert_contains <scenario> <haystack> <needle> <what>
  if grep -q "$3" <<<"$2"; then pass "$1" "$4 present"; else fail "$1" "$4 — expected to contain [$3], observed [$(head -c 300 <<<"$2")]"; fi
}
assert_not_contains() { # assert_not_contains <scenario> <haystack> <needle> <what>
  if grep -q "$3" <<<"$2"; then fail "$1" "$4 — must NOT contain [$3] but it does"; else pass "$1" "$4 absent (as required)"; fi
}

# run <args...> — a pl() invocation that never kills the script; captures
# stdout in OUT, stderr in ERR, exit code in RC.
run() {
  OUT="$(pl "$@" 2>"$SCRATCH/err")" && RC=0 || RC=$?
  ERR="$(cat "$SCRATCH/err")"
}

# run_nostdin <args...> — like run() but with stdin detached from the
# terminal, so interactive-confirm paths behave deterministically (non-TTY).
run_nostdin() {
  OUT="$(pl "$@" 2>"$SCRATCH/err" </dev/null)" && RC=0 || RC=$?
  ERR="$(cat "$SCRATCH/err")"
}

# run_stdin <payload> <args...> — like run() but feeds stdin.
run_stdin() {
  local payload="$1"; shift
  OUT="$(printf '%s\n' "$payload" | pl "$@" 2>"$SCRATCH/err")" && RC=0 || RC=$?
  ERR="$(cat "$SCRATCH/err")"
}

jqo() { jq -r "$1" <<<"$OUT"; } # jqo <filter> — jq over the last OUT

# compact <arrayname> — drop empty elements (defensive; arr_drop obsoletes it)
compact() {
  local -n ref="$1"
  local keep=() x
  for x in ${ref[@]+"${ref[@]}"}; do [[ -n "$x" ]] && keep+=("$x"); done
  ref=("${keep[@]+"${keep[@]}"}")
}

# arr_drop <arrayname> <value> — remove value (and empties) from an array.
arr_drop() {
  local -n ref="$1"
  local val="$2" keep=() x
  for x in ${ref[@]+"${ref[@]}"}; do [[ -n "$x" && "$x" != "$val" ]] && keep+=("$x"); done
  ref=("${keep[@]+"${keep[@]}"}")
}

# last_nonempty <arrayname> <fallback> — robust "pick a live id" helper.
last_nonempty() {
  local -n ref="$1"
  local fb="$2" x out
  out="$fb"
  for x in ${ref[@]+"${ref[@]}"}; do [[ -n "$x" ]] && out="$x"; done
  echo "$out"
}

# ---------------------------------------------------------------------------
# Created-record tracking for the cleanup trap (best effort, never purges).
# ---------------------------------------------------------------------------
TS="$(date +%s)"
declare -a ORGS=() PEOPLE=() DEALS=() ACTS=() PIPES=() STAGES=() WFS=() TPLS=() HOOKS=() DEFS=() NOTES=()

cleanup() {
  local id
  for id in ${TPLS[@]+"${TPLS[@]}"};     do pl templates delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${DEALS[@]+"${DEALS[@]}"};   do pl deals delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${ACTS[@]+"${ACTS[@]}"};     do pl activities delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${PEOPLE[@]+"${PEOPLE[@]}"}; do pl people delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${STAGES[@]+"${STAGES[@]}"}; do pl stages delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${PIPES[@]+"${PIPES[@]}"};   do pl pipelines delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${NOTES[@]+"${NOTES[@]}"};   do pl notes delete deals "${NOTES_PARENT:-x}" "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${HOOKS[@]+"${HOOKS[@]}"};   do pl webhooks delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${DEFS[@]+"${DEFS[@]}"};     do pl custom-fields delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${WFS[@]+"${WFS[@]}"};       do pl workflows delete "$id" --force --quiet >/dev/null 2>&1 || true; done
  for id in ${ORGS[@]+"${ORGS[@]}"};     do pl orgs delete "$id" --force --quiet >/dev/null 2>&1 || true; done
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Preflight — connectivity + a stage to hang test deals on.
# ---------------------------------------------------------------------------
echo "== pipelite FULL live E2E — $(date -u +%Y-%m-%dT%H:%M:%SZ) ==" | tee -a "$RESULTS_FILE"

run ping
assert_eq preflight "ping exit code" "0" "$RC"

STAGE_ID="$(pl stages list --format json --quiet 2>/dev/null | jq -r '.[0].id // empty' || true)"
if [[ -z "$STAGE_ID" ]]; then
  fail preflight "no stage found via 'stages list' — deals scenarios cannot run (server has no stages?)"
fi

# ===========================================================================
# S01 — preflight: version, OpenAPI docs, completions
# ===========================================================================
s01() {
  run --version
  assert_eq S01 "--version exit code" "0" "$RC"
  assert_contains S01 "$OUT" "pipelite" "version banner names the binary"

  local save="$SCRATCH/openapi.json"
  run docs --save "$save" --quiet
  assert_eq S01 "docs --save exit code" "0" "$RC"
  if [[ -f "$save" ]]; then
    assert_eq S01 "saved file is OpenAPI 3.1" "3.1.0" "$(jq -r '.openapi' "$save" 2>/dev/null || echo '?')"
  else
    fail S01 "docs --save produced no file at $save"
  fi
  run docs --save "$save" --quiet
  assert_eq S01 "docs --save refuses overwrite without --force (exit 2, pre-HTTP)" "2" "$RC"
  run docs --save "$save" --force --quiet
  assert_eq S01 "docs --save --force overwrites (exit 0)" "0" "$RC"

  run completions bash
  assert_eq S01 "completions bash exit code" "0" "$RC"
  if [[ ${#OUT} -gt 100 ]]; then pass S01 "completions output non-trivial (${#OUT} bytes)"; else fail S01 "completions output suspiciously small [${#OUT} bytes]"; fi
}

# ===========================================================================
# S02 — CRUD lifecycle for 6 entities (workflows live in S07)
#   create -> get (field round trip) -> update -> get (change visible)
#   -> list contains -> delete --force -> get is 404 (exit 1)
# ===========================================================================
s02_orgs() {
  local name="e2e-full-org-$TS"
  run orgs create --name "$name" --website "https://e2e-full-$TS.example.com" --format json --quiet
  assert_eq S02_orgs "org create exit code" "0" "$RC"
  local id; id="$(jqo '.id // empty')"
  [[ -z "$id" ]] && { fail S02_orgs "org create returned no id"; return 0; }
  ORGS+=("$id")

  run orgs get "$id" --format json --quiet
  assert_eq S02_orgs "org get exit code" "0" "$RC"
  assert_eq S02_orgs "org name round trip" "$name" "$(jq -r '.name' <<<"$OUT")"

  run orgs update "$id" --industry "e2e-testing" --format json --quiet
  assert_eq S02_orgs "org update exit code" "0" "$RC"
  run orgs get "$id" --format json --quiet
  assert_eq S02_orgs "org update visible on get" "e2e-testing" "$(jq -r '.industry' <<<"$OUT")"

  run orgs list --format json --quiet
  assert_eq S02_orgs "org list exit code" "0" "$RC"
  assert_contains S02_orgs "$OUT" "$id" "created org id appears in orgs list"

  run orgs delete "$id" --force --quiet
  assert_eq S02_orgs "org delete --force exit code" "0" "$RC"
  ORGS=()
  run orgs get "$id" --format json --quiet
  assert_eq S02_orgs "org get after delete exits 1 (not found)" "1" "$RC"

  # keep one org alive for downstream scenarios (deal/person links, trash, batch)
  run orgs create --name "e2e-full-anchor-$TS" --format json --quiet
  ORG_ID="$(jqo '.id // empty')"
  [[ -n "$ORG_ID" ]] && ORGS+=("$ORG_ID") || fail S02_orgs "anchor org create failed"
}

s02_people() {
  local last="e2e-full-$TS"
  run people create --first-name "E2E" --last-name "$last" --email "e2e-full-$TS@example.com" --org "$ORG_ID" --format json --quiet
  assert_eq S02_people "person create exit code" "0" "$RC"
  local id; id="$(jqo '.id // empty')"
  [[ -z "$id" ]] && { fail S02_people "person create returned no id"; return 0; }
  PEOPLE+=("$id")
  PERSON_ID="$id"

  run people get "$id" --format json --quiet
  assert_eq S02_people "person org link round trip" "$ORG_ID" "$(jq -r '.organization_id' <<<"$OUT")"

  run people update "$id" --last-name "updated-$TS" --format json --quiet
  assert_eq S02_people "person update exit code" "0" "$RC"
  run people get "$id" --format json --quiet
  assert_eq S02_people "person update visible on get" "updated-$TS" "$(jq -r '.last_name' <<<"$OUT")"

  run people list --limit 50 --format json --quiet
  assert_contains S02_people "$OUT" "$id" "created person id appears in people list"

  run people delete "$id" --force --quiet
  assert_eq S02_people "person delete --force exit code" "0" "$RC"
  PEOPLE=()
  run people get "$id" --format json --quiet
  assert_eq S02_people "person get after delete exits 1" "1" "$RC"

  # keep one person alive for the workflow action-node target (S07)
  run people create --first-name "E2E" --last-name "wf-$TS" --format json --quiet
  WF_PERSON_ID="$(jqo '.id // empty')"
  [[ -n "$WF_PERSON_ID" ]] && PEOPLE+=("$WF_PERSON_ID") || fail S02_people "workflow target person create failed"
}

s02_deals() {
  [[ -z "$STAGE_ID" ]] && { fail S02_deals "skipped — no stage id from preflight"; return 0; }
  local title="e2e-full-deal-$TS"
  # NOTE: the server rejects DATE-ONLY values for expected_close_date
  # ("Invalid ISO datetime") — full ISO datetimes are the working format.
  run deals create --title "$title" --stage "$STAGE_ID" --value 1500 --org "$ORG_ID" --expected-close-date "2026-12-31T00:00:00Z" --format json --quiet
  assert_eq S02_deals "deal create exit code" "0" "$RC"
  local id; id="$(jqo '.id // empty')"
  [[ -z "$id" ]] && { fail S02_deals "deal create returned no id"; return 0; }
  DEALS+=("$id")
  DEAL_ID="$id"

  run deals get "$id" --format json --quiet
  assert_eq S02_deals "deal stage_id round trip" "$STAGE_ID" "$(jq -r '.stage_id' <<<"$OUT")"
  assert_eq S02_deals "deal value round trip" "1500.0" "$(jq -r '.value' <<<"$OUT")"
  assert_eq S02_deals "deal org link round trip" "$ORG_ID" "$(jq -r '.organization_id' <<<"$OUT")"

  run deals update "$id" --value 2500 --title "e2e-full-deal-upd-$TS" --format json --quiet
  assert_eq S02_deals "deal update exit code" "0" "$RC"
  run deals get "$id" --format json --quiet
  assert_eq S02_deals "deal update visible on get" "2500.0" "$(jq -r '.value' <<<"$OUT")"

  run deals list --stage "$STAGE_ID" --org "$ORG_ID" --format json --quiet
  assert_eq S02_deals "deals list --stage --org exit code" "0" "$RC"
  assert_contains S02_deals "$OUT" "$id" "created deal appears in stage+org filtered list"

  run deals delete "$id" --force --quiet
  assert_eq S02_deals "deal delete --force exit code" "0" "$RC"
  DEALS=()
  run deals get "$id" --format json --quiet
  assert_eq S02_deals "deal get after delete exits 1" "1" "$RC"

  # keep one deal alive for notes (S03), expand (S09), batch (S08)
  run deals create --title "e2e-full-anchor-$TS" --stage "$STAGE_ID" --org "$ORG_ID" --format json --quiet
  DEAL_ID="$(jqo '.id // empty')"
  [[ -n "$DEAL_ID" ]] && DEALS+=("$DEAL_ID") || fail S02_deals "anchor deal create failed"
}

s02_activities() {
  local title="e2e-full-act-$TS"
  run activities create --title "$title" --type meeting --deal "$DEAL_ID" --due-at "2026-12-24T10:00:00Z" --format json --quiet
  assert_eq S02_activities "activity create exit code" "0" "$RC"
  local id; id="$(jqo '.id // empty')"
  [[ -z "$id" ]] && { fail S02_activities "activity create returned no id"; return 0; }
  ACTS+=("$id")

  run activities get "$id" --format json --quiet
  assert_eq S02_activities "activity deal link round trip" "$DEAL_ID" "$(jq -r '.deal_id' <<<"$OUT")"

  # Known server quirk (documented FAIL if it fires): --mark-done sends a
  # to_rfc3339() timestamp with +00:00 offset / ns precision, which this
  # server rejects ("Invalid ISO datetime"). Explicit --completed-at with a
  # Z timestamp is the supported path and MUST work.
  run activities update "$id" --mark-done --format json --quiet
  if [[ "$RC" == "0" ]]; then
    pass S02_activities "activities update --mark-done accepted by server (bug fixed upstream?)"
  else
    fail S02_activities "activities update --mark-done rejected by server (CLI sends RFC3339 +00:00; server wants Z-millis) — stderr: $(head -c 160 <<<"$ERR")"
  fi
  run activities update "$id" --completed-at "2026-09-05T10:00:00Z" --format json --quiet
  assert_eq S02_activities "activities update --completed-at (Z) exit code" "0" "$RC"
  run activities get "$id" --format json --quiet
  assert_contains S02_activities "$OUT" "2026-09-05T10:00:00" "completed_at round trip visible"
  run activities update "$id" --mark-undone --format json --quiet
  assert_eq S02_activities "activities update --mark-undone exit code" "0" "$RC"
  run activities get "$id" --format json --quiet
  assert_eq S02_activities "mark-undone clears completed_at" "null" "$(jq -r '.completed_at' <<<"$OUT")"

  run activities list --deal "$DEAL_ID" --format json --quiet
  assert_contains S02_activities "$OUT" "$id" "created activity appears in activities list --deal"

  run activities delete "$id" --force --quiet
  assert_eq S02_activities "activity delete --force exit code" "0" "$RC"
  ACTS=()
}

s02_pipelines_stages() {
  run pipelines create --name "e2e-full-pipe-$TS" --format json --quiet
  assert_eq S02_pipelines "pipeline create exit code" "0" "$RC"
  local pid; pid="$(jqo '.id // empty')"
  [[ -z "$pid" ]] && { fail S02_pipelines "pipeline create returned no id"; return 0; }
  PIPES+=("$pid")

  run stages create --name "e2e-full-stage-$TS" --pipeline "$pid" --type open --color blue --format json --quiet
  assert_eq S02_pipelines "stage create exit code" "0" "$RC"
  local sid; sid="$(jqo '.id // empty')"
  [[ -z "$sid" ]] && { fail S02_pipelines "stage create returned no id"; return 0; }
  STAGES+=("$sid")

  run stages get "$sid" --format json --quiet
  assert_eq S02_pipelines "stage pipeline_id round trip" "$pid" "$(jq -r '.pipeline_id' <<<"$OUT")"

  run stages update "$sid" --name "e2e-full-stage-upd-$TS" --format json --quiet
  assert_eq S02_pipelines "stage update exit code" "0" "$RC"
  run stages list --pipeline "$pid" --format json --quiet
  assert_contains S02_pipelines "$OUT" "$sid" "created stage appears in stages list --pipeline"

  run pipelines update "$pid" --name "e2e-full-pipe-upd-$TS" --format json --quiet
  assert_eq S02_pipelines "pipeline update exit code" "0" "$RC"

  run stages delete "$sid" --force --quiet
  assert_eq S02_pipelines "stage delete exit code" "0" "$RC"
  STAGES=()
  run pipelines delete "$pid" --force --quiet
  assert_eq S02_pipelines "pipeline delete exit code" "0" "$RC"
  PIPES=()
}

# ===========================================================================
# S03 — notes lifecycle on the anchor deal
# ===========================================================================
s03() {
  [[ -z "$DEAL_ID" ]] && { fail S03 "skipped — no anchor deal"; return 0; }
  local body="e2e-full note body $TS"
  run notes add deals "$DEAL_ID" --body "$body" --format json --quiet
  assert_eq S03 "notes add exit code" "0" "$RC"
  local nid; nid="$(jq -r '.id // empty' <<<"$OUT")"
  [[ -z "$nid" ]] && { fail S03 "notes add returned no note id"; return 0; }
  NOTES+=("$nid"); NOTES_PARENT="$DEAL_ID"

  run notes list deals "$DEAL_ID" --format json --quiet
  assert_eq S03 "notes list exit code" "0" "$RC"
  assert_contains S03 "$OUT" "$body" "note body visible via notes list (content field)"

  run notes edit deals "$DEAL_ID" "$nid" --body "e2e-full edited $TS" --format json --quiet
  assert_eq S03 "notes edit exit code" "0" "$RC"
  run notes list deals "$DEAL_ID" --format json --quiet
  assert_contains S03 "$OUT" "e2e-full edited $TS" "edited body visible via notes list"

  run notes delete deals "$DEAL_ID" "$nid" --force --quiet
  assert_eq S03 "notes delete --force exit code" "0" "$RC"
  NOTES=()
  run notes list deals "$DEAL_ID" --format json --quiet
  assert_not_contains S03 "$OUT" "e2e-full edited $TS" "deleted note gone from notes list"
}

# ===========================================================================
# S04 — webhooks: secret shown EXACTLY ONCE, update merge, validation
# ===========================================================================
s04() {
  local url="https://example.com/e2e-full-hook-$TS"
  run webhooks create --url "$url" --events deal.created --format json --quiet
  assert_eq S04 "webhook create exit code" "0" "$RC"
  local secret hid
  secret="$(jq -r '.secret // empty' <<<"$OUT")"
  hid="$(jq -r '.id // empty' <<<"$OUT")"
  [[ -z "$hid" ]] && { fail S04 "webhook create returned no id"; return 0; }
  HOOKS+=("$hid")

  if [[ -z "$secret" ]]; then
    fail S04 "create output carried no signing secret (must appear exactly once)"
  else
    assert_eq S04 "secret length (value REDACTED, only length recorded)" "64" "${#secret}"
    assert_eq S04 "secret occurrences in create output (shown exactly once)" "1" "$(grep -o -- "$secret" <<<"$OUT" | wc -l | tr -d ' ')"
    assert_not_contains S04 "$ERR" "$secret" "secret in stderr stream"
  fi

  run webhooks list --format json --quiet
  assert_eq S04 "webhooks list exit code" "0" "$RC"
  if [[ -n "$secret" ]]; then assert_not_contains S04 "$OUT" "$secret" "secret in webhooks list output"; fi
  assert_contains S04 "$OUT" "$hid" "created webhook appears in webhooks list"

  run webhooks get "$hid" --format json --quiet
  if [[ -n "$secret" ]]; then assert_not_contains S04 "$OUT" "$secret" "secret in webhooks get output"; fi

  run webhooks update "$hid" --events deal.created,deal.deleted --format json --quiet
  assert_eq S04 "webhook update (events merge) exit code" "0" "$RC"
  run webhooks get "$hid" --format json --quiet
  assert_contains S04 "$OUT" "deal.deleted" "updated events visible on get"

  run webhooks create --url "$url" --events deal.created,not-an-event --format json --quiet
  assert_eq S04 "unknown event rejected pre-HTTP (exit 2, zero mutations)" "2" "$RC"
  run webhooks create --url "http://insecure.example.com/hook" --events deal.created --format json --quiet
  assert_eq S04 "plain-http URL rejected pre-HTTP (exit 2)" "2" "$RC"

  run webhooks delete "$hid" --force --quiet
  assert_eq S04 "webhook delete --force exit code" "0" "$RC"
  HOOKS=()
  run webhooks get "$hid" --format json --quiet
  assert_eq S04 "webhook get after delete exits 1" "1" "$RC"
}

# ===========================================================================
# S05 — trash round trip: soft delete -> trash list -> restore -> verify
# ===========================================================================
s05() {
  local name="e2e-full-trash-$TS"
  run orgs create --name "$name" --format json --quiet
  local id; id="$(jqo '.id // empty')"
  [[ -z "$id" ]] && { fail S05 "org create for trash failed"; return 0; }
  ORGS+=("$id")

  run orgs delete "$id" --force --quiet
  assert_eq S05 "soft-delete exit code" "0" "$RC"
  arr_drop ORGS "$id"

  run trash list --type organizations --format json --quiet
  assert_eq S05 "trash list --type organizations exit code" "0" "$RC"
  assert_contains S05 "$OUT" "$id" "trashed org found via trash list"

  run trash restore organizations "$id" --quiet
  assert_eq S05 "trash restore exit code" "0" "$RC"

  run orgs get "$id" --format json --quiet
  assert_eq S05 "restored org fetchable, name matches" "$name" "$(jq -r '.name' <<<"$OUT")"

  # final cleanup delete — leaves ONE harmless trash entry (documented)
  run orgs delete "$id" --force --quiet
  assert_eq S05 "final cleanup delete exit code" "0" "$RC"
}

# ===========================================================================
# S06 — typed custom fields: definitions drive JSON typing
# ===========================================================================
s06() {
  local num="e2e_full_num_$TS" sel="e2e_full_sel_$TS" mul="e2e_full_mul_$TS"

  run custom-fields create --entity-type deals --key "$num" --type number --format json --quiet
  assert_eq S06 "number definition create exit code" "0" "$RC"
  local num_id; num_id="$(jqo '.id // empty')"
  [[ -n "$num_id" ]] && DEFS+=("$num_id")

  run custom-fields create --entity-type orgs --key "$sel" --type select --options "red,green" --format json --quiet
  assert_eq S06 "select definition create exit code" "0" "$RC"
  local sel_id; sel_id="$(jqo '.id // empty')"
  [[ -n "$sel_id" ]] && DEFS+=("$sel_id")

  run custom-fields create --entity-type orgs --key "$mul" --type multi_select --options "a,b,c" --format json --quiet
  assert_eq S06 "multi_select definition create exit code" "0" "$RC"
  local mul_id; mul_id="$(jqo '.id // empty')"
  [[ -n "$mul_id" ]] && DEFS+=("$mul_id")

  run custom-fields get "$num_id" --format json --quiet
  assert_eq S06 "custom-fields get returns the definition" "$num" "$(jq -r '.name' <<<"$OUT")"

  # number write -> JSON number on the wire (never "4")
  [[ -z "$STAGE_ID" ]] && { fail S06 "typed deal write skipped — no stage"; return 0; }
  run deals create --title "e2e-full-typed-$TS" --stage "$STAGE_ID" --custom-field "$num=4" --format json --quiet
  assert_eq S06 "typed number write (deals create) exit code" "0" "$RC"
  local did; did="$(jqo '.id // empty')"
  [[ -n "$did" ]] && DEALS+=("$did")
  run deals get "$did" --format json --quiet
  assert_eq S06 "stored value is JSON number 4" "4" "$(jq -r ".custom_fields.$num" <<<"$OUT")"
  assert_eq S06 "stored value JSON type is number" "number" "$(jq -r ".custom_fields.$num | type" <<<"$OUT")"
  run deals delete "$did" --force --quiet
  arr_drop DEALS "$did"

  # select: invalid option refused PRE-HTTP (exit 2); exclusive flags exit 2
  run orgs create --name "e2e-full-selbad-$TS" --custom-field "$sel=purple" --format json --quiet
  assert_eq S06 "invalid select option refused pre-HTTP (exit 2)" "2" "$RC"
  run orgs create --name "e2e-full-mutex-$TS" --custom-field "$sel=red" --custom-field-json '{}' --format json --quiet
  assert_eq S06 "mutually exclusive custom-field sources rejected (exit 2)" "2" "$RC"
  run orgs create --name "e2e-full-selok-$TS" --custom-field "$sel=red" --format json --quiet
  assert_eq S06 "valid select write exit code" "0" "$RC"
  local oid; oid="$(jqo '.id // empty')"
  [[ -n "$oid" ]] && ORGS+=("$oid")
  run orgs get "$oid" --format json --quiet
  assert_eq S06 "stored select value round trip" "red" "$(jq -r ".custom_fields.$sel" <<<"$OUT")"
  run orgs delete "$oid" --force --quiet
  arr_drop ORGS "$oid"

  # multi_select: comma-split into a JSON array
  run orgs create --name "e2e-full-mul-$TS" --custom-field "$mul=a,c" --format json --quiet
  assert_eq S06 "multi_select write exit code" "0" "$RC"
  oid="$(jqo '.id // empty')"
  [[ -n "$oid" ]] && ORGS+=("$oid")
  run orgs get "$oid" --format json --quiet
  assert_eq S06 "multi_select stored as JSON array" '["a","c"]' "$(jq -c ".custom_fields.$mul" <<<"$OUT")"
  run orgs delete "$oid" --force --quiet
  arr_drop ORGS "$oid"

  # --custom-field-json writes verbatim (bypasses inference)
  run orgs create --name "e2e-full-verbatim-$TS" --custom-field-json "{\"$num\": 9}" --format json --quiet
  assert_eq S06 "--custom-field-json verbatim write exit code" "0" "$RC"
  oid="$(jqo '.id // empty')"
  [[ -n "$oid" ]] && ORGS+=("$oid")
  run orgs get "$oid" --format json --quiet
  assert_eq S06 "verbatim JSON value visible" "9" "$(jq -r ".custom_fields.$num" <<<"$OUT")"
  run orgs delete "$oid" --force --quiet
  arr_drop ORGS "$oid"
}

# ===========================================================================
# S07 — workflows, runs, templates
# ===========================================================================
s07() {
  local name="e2e-full-wf-$TS"
  local nodes
  nodes="[{\"id\":\"n1\",\"label\":\"touch\",\"type\":\"action\",\"config\":{\"actionType\":\"crm_action\",\"entity\":\"person\",\"operation\":\"update\",\"targetId\":\"$WF_PERSON_ID\",\"fieldMapping\":{\"notes\":\"e2e-full run $TS\"}},\"nextNodeId\":null}]"
  run workflows create --name "$name" --description "e2e-full suite" --triggers '[{"type":"manual"}]' --nodes "$nodes" --format json --quiet
  assert_eq S07 "workflow create exit code" "0" "$RC"
  local wid; wid="$(jqo '.id // empty')"
  [[ -z "$wid" ]] && { fail S07 "workflow create returned no id"; return 0; }
  WFS+=("$wid")

  run workflows get "$wid" --format json --quiet
  assert_eq S07 "workflows create leaves workflow INACTIVE" "false" "$(jq -r '.active' <<<"$OUT")"

  run workflows trigger "$wid" --format json --quiet
  assert_eq S07 "trigger on inactive workflow exits 1 (HTTP 409)" "1" "$RC"
  assert_contains S07 "$ERR" "hint:" "inactive-trigger error carries a hint"

  run workflows update "$wid" --active true --format json --quiet
  assert_eq S07 "workflows update --active true exit code" "0" "$RC"

  run workflows trigger "$wid" --format json --quiet
  assert_eq S07 "trigger on active workflow exit code" "0" "$RC"
  local rid; rid="$(jq -r '.run_id // .id // empty' <<<"$OUT")"
  [[ -z "$rid" ]] && { fail S07 "trigger returned no run id"; return 0; }

  run workflows runs list --workflow "$wid" --format json --quiet
  assert_eq S07 "runs list exit code" "0" "$RC"
  assert_contains S07 "$OUT" "$rid" "fired run appears in runs list"

  run workflows runs get "$rid" --workflow "$wid" --watch --exit-status --format json --quiet
  assert_eq S07 "runs get --watch --exit-status (run completed) exit code" "0" "$RC"
  assert_contains S07 "$OUT" "completed" "run reached terminal completed state"

  run templates create --name "e2e-full-tpl-$TS" --workflow "$wid" --format json --quiet
  assert_eq S07 "templates create exit code" "0" "$RC"
  local tid; tid="$(jqo '.id // empty')"
  [[ -n "$tid" ]] && TPLS+=("$tid")

  run templates get "$tid" --format json --quiet
  assert_eq S07 "template carries the snapshot trigger" "true" "$(jq -r '.trigger != null' <<<"$OUT")"
  assert_eq S07 "template carries the snapshot nodes" "1" "$(jq -r '.nodes | length' <<<"$OUT")"

  run templates update "$tid" --name x --format json --quiet
  assert_eq S07 "templates update does not exist (exit 2)" "2" "$RC"

  run templates list --format json --quiet
  assert_contains S07 "$OUT" "$tid" "template appears in templates list"

  run templates delete "$tid" --force --quiet
  assert_eq S07 "template delete --force exit code" "0" "$RC"
  TPLS=()

  # a workflow with NO runs deletes cleanly...
  run workflows create --name "e2e-full-wf-noruns-$TS" --triggers '[{"type":"manual"}]' --nodes "$nodes" --format json --quiet
  local wid2; wid2="$(jqo '.id // empty')"
  [[ -n "$wid2" ]] && WFS+=("$wid2")
  run workflows delete "$wid2" --force --quiet
  assert_eq S07 "workflow WITHOUT runs deletes cleanly (exit 0)" "0" "$RC"
  arr_drop WFS "$wid2"

  # ...but one WITH run history 500s server-side (known upstream bug, NOT a
  # CLI bug — verified with a raw DELETE against /api/v1/workflows/<id>).
  # The CLI-side contract asserted here: the failure is reported cleanly
  # (exit 1, hint line present) instead of crashing or lying about success.
  run workflows delete "$wid" --force --quiet
  if [[ "$RC" == "0" ]]; then
    pass S07 "workflow WITH runs deleted (server bug fixed upstream?)"
    arr_drop WFS "$wid"
  else
    assert_eq S07 "workflow WITH runs: delete fails cleanly (exit 1)" "1" "$RC"
    assert_contains S07 "$ERR" "hint:" "workflow-with-runs delete failure carries a hint (server 500 — known upstream bug; workflow $wid left behind)"
  fi
}

# ===========================================================================
# S08 — batch operations and the exit-code contract
# ===========================================================================
s08() {
  run_stdin '[{"name":"e2e-full-b1-'"$TS"'"},{"name":"e2e-full-b2-'"$TS"'"}]' orgs create --stdin --format json --quiet
  assert_eq S08 "batch create --stdin (orgs) exit code" "0" "$RC"
  local id_a id_b
  id_a="$(jq -r '.[0].id // empty' <<<"$OUT")"
  id_b="$(jq -r '.[1].id // empty' <<<"$OUT")"
  [[ -z "$id_a" || -z "$id_b" ]] && { fail S08 "batch create did not return both ids"; return 0; }
  ORGS+=("$id_a" "$id_b")

  # all-ok batch update under --quiet: exit 0, silent data path
  run_stdin "[{\"id\":\"$id_a\",\"name\":\"e2e-full-ok1-$TS\"},{\"id\":\"$id_b\",\"name\":\"e2e-full-ok2-$TS\"}]" orgs update --stdin --quiet
  assert_eq S08 "all-ok batch update under --quiet exit code" "0" "$RC"

  # structural rejections happen BEFORE any HTTP (exit 2)
  run_stdin 'not json at all' orgs update --stdin --quiet
  assert_eq S08 "malformed stdin exit code (pre-HTTP)" "2" "$RC"
  run_stdin '{"id":"x","name":"single object"}' orgs update --stdin --quiet
  assert_eq S08 "single-object stdin rejected (array required, exit 2)" "2" "$RC"
  run_stdin '{"id":"x","name":"ndjson line"}
{"id":"y","name":"second line"}' orgs update --stdin --quiet
  assert_eq S08 "NDJSON rejected (array only, exit 2)" "2" "$RC"

  # mixed batch: per-item failure never aborts the run; summary on stderr;
  # exit 1 iff anything failed; summary survives --quiet
  local bogus="e2e-full-no-such-$TS"
  run_stdin "[{\"id\":\"$id_a\",\"name\":\"e2e-full-mix1-$TS\"},{\"id\":\"$bogus\",\"name\":\"x\"}]" orgs update --stdin --quiet
  assert_eq S08 "mixed batch (one bogus id) exit code" "1" "$RC"
  assert_contains S08 "$ERR" "1 failed" "failure count in stderr summary"
  assert_contains S08 "$ERR" "$bogus" "failed id named on stderr"
  run orgs get "$id_a" --format json --quiet
  assert_eq S08 "good item in mixed batch was still mutated" "e2e-full-mix1-$TS" "$(jq -r '.name' <<<"$OUT")"

  # batch delete via positional ids and via --stdin array
  run orgs delete "$id_a" "$id_b" --force --quiet
  assert_eq S08 "batch delete positional exit code" "0" "$RC"
  arr_drop ORGS "$id_a"; arr_drop ORGS "$id_b"
  run orgs get "$id_a" --format json --quiet
  assert_eq S08 "batch-deleted org is gone" "1" "$RC"

  run_stdin '["'"$bogus"'"]' orgs delete --stdin --force --quiet
  assert_eq S08 "batch delete of nonexistent id exits 1 (per-item failure)" "1" "$RC"
}

# ===========================================================================
# S09 — pagination, projection, expand
# ===========================================================================
s09() {
  run deals list --limit 5 --format json --quiet
  assert_eq S09 "deals list --limit 5 exit code" "0" "$RC"
  assert_eq S09 "--limit 5 returns exactly 5 items" "5" "$(jq 'length' <<<"$OUT")"

  local first_page_first offset_first
  first_page_first="$(jq -r '.[0].id' <<<"$OUT")"
  run deals list --limit 5 --offset 5 --format json --quiet
  offset_first="$(jq -r '.[0].id // empty' <<<"$OUT")"
  if [[ -n "$offset_first" && "$offset_first" != "$first_page_first" ]]; then
    pass S09 "--offset 5 starts at a different record"
  else
    fail S09 "--offset 5 — expected first id [$first_page_first] to change, observed [$offset_first]"
  fi

  run deals get "$DEAL_ID" --fields id,title --format json --quiet
  assert_eq S09 "--fields id,title projects exactly those keys" '["id","title"]' "$(jq -c 'keys_unsorted' <<<"$OUT")"

  run deals get "$DEAL_ID" --expand organization --format json --quiet
  assert_eq S09 "--expand organization exit code (payload presence is server-dependent)" "0" "$RC"

  run pipelines list --all --format json --quiet
  assert_eq S09 "pipelines list --all exit code" "0" "$RC"
  assert_not_contains S09 "$ERR" "server ceiling" "no ceiling warning when under the cap"

  run deals list --all --format json --quiet
  assert_eq S09 "deals list --all exit code" "0" "$RC"
  assert_eq S09 "--all stops at the 1000-record ceiling" "1000" "$(jq 'length' <<<"$OUT")"
  assert_contains S09 "$ERR" "server ceiling" "ceiling warning printed to stderr"
}

# ===========================================================================
# S10 — output formats and global flags
# ===========================================================================
s10() {
  local anchor
  anchor="$(last_nonempty ORGS "$ORG_ID")"

  for fmt in json table csv plain; do
    run orgs get "$anchor" --format "$fmt" --quiet
    assert_eq S10 "org get --format $fmt exit code" "0" "$RC"
  done
  run orgs get "$anchor" --format json --quiet
  assert_eq S10 "json output parses as an object" "true" "$(jq -r 'has("id")' <<<"$OUT")"
  run orgs get "$anchor" --format csv --quiet
  assert_eq S10 "csv output has a header line" "true" "$([[ $(wc -l <<<"$OUT") -ge 2 ]] && echo true || echo false)"
  run orgs get "$anchor" --format table --no-color --quiet
  assert_not_contains S10 "$OUT" $'\033[' "no ANSI escapes under --no-color"

  # data on stdout, diagnostics on stderr: a successful command keeps stderr clean
  run orgs get "$anchor" --format json
  assert_eq S10 "successful command writes nothing to stderr" "" "$ERR"

  # --dry-run previews without touching the server
  run orgs create --name "e2e-full-dry-$TS" --dry-run --format json --quiet
  assert_eq S10 "--dry-run exit code" "0" "$RC"
  run orgs list --limit 100 --format json --quiet
  assert_not_contains S10 "$OUT" "e2e-full-dry-$TS" "--dry-run created nothing"

  # Non-TTY delete WITHOUT --force must refuse (exit 1, zero HTTP) per the
  # documented contract — use a THROWAWAY record so a contract violation
  # cannot cascade into later scenarios. Currently the single-positional-ID
  # path has NO gate at all (v1.0 flow), so this documents the drift.
  run orgs create --name "e2e-full-gate-$TS" --format json --quiet
  local gid; gid="$(jqo '.id // empty')"
  [[ -z "$gid" ]] && { fail S10 "gate-test org create failed"; return 0; }
  ORGS+=("$gid")
  run_nostdin orgs delete "$gid"
  if [[ "$RC" == "1" ]]; then
    pass S10 "non-TTY delete without --force refused (exit 1)"
    run orgs get "$gid" --format json --quiet
    assert_eq S10 "refused delete left the record intact" "0" "$RC"
    run orgs delete "$gid" --force --quiet
    arr_drop ORGS "$gid"
  elif [[ "$RC" == "0" ]]; then
    # the bypass really deleted it — nothing left to clean up
    arr_drop ORGS "$gid"
    fail S10 "non-TTY delete without --force was NOT refused (exit 0) — single-ID deletes bypass the documented --force gate; record $gid was really deleted"
  else
    fail S10 "non-TTY delete without --force: unexpected exit $RC (record $gid left for cleanup): $(head -c 160 <<<"$ERR")"
  fi
}

# ===========================================================================
# S11 — exit-code contract and error UX
# ===========================================================================
s11() {
  run deals get
  assert_eq S11 "missing required id exits 2 (pre-HTTP)" "2" "$RC"

  run custom-fields create --entity-type orgs --key x --type bogus --quiet
  assert_eq S11 "bad enum value exits 2 (pre-HTTP)" "2" "$RC"

  run deals create --stage "$STAGE_ID" --quiet
  assert_eq S11 "missing required --title exits 2" "2" "$RC"

  run orgs get "e2e-full-does-not-exist-$TS" --format json --quiet
  assert_eq S11 "get nonexistent exits 1 (runtime)" "1" "$RC"
  assert_contains S11 "$ERR" "hint:" "runtime error carries a hint line"

  # verbose flag exists and does not corrupt the data path
  run orgs get "$(last_nonempty ORGS "$ORG_ID")" --verbose --format json --quiet
  assert_eq S11 "--verbose keeps exit 0" "0" "$RC"
}

# ===========================================================================
# S12 — audit log (admin-gated; this suite requires an admin key)
# ===========================================================================
s12() {
  run audit list --limit 3 --format json --quiet
  assert_eq S12 "audit list exit code" "0" "$RC"
  assert_eq S12 "audit list --limit 3 returns at most 3 entries" "true" "$(jq -r 'length <= 3' <<<"$OUT")"
  assert_eq S12 "entries carry action + actor_kind" "true" "$(jq -r 'all(.[]; has("action") and has("actor_kind"))' <<<"$OUT")"

  run audit list --entity-type deal --limit 3 --format json --quiet
  assert_eq S12 "audit list --entity-type deal exit code" "0" "$RC"

  run audit list --limit 3
  assert_eq S12 "audit list non-json format exit code" "0" "$RC"
}

# ===========================================================================
# S13 — config file round trip + cache refresh/clear (fully isolated HOME)
#   The real ~/.pipelite is NEVER touched: PIPELITE_CONFIG + HOME both point
#   into the scratch dir. The temp config holds env creds (chmod 600, wiped
#   by the scratch cleanup) — they are never echoed into results.
# ===========================================================================
s13() {
  local iso="$SCRATCH/iso"
  mkdir -p "$iso"
  local cfg="$iso/config.toml"
  umask 077
  printf '[server]\nurl = "%s"\napi_key = "%s"\n[output]\nformat = "json"\n' \
    "$PIPELITE_SERVER_URL" "$PIPELITE_API_KEY" > "$cfg"
  umask 022

  env PIPELITE_CONFIG="$cfg" "$BIN" config set server.url "$PIPELITE_SERVER_URL" >/dev/null 2>&1 \
    && pass S13 "config set on isolated config exits 0" \
    || fail S13 "config set on isolated config failed"
  local got
  got="$(env PIPELITE_CONFIG="$cfg" "$BIN" config get server.url 2>/dev/null | tail -1)"
  assert_eq S13 "config set/get round trip" "$PIPELITE_SERVER_URL" "$got"
  assert_eq S13 "config file written with 0600 permissions" "600" "$(stat -c '%a' "$cfg")"

  # cache: refresh pre-populates from the server, clear wipes — under $HOME=$iso
  env HOME="$iso" PIPELITE_CONFIG="$cfg" "$BIN" cache refresh >/dev/null 2>&1 \
    && pass S13 "cache refresh exits 0" \
    || fail S13 "cache refresh failed"
  local n
  n="$(find "$iso/.pipelite/cache" -type f 2>/dev/null | wc -l | tr -d ' ')"
  if [[ "$n" -gt 0 ]]; then pass S13 "cache refresh populated $n cache file(s)"; else fail S13 "cache refresh populated no files"; fi
  env HOME="$iso" PIPELITE_CONFIG="$cfg" "$BIN" cache clear >/dev/null 2>&1 \
    && pass S13 "cache clear exits 0" \
    || fail S13 "cache clear failed"
  n="$(find "$iso/.pipelite/cache" -type f 2>/dev/null | wc -l | tr -d ' ')"
  assert_eq S13 "cache clear wiped the cache dir" "0" "$n"
}

# ===========================================================================
# S14 — dashboard (fetches EVERY deal — slow on big datasets)
# ===========================================================================
s14() {
  if [[ "${E2E_SKIP_DASHBOARD:-0}" == "1" ]]; then
    echo "SKIP S14: dashboard (E2E_SKIP_DASHBOARD=1)" | tee -a "$RESULTS_FILE"
    return 0
  fi
  # ~25k deals => the full scan is hundreds of paginated requests; allow up
  # to 5 minutes. (Known CLI bug this test documents: the server caps pages
  # at 100 items but the dashboard advances offset by its REQUESTED limit of
  # 500, so once offset passes the row count it fetches empty pages forever
  # — an infinite loop.)
  if timeout 300 "$BIN" dashboard --format json --quiet >"$SCRATCH/dash.json" 2>"$SCRATCH/dash.err"; then
    pass S14 "dashboard exit code (full scan)"
    assert_eq S14 "dashboard JSON parses with a pipelines array" "true" \
      "$(jq -r '.pipelines | type == "array"' "$SCRATCH/dash.json" 2>/dev/null || echo false)"
    assert_eq S14 "dashboard carries a workflows summary" "true" \
      "$(jq -r '.workflows | has("total")' "$SCRATCH/dash.json" 2>/dev/null || echo false)"
  else
    fail S14 "dashboard did not complete within 300s — known CLI bug: fetch_all_deals advances offset by the requested limit (500) while the server caps pages at 100 items, so the loop never reaches meta.total and spins on empty pages forever"
  fi
}

# ---------------------------------------------------------------------------
# Run everything.
# ---------------------------------------------------------------------------
s01
s02_orgs
s02_people
s02_deals
s02_activities
s02_pipelines_stages
s03
s04
s05
s06
s07
s08
s09
s10
s11
s12
s13
s14

echo "" | tee -a "$RESULTS_FILE"
echo "== Summary: $PASSCount PASS, $FAILCount FAIL ==" | tee -a "$RESULTS_FILE"
echo "Results appended to $RESULTS_FILE (local scratch — do not commit)." | tee -a "$RESULTS_FILE"

if [[ $FAILCount -gt 0 ]]; then exit 1; fi
exit 0
