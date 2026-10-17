#!/usr/bin/env bash
# mnp-2's run (ralph/next/compose/STATE.md): Movie Night cut to what
# svrngs tells a page that reads, rerun as mn-1 ran it
# (demo/runs/mn/2026-10-08/run.sh), its steps in the same order, the
# plants first. Needs what that one needs. Writes here.
set -euo pipefail; cd "$(dirname "$0")/../../../.."
C=apps/movie-night/check/check.sh R="${1:-/tmp/mnp}" OUT=demo/runs/mnp/2026-10-09
$C up "$R" $OUT
for plant in per-copy asleep-empty lender-token; do $C plant "$R" $OUT $plant; done
$C phone "$R" $OUT chromium
$C asleep "$R" $OUT chromium
$C phone "$R" $OUT webkit
$C asleep "$R" $OUT webkit
$C computer "$R" $OUT
$C limit "$R" $OUT
$C judge "$R" $OUT
$C down "$R" $OUT
