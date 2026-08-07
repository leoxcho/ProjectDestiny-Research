#!/usr/bin/env bash
set -Eeuo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROTOCOL_DB="${DESTINY_PROTOCOL_DB:-$ROOT_DIR/protocol.db}"
if [[ ! -f "$PROTOCOL_DB" ]]; then echo "protocol database not found: $PROTOCOL_DB" >&2; exit 1; fi
command -v sqlite3 >/dev/null 2>&1 || { echo "sqlite3 is required" >&2; exit 1; }
sqlite3 -separator $'\t' "$PROTOCOL_DB" "SELECT coalesce((SELECT id FROM sessions ORDER BY last_timestamp DESC LIMIT 1), 'none'), coalesce((SELECT packet_count FROM sessions ORDER BY last_timestamp DESC LIMIT 1), 0);" | while IFS=$'\t' read -r session packets; do
  echo "session ID: $session"
  echo "packet count: $packets"
  echo "capture location: $PROTOCOL_DB (messages and packet_samples)"
done
