#!/bin/bash
# arch-stream (a): the streaming test alone, both engines at once, `runs`
# times in `tree`, under `burn` processes of `yes > /dev/null` (0: none
# started on purpose), each run's output, the relay's log and the host's load
# kept under data/<cell>/<n>.
#
#   run.sh <tree> <cell> <burn> <first run> <last run>
#
# Each `yes` dies on its own after 15 minutes (an alarm, kept across exec),
# and is killed when this script ends, so no load outlives it. A burned run
# waits a minute first, so uptime's 1-minute average has risen to the load.
set -u
tree=$1 cell=$2 burn=$3 first=$4 last=$5
here=$(cd "$(dirname "$0")" && pwd)
out="$here/data/$cell"
mkdir -p "$out"
pids=()
stop() { [ ${#pids[@]} -gt 0 ] && kill "${pids[@]}" 2>/dev/null; }
trap stop EXIT INT TERM HUP
for _ in $(seq 1 "$burn"); do
  perl -e 'alarm 900; exec "yes"' >/dev/null &
  pids+=($!)
done
[ "$burn" -gt 0 ] && sleep 60
export CARGO_INCREMENTAL=0
for n in $(seq "$first" "$last"); do
  {
    echo "tree: $tree at $(git -C "$tree" rev-parse --short HEAD), burn: $burn"
    echo "start: $(date -u +%FT%TZ) $(uptime)"
  } >"$out/$n.load"
  (cd "$tree" && cargo test -q -p svrngs-node --test browser -- streaming_both_ways --nocapture) >"$out/$n.out" 2>&1
  echo "exit: $?" >>"$out/$n.load"
  echo "end: $(date -u +%FT%TZ) $(uptime)" >>"$out/$n.load"
  cp "$tree/target/traces/browser.log" "$out/$n.relay.log"
  echo "$cell $n: $(grep -h '^test result' "$out/$n.out")"
done
