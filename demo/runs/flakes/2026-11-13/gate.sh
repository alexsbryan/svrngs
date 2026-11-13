#!/usr/bin/env bash
# The full gate in the clean worktree $1, under the gate lock (taken by the caller).
cd "$1" || exit 1
echo "worktree $(git rev-parse --short HEAD) ($(git status --short | wc -l | tr -d ' ') changed files), eudoxia $(git -C ../eudoxia rev-parse --short HEAD) ($(git -C ../eudoxia status --short | wc -l | tr -d ' ') changed files); began $(date -u +%FT%TZ); lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh, CARGO_INCREMENTAL=0, npm ci --prefix web"
uptime
CARGO_INCREMENTAL=0 bash scripts/check.sh; s=$?
echo "ended $(date -u +%FT%TZ), exit $s"; uptime
exit $s
