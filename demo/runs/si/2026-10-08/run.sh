#!/usr/bin/env bash
# si-1's Jellyfin run: mwp-3's (demo/runs/mwp/2026-10-08/run.sh,
# `apps/jellyfin/check/check.sh mw2`), its steps one by one, as mw2 runs
# them, so each stays inside a ten-minute call; after the sign-in was
# decided at one site, asked for each person apart, and its stand-in named
# in the door's ask (#87, #88, #90). Needs Jellyfin.app, Playwright's
# browsers (web/tests) and `cargo build -p svrngs`.
set -euo pipefail; cd "$(dirname "$0")/../../../.."
R="${1:-/tmp/si1b}"; OUT=demo/runs/si/2026-10-08; C=apps/jellyfin/check/check.sh
case "${2:-all}" in
  up|all) "$C" up "$R" "$OUT"; "$C" mw "$R" "$OUT" mw1-again ;;&
  plants|all) for plant in no-sign-in lender-key one-device; do
      JELLYFIN_MW_PLANT=$plant MW=1 "$C" pair "$R" "$OUT" computer chromium
      "$C" judge "$R" "$OUT" "mw-computer-chromium-plant-$plant" || true
    done ;;&
  computer|all) for engine in chromium webkit; do MW=1 "$C" pair "$R" "$OUT" computer $engine; done ;;&
  phone|all) for engine in chromium webkit; do MW=1 "$C" pair "$R" "$OUT" phone $engine; done ;;&
  judge|all) "$C" judge "$R" "$OUT" mw-computer-chromium mw-computer-webkit mw-phone-chromium mw-phone-webkit ;;&
  down|all) "$C" down "$R" "$OUT" ;;
esac
