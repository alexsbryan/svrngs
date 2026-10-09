#!/usr/bin/env bash
# su-1's run (ralph/next/compose/STATE.md): scoped use on mn-1's group —
# Ana's taste on her computer, Tonight's Picks and the other app lent by
# Sam — on this host through iroh's public relays
# (apps/taste/check/check.sh): the plants first, each watched failing its
# bar, then the runs as the code is kept, Ana's computer put to sleep
# last, then the judge. Each step stays inside ten minutes; the whole
# about twenty. Needs what mn-1's run needs (demo/runs/mn/2026-10-08/run.sh)
# and node. Writes here; the scratch dir keeps the secrets. The taste
# service's lines by su-0's rule are the judge's `the count`.
set -euo pipefail; cd "$(dirname "$0")/../../../.."
C=apps/taste/check/check.sh R="${1:-/tmp/su}" OUT=demo/runs/su/2026-10-09
$C up "$R" $OUT
for plant in bo history other-app after-stop empty; do $C plant "$R" $OUT $plant; done
$C night "$R" $OUT chromium
$C night "$R" $OUT webkit
$C asleep "$R" $OUT
$C judge "$R" $OUT
$C down "$R" $OUT
