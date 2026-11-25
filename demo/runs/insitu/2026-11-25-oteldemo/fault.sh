#!/usr/bin/env bash
# One fault, as PROTOCOL.md has it: reset, 3 minutes clean, the flag and
# emitRawPii on, 4 minutes for it to show. Prints the times.
#   ./fault.sh <flag> <variant>     |   ./fault.sh reset
set -euo pipefail
cd "$(dirname "$0")"
python3 -I flags.py reset >/dev/null
[ "$1" = reset ] && { echo "reset $(date -u +%T)"; exit 0; }
echo "reset $(date -u +%T); clean for 180 s"; sleep 180
python3 -I flags.py set emitRawPii on >/dev/null
python3 -I flags.py set "$1" "$2"
echo "on $(date -u +%T); showing for 240 s"; sleep 240
echo "ready $(date -u +%T)"
