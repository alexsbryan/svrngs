#!/usr/bin/env bash
# deploy-page.sh — publish the page where a group's link opens it: the page
# at https://<host>/ and every app at https://<label>.<host>/ (DESIGN §7),
# the same files at the root of every name under the host, so the host needs
# no rewrite. Ours (the operator, 2026-10-06) is the Vercel project
# svrngs-page, its domains group.svrnme.sh and *.group.svrnme.sh, apart from
# the svrnme.sh landing site. Beside the page, what a computer downloads
# from it (scripts/build-release.sh; DESIGN §7). Needs the vercel CLI,
# logged in to the scope, and build-release.sh's toolchain.
#
#   scripts/deploy-page.sh [--project svrngs-page] [--scope svrnmesh]
set -euo pipefail
PROJECT=svrngs-page
SCOPE=svrnmesh
while [ $# -gt 0 ]; do
  case "$1" in
    --project) PROJECT="${2:?--project needs a name}"; shift 2 ;;
    --scope) SCOPE="${2:?--scope needs a team}"; shift 2 ;;
    *) echo "deploy-page: unknown argument $1" >&2; exit 2 ;;
  esac
done
cd "$(dirname "$0")/.."
OUT="$(mktemp -d)/site"
scripts/build-web.sh --out "$OUT"
scripts/build-release.sh --out "$OUT"
cp web/vercel.json "$OUT/vercel.json"
echo "deploy-page: $(git rev-parse --short HEAD) to $PROJECT ($SCOPE) from $OUT"
vercel link --yes --project "$PROJECT" --scope "$SCOPE" --cwd "$OUT" >/dev/null
vercel deploy --prod --yes --scope "$SCOPE" --cwd "$OUT"
