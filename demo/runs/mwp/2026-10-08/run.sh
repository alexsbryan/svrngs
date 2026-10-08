#!/usr/bin/env bash
# mwp-3's runs: mw-2's (demo/runs/mw/2026-10-08/mw-2/run.sh), in the order
# they ran (steps.log), with Jellyfin's middleware now the lend's sign-in —
# Jellyfin lent itself, `share 8096 --name jellyfin-mw --sign-in 8097` —
# against a scratch Jellyfin 12.0.0 on this host (apps/jellyfin/check/check.sh
# `mw2`). Needs Jellyfin.app, Playwright's browsers (web/tests) and `cargo
# build -p svrngs`. Then the middleware's code lines, by DESIGN §3 call 6's
# bar 2: awk 'FNR==1{t=0} /^[[:space:]]*#\[cfg\(test\)\]/{t=1} !t &&
# !/^[[:space:]]*($|\/\/|\/\*|\*)/' apps/jellyfin/mw/src/*.rs | wc -l
set -euo pipefail; cd "$(dirname "$0")/../../../.."
apps/jellyfin/check/check.sh mw2 "${1:-/tmp/mwp3}" demo/runs/mwp/2026-10-08
