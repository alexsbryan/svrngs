#!/usr/bin/env bash
# deploy-page.sh — publish the page where a group's link opens it: the page
# at https://<host>/ and every app at https://<label>.<host>/ (DESIGN §7),
# the same files at the root of every name under the host, so the host needs
# no rewrite. Ours (the operator, 2026-10-06) is the Vercel project
# svrngs-page, its domains group.svrnme.sh and *.group.svrnme.sh, apart from
# the svrnme.sh landing site. Beside the page, what a computer downloads
# from it (scripts/build-release.sh; DESIGN §7): at once for a build that
# meets its groups at n0's relays, and for one at svrngs's only once that
# relay answers (scripts/relay-answers.sh) — the switch, `SHIPPED` in
# crates/svrngs/src/cli.rs, read here. Needs the vercel CLI, logged in to
# the scope, and build-release.sh's toolchain.
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
# Where its computers meet their groups, the switch (PLAN §2): at n0's
# relays, chosen until the operator flips it; at svrngs's, whose relay
# answers before the build is published (DESIGN §7); unread, refused.
case "$(sed -n 's/^pub const SHIPPED: MeetAt = MeetAt::\(.*\);$/\1/p' crates/svrngs/src/cli.rs)" in
  N0) echo "deploy-page: this build meets its groups at n0's relays, until the switch to svrngs's (DESIGN §9)" ;;
  Svrngs) RELAY="$(sed -n 's/^pub const DEFAULT_RELAY: &str = "\(.*\)";$/\1/p' crates/svrngs/src/cli.rs)"
    scripts/relay-answers.sh "$RELAY" || { echo "deploy-page: ${RELAY:-no relay} does not answer, and this build meets its groups there: run scripts/deploy-relay.sh first" >&2; exit 1; } ;;
  *) echo "deploy-page: cannot read where this build meets its groups (SHIPPED, crates/svrngs/src/cli.rs)" >&2; exit 1 ;;
esac
OUT="$(mktemp -d)/site"
scripts/build-web.sh --out "$OUT"
scripts/build-release.sh --out "$OUT"
cp web/vercel.json "$OUT/vercel.json"
echo "deploy-page: $(git rev-parse --short HEAD) to $PROJECT ($SCOPE) from $OUT"
vercel link --yes --project "$PROJECT" --scope "$SCOPE" --cwd "$OUT" >/dev/null
vercel deploy --prod --yes --scope "$SCOPE" --cwd "$OUT"
