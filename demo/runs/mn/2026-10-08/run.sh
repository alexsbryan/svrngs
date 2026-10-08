#!/usr/bin/env bash
# mn-1's run (ralph/next/compose/STATE.md): Movie Night over Sam's and
# Dee's scratch Jellyfin 12.0.0s and Cy's asleep one, on this host through
# iroh's public relays (apps/movie-night/check/check.sh), the plants first,
# each watched failing its bar, then the runs as Movie Night is kept, then
# the judge. Each step stays inside ten minutes; the whole about forty.
# Needs Jellyfin.app, Playwright's browsers (web/tests), `cargo build -p
# svrngs` and the middleware's (`cargo build --manifest-path
# apps/jellyfin/mw/Cargo.toml`). Writes here; the scratch dir keeps the
# secrets. Movie Night's lines by mn-0's rule are the judge's `the count`.
set -euo pipefail; cd "$(dirname "$0")/../../../.."
C=apps/movie-night/check/check.sh R="${1:-/tmp/mn}" OUT=demo/runs/mn/2026-10-08
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
