#!/usr/bin/env bash
# One WebKit computer pair through the sign-in, outside the record: up, mw,
# the pair, down. $1 scratch, $2 out.
set -uo pipefail; cd "${REPO:-/Users/alexsbryan/dev/svrngs}"
C=apps/jellyfin/check/check.sh; R="$1"; OUT="$2"
"$C" up "$R" "$OUT" >/dev/null 2>&1 && "$C" mw "$R" "$OUT" mw1-again >/dev/null 2>&1
MW=1 "$C" pair "$R" "$OUT" computer webkit 2>&1 | grep -E "^[0-9:]+ pair" | grep -oE "pair [a-z-]+|left alone: [A-Za-z]+ \([^)]*\)" | tr '\n' ' '; echo
"$C" down "$R" "$OUT" >/dev/null 2>&1
