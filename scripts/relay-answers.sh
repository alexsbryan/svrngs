#!/bin/sh
# relay-answers.sh <url> — whether a relay answers at <url> as iroh-relay
# answers: its /ping 200 and empty (iroh-relay 1.0.2 server.rs
# `probe_handler`), never a page that a host's wildcard serves at the name
# (relay.svrnme.sh answered Vercel's 404, 2026-10-12). deploy-relay.sh
# waits on it; deploy-page.sh publishes no build whose default relay fails
# it (DESIGN §7). Exit 0 when it answers, else 1, saying why.
set -u
case "${1:?relay-answers: give the address a relay is reached at}" in */) ;; *) set -- "$1/" ;; esac
body="$(curl -fsS --max-time 10 "${1}ping")" || exit 1
[ -z "$body" ] || { echo "relay-answers: ${1}ping answered a page, not a relay" >&2; exit 1; }
