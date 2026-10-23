#!/bin/bash
# streaming-stall (a): the streaming test `runs` times, under `burn`
# processes of `yes > /dev/null` of this run's own (0: none started on
# purpose), each run's output, the relay's and doors' log and the host's load
# kept under data/<cell>/<n>.
#
#   run.sh <cell> alone|suite <burn> <first run> <last run>
#
# `alone` is the streaming test by itself, both engines at once
# (`streaming_both_ways`); `suite` is the whole browser suite, as
# scripts/check.sh's "browser" step runs it. Each `yes` dies on its own after
# 40 minutes (an alarm, kept across exec), and is killed when this script
# ends, so no load outlives it. A burned run waits a minute first, so
# uptime's 1-minute average has risen to the load. Adapted from
# demo/runs/stream/2026-10-15/run.sh.
set -u
cell=$1 what=$2 burn=$3 first=$4 last=$5
here=$(cd "$(dirname "$0")" && pwd)
tree=$(git -C "$here" rev-parse --show-toplevel)
out="$here/data/$cell"
mkdir -p "$out"
pids=()
stop() { [ ${#pids[@]} -gt 0 ] && kill "${pids[@]}" 2>/dev/null; }
trap stop EXIT INT TERM HUP
for _ in $(seq 1 "$burn"); do
  perl -e 'alarm 2400; exec "yes"' >/dev/null &
  pids+=($!)
done
[ "$burn" -gt 0 ] && sleep 60
export CARGO_INCREMENTAL=0
ulimit -Sn 10240 2>/dev/null
case $what in
  alone) filter=(-- streaming_both_ways --nocapture) ;;
  suite) filter=(-- --nocapture) ;;
  *) echo "alone or suite, not $what" >&2; exit 2 ;;
esac
for n in $(seq "$first" "$last"); do
  {
    echo "tree: $tree at $(git -C "$tree" rev-parse --short HEAD), $what, burn: $burn"
    echo "start: $(date -u +%FT%TZ) $(uptime)"
  } >"$out/$n.load"
  (cd "$tree" && cargo test -q -p svrngs-node --test browser "${filter[@]}") >"$out/$n.out" 2>&1
  echo "exit: $?" >>"$out/$n.load"
  echo "end: $(date -u +%FT%TZ) $(uptime)" >>"$out/$n.load"
  cp "$tree/target/traces/browser.log" "$out/$n.relay.log"
  echo "$cell $n: $(grep -h '^test result' "$out/$n.out")"
done
