#!/usr/bin/env bash
# scripts/e2e-v1.1.sh — Live-server E2E for the four ROADMAP SC-2 cross-cutting checks.
#
# EXECUTION CONTRACT (read before running):
#   * Credentials come ONLY from the session environment: export
#       PIPELITE_SERVER_URL and PIPELITE_API_KEY (an ADMIN key)
#     before running. The script refuses to run when either is unset.
#     Credentials are NEVER read from files, never given defaults, never echoed.
#   * Only throwaway records prefixed "e2e-" are created, mutated, and deleted.
#   * "trash purge" is NEVER invoked (permanent, admin-only) — enforced by the
#     pl() guard below, not by discipline.
#   * The trash round-trip leaves ONE harmless trash entry behind (documented
#     in docs/e2e-v1.1-report.md).
#   * Results are printed to stdout AND appended to $E2E_RESULTS_FILE
#     (default: ./e2e-v1.1-results.txt — a local scratch file; transcribe the
#     lines into docs/e2e-v1.1-report.md and do NOT commit the scratch file).
#   * Exit status: 0 when every scenario PASSes, 1 otherwise.
set -euo pipefail

# ---------------------------------------------------------------------------
# Guard 1 — environment credentials, checked FIRST, with a usage message.
# ---------------------------------------------------------------------------
if [[ -z "${PIPELITE_SERVER_URL:-}" || -z "${PIPELITE_API_KEY:-}" ]]; then
  cat >&2 <<'USAGE'
e2e-v1.1.sh: missing credentials.

This script drives REAL mutations against a live Pipelite server and needs
session-environment credentials (never committed, never read from files):

    export PIPELITE_SERVER_URL   (your live server URL)
    export PIPELITE_API_KEY      (an ADMIN key — non-admin keys cannot reach
                                  audit-grade surfaces)

Then re-run:  bash scripts/e2e-v1.1.sh
USAGE
  exit 2
fi

# ---------------------------------------------------------------------------
# Guard 2 — the no-purge / no-secret-echo rules, ENCODED as code:
#   * every pipelite invocation MUST go through pl(), which refuses the
#     permanent-destroy subcommand outright (exit 70);
#   * readonly FORBIDDEN_PATTERNS drive assert_no_purge() and assert_no_secrets(),
#     re-run at the end against both authored artifacts (script + report).
# ---------------------------------------------------------------------------
readonly FORBIDDEN_PATTERNS='trash +purge'

pl() {
  local arg
  for arg in "$@"; do
    if [[ "$arg" =~ ^purge$ ]]; then
      echo "FATAL: 'trash purge' is forbidden in this script (permanent, admin-only)." >&2
      echo "       This guard is part of the E2E safety contract (ROADMAP SC-2)." >&2
      exit 70
    fi
  done
  "$BIN" "$@"
}
readonly -f pl

# Binary resolution: build release once and run that binary directly.
cargo build --release --quiet
readonly BIN="$(pwd)/target/release/pipelite"

RESULTS_FILE="${E2E_RESULTS_FILE:-e2e-v1.1-results.txt}"
: > "$RESULTS_FILE"

# ---------------------------------------------------------------------------
# Assertion helpers — every failure prints observed vs expected.
# ---------------------------------------------------------------------------
PASSCount=0
FAILCount=0
declare -a CREATED_DEALS=() CREATED_ORGS=() CREATED_WEBHOOKS=() CREATED_DEFS=()

pass() { # pass <scenario> <evidence>
  local line="PASS $1: $2"
  echo "$line"; echo "$line" >> "$RESULTS_FILE"; PASSCount=$((PASSCount+1))
}
fail() { # fail <scenario> <evidence>
  local line="FAIL $1: $2"
  echo "$line" >&2; echo "$line" >> "$RESULTS_FILE"; FAILCount=$((FAILCount+1))
}
assert_eq() { # assert_eq <scenario> <what> <expected> <observed>
  if [[ "$4" == "$3" ]]; then pass "$1" "$2 = $3"; else fail "$1" "$2 — expected [$3], observed [$4]"; fi
}
assert_contains() { # assert_contains <scenario> <haystack> <needle> <what>
  if grep -q "$3" <<<"$2"; then pass "$1" "$4 present"; else fail "$1" "$4 — expected to contain [$3], observed [$(head -c 400 <<<"$2")]"; fi
}
assert_not_contains() { # assert_not_contains <scenario> <haystack> <needle> <what>
  if grep -q "$3" <<<"$2"; then fail "$1" "$4 — must NOT contain [$3] but it does"; else pass "$1" "$4 absent (as required)"; fi
}

assert_no_purge() { # the authored script must not invoke the permanent-destroy subcommand
  # (character classes keep this pattern from matching its own source line)
  if grep -qE 'pl[[:space:]]+trash[[:space:]]+purge|BIN[[:space:]]+trash[[:space:]]+purge' "$0"; then
    fail guard "script text references trash purge"
  else
    pass guard "no 'trash purge' invocation anywhere in script (grep-proven)"
  fi
}
assert_no_secrets() { # neither the API key nor the webhook secret may appear in the artifacts
  local leaked=0
  if grep -q "$PIPELITE_API_KEY" "$0" "$RESULTS_FILE" 2>/dev/null; then leaked=1; fi
  if [[ -n "${WEBHOOK_SECRET:-}" ]] && grep -q "$WEBHOOK_SECRET" "$RESULTS_FILE" 2>/dev/null; then leaked=1; fi
  if [[ $leaked -eq 1 ]]; then fail guard "credential-shaped literal found in script/results"; else pass guard "no credentials in script or results (grep-proven)"; fi
}
readonly -f assert_eq assert_contains assert_not_contains assert_no_purge assert_no_secrets

TS="$(date +%s)"
STAGE_ID=""

cleanup() { # best-effort; never purge, never fail the run on cleanup errors
  local id
  for id in ${CREATED_DEALS[@]+"${CREATED_DEALS[@]}"}; do "$BIN" deals delete "$id" --force >/dev/null 2>&1 || true; done
  for id in ${CREATED_ORGS[@]+"${CREATED_ORGS[@]}"}; do "$BIN" orgs delete "$id" --force >/dev/null 2>&1 || true; done
  for id in ${CREATED_WEBHOOKS[@]+"${CREATED_WEBHOOKS[@]}"}; do "$BIN" webhooks delete "$id" --force >/dev/null 2>&1 || true; done
  for id in ${CREATED_DEFS[@]+"${CREATED_DEFS[@]}"}; do "$BIN" custom-fields delete "$id" --force >/dev/null 2>&1 || true; done
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Preflight — connectivity + a stage to hang test deals on.
# ---------------------------------------------------------------------------
echo "== pipelite v1.1 live E2E — $(date -u +%Y-%m-%dT%H:%M:%SZ) ==" | tee -a "$RESULTS_FILE"

STAGE_ID="$("$BIN" stages list --format json --quiet 2>/dev/null | jq -r '.[0].id // empty' || true)"
if [[ -z "$STAGE_ID" ]]; then
  fail preflight "no stage found via 'stages list' — cannot create throwaway deals (server has no stages?)"
fi

# ---------------------------------------------------------------------------
# Scenario A: batch_exit_codes_quiet
#   (1) all-ok batch update under --quiet -> exit 0
#       NOTE: the locked Phase 7 contract prints the "N ok, M failed" summary on
#       the FAILURE path only; an all-ok batch under --quiet is silent. Pinned
#       by tests/contract_matrix_test.rs (61/61 green) — asserted accordingly.
#   (2) malformed stdin -> exit 2 BEFORE any HTTP
#   (3) mixed batch (one bogus ID) -> exit 1 AND "1/2 deal updated, 1 failed"
#       summary on stderr (per-item failures never abort the batch; there is no
#       --continue-on-error flag — continuing is the built-in behavior).
# ---------------------------------------------------------------------------
batch_exit_codes_quiet() {
  local out rc
  if [[ -z "$STAGE_ID" ]]; then fail batch_exit_codes_quiet "skipped — no stage id from preflight"; return 0; fi

  out="$("$BIN" deals create --stdin --format json --quiet <<J2
[{"title":"e2e-batch-a-$TS","stage_id":"$STAGE_ID"},{"title":"e2e-batch-b-$TS","stage_id":"$STAGE_ID"}]
J2
)" || { fail batch_exit_codes_quiet "batch create failed: $out"; return 0; }
  local id_a id_b
  id_a="$(jq -r '.[0].id' <<<"$out")"; id_b="$(jq -r '.[1].id' <<<"$out")"
  CREATED_DEALS+=("$id_a" "$id_b")

  set +e
  out="$(printf '[{"id":"%s","title":"e2e-ok-%s"},{"id":"%s","title":"e2e-ok-%s"}]\n' \
    "$id_a" "$TS" "$id_b" "$TS" \
    | "$BIN" deals update --stdin --quiet 2>&1)"
  rc=$?
  set -e
  assert_eq batch_exit_codes_quiet "all-ok batch update under --quiet exit code" "0" "$rc"

  set +e
  out="$(printf 'not json at all\n' | "$BIN" deals update --stdin 2>&1)"
  rc=$?
  set -e
  assert_eq batch_exit_codes_quiet "malformed stdin exit code (pre-HTTP)" "2" "$rc"

  local bogus="e2e-no-such-deal-$TS"
  set +e
  out="$(printf '[{"id":"%s","title":"x"},{"id":"%s","title":"x"}]\n' \
    "$id_a" "$bogus" \
    | "$BIN" deals update --stdin --quiet 2>&1)"
  rc=$?
  set -e
  assert_eq batch_exit_codes_quiet "mixed batch (one bogus ID) exit code" "1" "$rc"
  assert_contains batch_exit_codes_quiet "$out" "1/2" "per-item summary line"
  assert_contains batch_exit_codes_quiet "$out" "1 failed" "failure count in summary"

  set +e
  out="$("$BIN" deals delete "$id_a" "$id_b" --force --quiet 2>&1)"; rc=$?
  set -e
  assert_eq batch_exit_codes_quiet "batch delete --force cleanup exit code" "0" "$rc"
  CREATED_DEALS=()
}

# ---------------------------------------------------------------------------
# Scenario B: webhook_show_once_secret
#   The signing secret must appear EXACTLY ONCE (create output, full 64 chars),
#   and NEVER in list/get output. The value itself is REDACTED everywhere —
#   only its length and the shown-once yes/no are recorded.
# ---------------------------------------------------------------------------
webhook_show_once_secret() {
  local out rc secret id list_out get_out occurrences
  set +e
  out="$("$BIN" webhooks create --url "https://example.com/e2e-hook-$TS" --events deal.created --format json 2>&1)"
  rc=$?
  set -e
  if [[ $rc -ne 0 ]]; then fail webhook_show_once_secret "create failed: $(head -c 200 <<<"$out")"; return 0; fi

  local out_json
  out_json="$(awk '/^\{/{f=1} f{print} /^\}/{exit}' <<<"$out")"   # exactly the JSON object (warning may precede or trail via 2>&1)
  secret="$(jq -r '.secret // empty' <<<"$out_json")"
  id="$(jq -r '.id // empty' <<<"$out_json")"
  CREATED_WEBHOOKS+=("$id")

  if [[ -z "$secret" ]]; then
    fail webhook_show_once_secret "no secret line found after the 'Signing secret' warning"
  else
    assert_eq webhook_show_once_secret "secret length (recorded, value REDACTED)" "64" "${#secret}"
    occurrences="$(grep -o -- "$secret" <<<"$out" | wc -l | tr -d ' ')"
    assert_eq webhook_show_once_secret "secret occurrences in create output (shown exactly once)" "1" "$occurrences"
  fi

  list_out="$("$BIN" webhooks list 2>&1)"
  assert_not_contains webhook_show_once_secret "$list_out" "$secret" "secret in webhooks list output"
  get_out="$("$BIN" webhooks get "$id" 2>&1)"
  assert_not_contains webhook_show_once_secret "$get_out" "$secret" "secret in webhooks get output"

  set +e
  out="$("$BIN" webhooks delete "$id" --force 2>&1)"; rc=$?
  set -e
  assert_eq webhook_show_once_secret "webhook delete --force exit code" "0" "$rc"
  CREATED_WEBHOOKS=()
}

# ---------------------------------------------------------------------------
# Scenario C: trash_round_trip
#   create org -> delete (soft) -> trash list --type orgs finds it -> restore
#   -> record fetchable again. trash purge is NEVER called; the cleanup delete
#   leaves ONE harmless trash entry (documented in the report).
# ---------------------------------------------------------------------------
trash_round_trip() {
  local out rc name id
  name="e2e-trash-$TS"
  out="$("$BIN" orgs create --name "$name" --format json --quiet 2>&1)" \
    || { fail trash_round_trip "org create failed: $out"; return 0; }
  id="$(jq -r '.id' <<<"$out")"
  CREATED_ORGS+=("$id")

  set +e
  out="$("$BIN" orgs delete "$id" --force 2>&1)"; rc=$?
  set -e
  assert_eq trash_round_trip "soft-delete exit code" "0" "$rc"

  out="$("$BIN" trash list --type orgs --format json 2>&1)"
  assert_contains trash_round_trip "$out" "$id" "trashed org found via trash list --type orgs"

  set +e
  out="$("$BIN" trash restore organizations "$id" 2>&1)"; rc=$?   # PLURAL tab token in URL
  set -e
  assert_eq trash_round_trip "trash restore exit code" "0" "$rc"

  out="$("$BIN" orgs get "$id" --format json 2>&1)"
  assert_eq trash_round_trip "restored org fetchable, name matches" "$name" "$(jq -r '.name' <<<"$out")"

  set +e
  out="$("$BIN" orgs delete "$id" --force 2>&1)"; rc=$?   # re-delete -> the documented leftover trash entry
  set -e
  CREATED_ORGS=()
}

# ---------------------------------------------------------------------------
# Scenario D: typed_write_server_side  (closes the Phase 12 deferred item)
#   definition e2e_price (number) on orgs -> org create --custom-field
#   e2e_price=4 -> read back JSON -> the field must be JSON NUMBER 4 (never "4").
# ---------------------------------------------------------------------------
typed_write_server_side() {
  local out rc def_id org_id
  def_id="$("$BIN" custom-fields list --entity-type orgs --format json --quiet 2>/dev/null \
    | jq -r '.[] | select(.name=="e2e_price") | .id' | head -1 || true)"

  if [[ -z "$def_id" ]]; then
    out="$("$BIN" custom-fields create --entity-type orgs --key e2e_price --type number --format json 2>&1)" \
      || { fail typed_write_server_side "definition create failed: $(head -c 200 <<<"$out")"; return 0; }
    def_id="$(jq -r '.id // .data.id' <<<"$out")"
    CREATED_DEFS+=("$def_id")
    pass typed_write_server_side "definition e2e_price (number, orgs) created ($def_id)"
  else
    pass typed_write_server_side "definition e2e_price already exists ($def_id) — reusing"
  fi

  out="$("$BIN" orgs create --name "e2e-typed-write-$TS" --custom-field e2e_price=4 --format json 2>&1)" \
    || { fail typed_write_server_side "org create with typed field failed: $(head -c 200 <<<"$out")"; return 0; }
  org_id="$(jq -r '.id' <<<"$out")"
  CREATED_ORGS+=("$org_id")

  out="$("$BIN" orgs get "$org_id" --format json 2>&1)"
  assert_eq typed_write_server_side "server-side value (jq .custom_fields.e2e_price)" "4" "$(jq -r '.custom_fields.e2e_price' <<<"$out")"
  assert_eq typed_write_server_side "JSON type of the stored value (number, not string)" "number" "$(jq -r '.custom_fields.e2e_price | type' <<<"$out")"

  set +e
  "$BIN" orgs delete "$org_id" --force >/dev/null 2>&1 || true
  "$BIN" custom-fields delete "$def_id" --force >/dev/null 2>&1 || true
  set -e
  CREATED_ORGS=(); CREATED_DEFS=()
}

# ---------------------------------------------------------------------------
# Run everything.
# ---------------------------------------------------------------------------
assert_no_purge
assert_no_secrets
batch_exit_codes_quiet
webhook_show_once_secret
trash_round_trip
typed_write_server_side
assert_no_secrets   # re-check after the run (results file now holds evidence lines)

echo "" | tee -a "$RESULTS_FILE"
echo "== Summary: $PASSCount PASS, $FAILCount FAIL ==" | tee -a "$RESULTS_FILE"
echo "Results appended to $RESULTS_FILE — transcribe into docs/e2e-v1.1-report.md (do not commit the scratch file)." | tee -a "$RESULTS_FILE"

if [[ $FAILCount -gt 0 ]]; then exit 1; fi
exit 0
