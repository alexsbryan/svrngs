#!/bin/bash
# measure.sh <label> <runs> <burners> [test filter...], run from the repo:
# the browser suite (or the tests the filters name, --exact) <runs> times,
# with <burners> busy processes (`yes`) beside it for the whole of each run,
# each run's verdicts, its load at the start and end, and the relay's, doors'
# and nodes' trace kept under data/<label>/<run>/ (bash-film-load (a)).
# One line per test per run goes to data/runs.tsv:
#   label run test verdict load_start load_end cores burners
set -u
here=$(cd "$(dirname "$0")" && pwd)
label=$1 runs=$2 burners=$3; shift 3
cores=$(getconf _NPROCESSORS_ONLN)
filters=()
[ $# -gt 0 ] && filters=(--exact "$@")
pids=()
stop() { [ ${#pids[@]} -gt 0 ] && kill "${pids[@]}" 2>/dev/null; pids=(); }
trap stop EXIT
cargo test -q -p svrngs-node --test browser --no-run 2>/dev/null
for i in $(seq "${FROM:-1}" "$runs"); do
  out=$here/data/$label/$i; mkdir -p "$out"
  for ((j = 0; j < burners; j++)); do yes >/dev/null & pids+=($!); done
  sleep $(( burners > 0 ? 20 : 0 ))  # the 1-minute average climbs to the burners
  start=$(uptime | sed 's/.*averages*: //; s/,//g' | cut -d' ' -f1)
  cargo test -p svrngs-node --test browser -- --nocapture ${filters[@]+"${filters[@]}"} >"$out/test.out" 2>&1
  end=$(uptime | sed 's/.*averages*: //; s/,//g' | cut -d' ' -f1)
  stop
  gzip -c target/traces/browser.log >"$out/trace.log.gz"
  grep -E '^test .* \.\.\. (ok|FAILED)$' "$out/test.out" | sed -E 's/^test (.*) \.\.\. (ok|FAILED)$/\1 \2/' |
    while read -r t v; do printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$label" "$i" "$t" "$v" "$start" "$end" "$cores" "$burners"; done |
    tee -a "$here/data/runs.tsv"
  sleep $(( burners > 0 ? 30 : 0 ))  # let the burners' load fall before an unloaded run
done
