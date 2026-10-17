#!/usr/bin/env bash
# supf's run (ralph/next/lean/STATE.md, lean-4-once-and-follow-ups (b)):
# sup-4's computer journeys again (demo/runs/sup/2026-10-10/run.sh as the
# shape), on svrngs's own answers at a lend's origin now readable by a page
# that reads the lend — and on (c)'s one wording and (d)'s `stop` beside
# `ask`, which the same journeys meet. Ana's taste on her computer lent
# `--ask`, Tonight's Picks and the other app lent by Sam, on this host
# through iroh's public relays (apps/taste/check/check.sh): Ana's computer
# and Alex's, from their home pages, in Chromium and then WebKit. No plants,
# phones or sleep: (b) changes only what a computer's origin answers. Each
# step stays inside ten minutes. Needs what mn-1's run needs
# (demo/runs/mn/2026-10-08/run.sh) and node. Writes here; the scratch dir
# keeps the secrets.
set -euo pipefail; cd "$(dirname "$0")/../../../.."
C=apps/taste/check/check.sh R="${1:-/tmp/supf}" OUT=demo/runs/supf/2026-10-10
cargo build -p svrngs
trap '$C down "$R" $OUT' EXIT
$C up "$R" $OUT
$C computer "$R" $OUT chromium
$C computer "$R" $OUT webkit
