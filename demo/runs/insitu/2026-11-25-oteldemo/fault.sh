#!/usr/bin/env bash
# One fault, as PROTOCOL.md has it: reset, 3 minutes clean, the flag and
# emitRawPii on, 4 minutes for it to show. Prints the times.
#   ./fault.sh <flag> <variant>     |   ./fault.sh reset
set -euo pipefail
cd "$(dirname "$0")"
python3 -I flags.py reset >/dev/null
[ "$1" = reset ] && { echo "reset $(date -u +%T)"; exit 0; }
# Fresh views at the reset, so a fault's window holds no earlier fault
# (F3, 2026-11-25: R diagnosed F2's leftover errors). Each restart is
# approved again, as the share binding asks.
B=/home/alexbryan/dev/svrngs/target/release/svrngs
for arm in P R; do
  port=$([ $arm = P ] && echo 8721 || echo 8722); lend=$([ $arm = P ] && echo site-view || echo site-raw)
  kill "$(cat out/view$arm.pid)" 2>/dev/null || true; sleep 1
  node siteview.mjs out/site/raw.jsonl --port $port --fresh $([ $arm = R ] && echo --raw) > out/view$arm.log 2>&1 & echo $! > out/view$arm.pid
  sleep 2; HOME=$PWD/out/site-node/home SVRNGS_DIR=$PWD/out/site-node/data python3 -I approve.py $B $lend >/dev/null
done
echo "reset $(date -u +%T); clean for 180 s"; sleep 180
python3 -I flags.py set emitRawPii on >/dev/null
python3 -I flags.py set "$1" "$2"
echo "on $(date -u +%T); showing for 240 s"; sleep 240
echo "ready $(date -u +%T)"
