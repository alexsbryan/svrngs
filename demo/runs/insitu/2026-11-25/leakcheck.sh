#!/usr/bin/env bash
# Reads every route of the view and fails if any of it holds a client's name,
# a client id, or a phrase of a note: what the inbox holds, by construction.
# Plant: point it at the inbox itself with `./leakcheck.sh --plant`.
set -uo pipefail
cd "$(dirname "$0")"
all=$( [ "${1:-}" = --plant ] && cat out/inbox/* || for r in / /site /trials /service/state /service/events; do ./view.sh "$r"; done )
hits=0
while read -r w; do grep -qiF -- "$w" <<<"$all" && { echo "LEAK: $w"; hits=$((hits+1)); }; done < <(
  ls out/inbox | sed 's/^[0-9-]* //; s/ - session.txt$//' | tr ' ' '\n'
  grep -ho 'TEST-[0-9]*' out/inbox/*
  printf '%s\n' 'week was hard' 'grounding' 'CBT' 'Client said')
[ $hits = 0 ] && echo "passed     no name, id or note phrase in the view ($(wc -c <<<"$all") bytes read)" || { echo "failed     $hits leak(s)"; exit 1; }
