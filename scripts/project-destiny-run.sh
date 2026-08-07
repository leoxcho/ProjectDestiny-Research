#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_DIR="${DESTINY_LOG_DIR:-$ROOT_DIR/logs}"
CAPTURE_DIR="${DESTINY_CAPTURE_DIR:-$ROOT_DIR/captures}"
HARNESS_ADDR="${DESTINY_HARNESS_ADDR:-127.0.0.1:39000}"
API_ADDR="${DESTINY_API_ADDR:-127.0.0.1:8080}"
PROTOCOL_DB="${DESTINY_PROTOCOL_DB:-$ROOT_DIR/protocol.db}"
DESTINY_DB="${DESTINY_DB:-$ROOT_DIR/destiny.db}"
WAIT_SECONDS="${DESTINY_WAIT_SECONDS:-30}"

mkdir -p "$LOG_DIR" "$CAPTURE_DIR"
HARNESS_LOG="$LOG_DIR/destiny-rpcs3-harness-$RUN_ID.log"
API_LOG="$LOG_DIR/destiny-definition-api-$RUN_ID.log"
SESSION_CAPTURE="$CAPTURE_DIR/session-$RUN_ID.txt"
HARNESS_PID=""
API_PID=""
BOOTSTRAP_PID=""

cleanup() {
  local status=$?
  trap - EXIT INT TERM
  for pid in "$API_PID" "$HARNESS_PID" "$BOOTSTRAP_PID"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then kill "$pid" 2>/dev/null || true; fi
  done
  wait "$API_PID" "$HARNESS_PID" 2>/dev/null || true
  exit "$status"
}
trap cleanup EXIT INT TERM

cd "$ROOT_DIR"
if [[ "${DESTINY_SKIP_BOOTSTRAP:-0}" != "1" ]]; then
  echo "Starting local DNS/STUN bootstrap"
  python3 "$ROOT_DIR/destiny_local_dns.py" >"$LOG_DIR/destiny-bootstrap-$RUN_ID.log" 2>&1 &
  BOOTSTRAP_PID=$!
  sleep 1
fi
echo "Starting destiny-rpcs3-harness (log: $HARNESS_LOG)"
cargo run --quiet -p destiny-rpcs3-harness -- --listen "$HARNESS_ADDR" --database "$PROTOCOL_DB" >"$HARNESS_LOG" 2>&1 &
HARNESS_PID=$!

echo "Starting destiny-definition-api (log: $API_LOG)"
cargo run --quiet -p destiny-definition-api -- --database "$DESTINY_DB" --listen "$API_ADDR" >"$API_LOG" 2>&1 &
API_PID=$!

deadline=$((SECONDS + WAIT_SECONDS))
while (( SECONDS < deadline )); do
  [[ -f "$PROTOCOL_DB" && -f "$DESTINY_DB" ]] && \
    curl --silent --show-error --fail --max-time 2 "http://$API_ADDR/stats" >/dev/null 2>&1 && \
    (nc -z "${HARNESS_ADDR%:*}" "${HARNESS_ADDR##*:}" >/dev/null 2>&1 || curl --silent --max-time 2 "http://$HARNESS_ADDR" >/dev/null 2>&1) && break
  sleep 1
done

[[ -f "$PROTOCOL_DB" ]] || { echo "protocol.db not found: $PROTOCOL_DB" >&2; exit 1; }
[[ -f "$DESTINY_DB" ]] || { echo "destiny.db not found: $DESTINY_DB" >&2; exit 1; }
curl --silent --show-error --fail --max-time 2 "http://$API_ADDR/stats" >/dev/null || { echo "definition API health check failed" >&2; exit 1; }
nc -z "${HARNESS_ADDR%:*}" "${HARNESS_ADDR##*:}" || { echo "harness port check failed" >&2; exit 1; }

{
  echo "run_id=$RUN_ID"
  echo "protocol_db=$PROTOCOL_DB"
  echo "destiny_db=$DESTINY_DB"
  echo "harness=$HARNESS_ADDR pid=$HARNESS_PID"
  echo "definition_api=$API_ADDR pid=$API_PID"
  echo "harness_log=$HARNESS_LOG"
  echo "api_log=$API_LOG"
  echo "captured_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} | tee "$SESSION_CAPTURE"

echo "Pipeline healthy; services remain running. Press Ctrl-C to stop them."
wait
