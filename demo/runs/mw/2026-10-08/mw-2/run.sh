#!/usr/bin/env bash
# mw-2's runs, in the order they ran (steps.log), against a scratch
# Jellyfin 12.0.0 on this host: up, the middleware lent (mw-1's check, run
# again), each plant on a computer in Chromium, then the four pairs, judged
# with the scan over every byte; then down. Needs Jellyfin.app, Playwright's
# browsers (web/tests) and `cargo build -p svrngs`. The first pair, whose
# socket was refused (README "What it found" 1), ran before the middleware's
# fix and is not rerun.
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../../../../.." && pwd)"; cd "$REPO"
C=apps/jellyfin/check/check.sh; R="${1:-/tmp/mw2}"; O=demo/runs/mw/2026-10-08/mw-2
$C up "$R" $O
$C mw "$R" $O mw1-again
for plant in no-sign-in lender-key one-device; do
  JELLYFIN_MW_PLANT=$plant MW=1 $C pair "$R" $O computer chromium
  $C judge "$R" $O mw-computer-chromium-plant-$plant || true
done
for way in computer phone; do for engine in chromium webkit; do
  MW=1 $C pair "$R" $O $way $engine
done; done
$C judge "$R" $O mw-computer-chromium mw-computer-webkit mw-phone-chromium mw-phone-webkit
$C down "$R" $O
git diff 5e877c9..HEAD -- crates web | wc -l
