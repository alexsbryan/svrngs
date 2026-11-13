#!/usr/bin/env bash
# check.sh's own scenes and n0 steps (steps.sh: check.sh with SVRNGS_STEPS
# set and each step's whole log kept), in the clean worktree $1, every
# socket of the run's process tree sampled (sockets.py). Run under the gate
# lock, with a hard timeout, by the caller.
set -u
W=$1 out=$2 here=$(cd "$(dirname "$0")" && pwd)
cd "$W" || exit 1
mkdir -p "$out"
cp "$here/steps.sh" scripts/n0-steps.sh
{ echo "worktree $(git rev-parse --short HEAD) (tree $(git rev-parse --short 'HEAD^{tree}'); $(git status --short | grep -v n0-steps.sh | wc -l | tr -d ' ') changed files), eudoxia $(git -C ../eudoxia rev-parse --short HEAD) ($(git -C ../eudoxia status --short | wc -l | tr -d ' ') changed files); began $(date -u +%FT%TZ)"; uptime; } > "$out/run.txt"
RUN_STEPS=$'scenes\nn0' RUN_OUT="$out" CARGO_INCREMENTAL=0 bash scripts/n0-steps.sh >> "$out/run.txt" 2>&1 & p=$!
python3 "$here/sockets.py" "$p" "$out/sockets.tsv" 2 > "$out/sockets.txt" 2>&1
wait $p; s=$?
{ echo "ended $(date -u +%FT%TZ), exit $s"; uptime; } >> "$out/run.txt"
rm -f scripts/n0-steps.sh
exit $s
