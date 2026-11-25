#!/usr/bin/env bash
# SUPPORT_LINK, "Prove the link's boundary", from the desk's side, on
# today's primitives. Each line is a desk read and its answer; the app's
# own refusals and svrngs's are told apart by the x-svrngs-refused header.
#
#   ./probe.sh <before-end|after-app-end|after-unshare>
set -uo pipefail
cd "$(dirname "$0")"
R=$PWD/out/lab
d() { HOME=$R/desk/home SVRNGS_DIR=$R/desk/data NO_COLOR=1 node desk.mjs "$@" | head -c 400; echo; }
case "$1" in
  before-end)
    echo '## a read that works';                     d /incident
    echo '## a write to a listed check';            d /timeline --method POST
    echo '## a replay started from the desk';       d /replay --method POST
    echo '## another viewing session, by its id';   d '/timeline?attempt=1'
    echo '## a route the app did not list';         d /attempts
    ;;
  after-app-end)
    echo '## the same read, after End support access'; d /incident
    ;;
  after-unshare)
    echo '## the same read, after svrngs unshare';     d /incident
    ;;
esac
