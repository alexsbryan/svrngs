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
# the scope, gpg, and build-release.sh's toolchain.
#
# A page is published only with its release's signature beside its sums,
# since every svrngs updates only from sums the release key signed (DESIGN
# §7, "A computer updates only from a signed release"). So in three steps:
#
#   scripts/deploy-page.sh --build             builds the site, prints its directory
#   scripts/sign-release.sh <tag> <commit> <site>   the operator signs it
#   scripts/deploy-page.sh --site <site> [--project svrngs-page] [--scope svrnmesh]
set -euo pipefail
PROJECT=svrngs-page
SCOPE=svrnmesh
MODE=
while [ $# -gt 0 ]; do
  case "$1" in
    --build) MODE=build; shift ;;
    --site) MODE=site; SITE="${2:?--site needs the directory --build printed}"; shift 2 ;;
    --project) PROJECT="${2:?--project needs a name}"; shift 2 ;;
    --scope) SCOPE="${2:?--scope needs a team}"; shift 2 ;;
    *) echo "deploy-page: unknown argument $1" >&2; exit 2 ;;
  esac
done
if [ "$MODE" = site ]; then
  SITE="$(cd "$SITE" 2>/dev/null && pwd)" || { echo "deploy-page: --site names no directory" >&2; exit 2; }
fi
cd "$(dirname "$0")/.."
VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
if [ -z "$MODE" ]; then
  echo "deploy-page: a page is published only with its release's signature, so in three steps:" >&2
  echo "  scripts/deploy-page.sh --build" >&2
  echo "  scripts/sign-release.sh v$VERSION <commit> <the directory it printed>" >&2
  echo "  scripts/deploy-page.sh --site <that directory>" >&2
  exit 2
fi
# Where its computers meet their groups, the switch (PLAN §2): at n0's
# relays, chosen until the operator flips it; at svrngs's, whose relay
# answers before the build is published, or signed for the GitHub release
# (DESIGN §7); unread, refused.
case "$(sed -n 's/^pub const SHIPPED: MeetAt = MeetAt::\(.*\);$/\1/p' crates/svrngs/src/cli.rs)" in
  N0) echo "deploy-page: this build meets its groups at n0's relays, until the switch to svrngs's (PLAN §2)" >&2 ;;
  Svrngs) RELAY="$(sed -n 's/^pub const DEFAULT_RELAY: &str = "\(.*\)";$/\1/p' crates/svrngs/src/cli.rs)"
    scripts/relay-answers.sh "$RELAY" || { echo "deploy-page: ${RELAY:-no relay} does not answer, and this build meets its groups there: run scripts/deploy-relay.sh first" >&2; exit 1; } ;;
  *) echo "deploy-page: cannot read where this build meets its groups (SHIPPED, crates/svrngs/src/cli.rs)" >&2; exit 1 ;;
esac
if [ "$MODE" = build ]; then
  OUT="$(mktemp -d)/site"
  scripts/build-web.sh --out "$OUT" >&2
  scripts/build-release.sh --out "$OUT" >&2
  cp web/vercel.json "$OUT/vercel.json"
  echo "deploy-page: $(git rev-parse --short HEAD) built, not deployed; the operator signs it, then:" >&2
  echo "  scripts/sign-release.sh v$VERSION $(git rev-parse HEAD) $OUT" >&2
  echo "  scripts/deploy-page.sh --site $OUT" >&2
  echo "$OUT"
  exit 0
fi
# The signature, checked by gpg in a keyring of its own that holds only the
# release key, so no key this computer keeps counts.
command -v gpg >/dev/null || { echo "deploy-page: gpg checks the release's signature before the page is published: install GnuPG" >&2; exit 1; }
KEYRING="$(mktemp -d)"
trap 'rm -rf "$KEYRING"' EXIT
gpg --homedir "$KEYRING" --batch --quiet --import svrngs-release-key.asc 2>/dev/null
if ! gpg --homedir "$KEYRING" --batch --verify "$SITE/get/SHA256SUMS.sig" "$SITE/get/SHA256SUMS" 2>/dev/null; then
  echo "deploy-page: $SITE/get/SHA256SUMS has no valid signature by the release key (svrngs-release-key.asc) beside it, so it is not published: every svrngs would refuse the update. The operator signs it first: scripts/sign-release.sh v$VERSION <commit> $SITE" >&2
  exit 1
fi
echo "deploy-page: $SITE, its sums signed by the release key, to $PROJECT ($SCOPE)" >&2
vercel link --yes --project "$PROJECT" --scope "$SCOPE" --cwd "$SITE" >/dev/null
vercel deploy --prod --yes --scope "$SCOPE" --cwd "$SITE"
