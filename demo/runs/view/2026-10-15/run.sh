#!/bin/sh
# arch-view (a): the scale instrument at four points, one run each, in release.
# Usage: run.sh <label>   — writes data/<label>/<members>x<acts>.json and the
# commit and host load each was run at.
set -eu
label=$1
here=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$here" rev-parse --show-toplevel)
out="$here/data/$label"
mkdir -p "$out"
git -C "$root" rev-parse HEAD > "$out/commit.txt"
for point in 2x1000 5x1000 25x1000 25x10000; do
  m=${point%x*}; a=${point#*x}
  echo "$point load-before: $(sysctl -n vm.loadavg)" >> "$out/load.txt"
  ( cd "$root" && SVRNGS_SCALE_MEMBERS=$m SVRNGS_SCALE_ACTS=$a SVRNGS_SCALE_ASLEEP=0 \
      cargo test --release -p svrngs-node --test scale -- --ignored --nocapture --exact scale ) \
    2>&1 | sed -n 's/^SCALE //p' > "$out/$point.json"
  echo "$point load-after: $(sysctl -n vm.loadavg)" >> "$out/load.txt"
  test -s "$out/$point.json" || { echo "$point printed no SCALE line" >&2; exit 1; }
done
