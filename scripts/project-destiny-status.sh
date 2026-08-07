#!/usr/bin/env bash
set -Eeuo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROTOCOL_DB="${DESTINY_PROTOCOL_DB:-$ROOT_DIR/protocol.db}"
DESTINY_DB="${DESTINY_DB:-$ROOT_DIR/destiny.db}"
HARNESS_ADDR="${DESTINY_HARNESS_ADDR:-127.0.0.1:39000}"
API_ADDR="${DESTINY_API_ADDR:-127.0.0.1:8080}"
CAPTURE_DIR="${DESTINY_CAPTURE_DIR:-$ROOT_DIR/captures}"
LOG_DIR="${DESTINY_LOG_DIR:-$ROOT_DIR/logs}"
echo "database status"
for db in "$PROTOCOL_DB" "$DESTINY_DB"; do [[ -f "$db" ]] && echo "  OK $db" || echo "  MISSING $db"; done
echo "running services"
pgrep -af 'destiny-rpcs3-harness|destiny-definition-api' || echo "  none"
echo "port status"
for addr in "$HARNESS_ADDR" "$API_ADDR"; do host="${addr%:*}"; port="${addr##*:}"; nc -z "$host" "$port" >/dev/null 2>&1 && echo "  OPEN $addr" || echo "  CLOSED $addr"; done
echo "latest capture"
latest_capture="$(find "$CAPTURE_DIR" -type f -name 'session-*.txt' -print 2>/dev/null | sort | tail -1)"
[[ -n "$latest_capture" ]] && echo "  $latest_capture" || echo "  none"
echo "latest protocol events"
if [[ -f "$PROTOCOL_DB" ]] && command -v sqlite3 >/dev/null 2>&1; then
  sqlite3 -header -column "$PROTOCOL_DB" 'SELECT session_id, timestamp, direction, payload_size, confidence, notes FROM messages ORDER BY timestamp DESC LIMIT 10;' || true
else
  echo "  unavailable (database or sqlite3 missing)"
fi
