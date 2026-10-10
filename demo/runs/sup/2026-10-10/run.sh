#!/usr/bin/env bash
# sup-4's run (ralph/next/sup/STATE.md): scoped use on mn-1's group, rerun
# on what svrngs carries since sup-1 to sup-3, the computer first —
# Ana's taste on her computer lent `--ask`, Tonight's Picks and the other
# app lent by Sam — on this host through iroh's public relays
# (apps/taste/check/check.sh): the plants first, on the computer, each
# watched failing its bar; then the computers, from their home pages, in
# Chromium and WebKit; then the phones; Ana's computer put to sleep last;
# then the judge. Each step stays inside ten minutes. Needs what mn-1's run
# needs (demo/runs/mn/2026-10-08/run.sh) and node. Writes here; the scratch
# dir keeps the secrets.
set -euo pipefail; cd "$(dirname "$0")/../../../.."
C=apps/taste/check/check.sh R="${1:-/tmp/sup}" OUT=demo/runs/sup/2026-10-10
$C up "$R" $OUT
for plant in bo history other-app after-stop empty; do $C plant "$R" $OUT $plant; done
$C computer "$R" $OUT chromium
$C computer "$R" $OUT webkit
$C phone "$R" $OUT chromium
$C phone "$R" $OUT webkit
$C asleep "$R" $OUT
$C judge "$R" $OUT
$C down "$R" $OUT
