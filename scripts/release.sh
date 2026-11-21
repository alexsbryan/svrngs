#!/usr/bin/env bash
# release.sh — release svrngs: build the page and what a computer downloads
# from it, sign them, then publish the page where a group's link opens it:
# the page at https://<host>/ and every app at https://<label>.<host>/
# (DESIGN §7), the same files at the root of every name under the host, so
# the host needs no rewrite. Ours (the operator, 2026-10-06) is the Vercel
# project svrngs-page, its domains group.svrnme.sh and *.group.svrnme.sh,
# apart from the svrnme.sh landing site. Beside the page, what a computer
# downloads from it (scripts/build-release.sh; DESIGN §7): at once for a
# build that meets its groups at n0's relays, and for one at svrngs's only
# once that relay answers (scripts/relay-answers.sh) — the switch, `SHIPPED`
# in crates/svrngs/src/cli.rs, read here. Needs the vercel CLI, logged in to
# the scope, gpg, gh, and build-release.sh's toolchain.
#
# A page is published only with its release's signature beside its sums,
# since every svrngs updates only from sums the release key signed (DESIGN
# §7, "A computer updates only from a signed release"). So in three steps:
#
#   scripts/release.sh build                          builds the site, prints its directory
#   scripts/release.sh sign <tag> <commit> <site>     the operator signs it
#   scripts/release.sh page <site> [--project svrngs-page] [--scope svrnmesh]
#
# sign gives each download, SHA256SUMS and install.sh a detached OpenPGP
# signature (`<file>.sig`, what a PKGBUILD's validpgpkeys checks), signs the
# tag, and keeps the release on GitHub with the signatures and the public
# key. The operator runs it and types the release key's passphrase in gpg's
# dialog; no agent signs (RELEASES.md, "Signed releases"). The notes are
# RELEASES.md's section for the version, as the tagged commit has it.
set -euo pipefail
PROJECT=svrngs-page
SCOPE=svrnmesh
MODE="${1:-}"
if [ "$MODE" = page ]; then
  SITE="$(cd "${2:?page needs the directory build printed}" 2>/dev/null && pwd)" || { echo "release: page names no directory" >&2; exit 2; }
  shift 2
  while [ $# -gt 0 ]; do
    case "$1" in
      --project) PROJECT="${2:?--project needs a name}"; shift 2 ;;
      --scope) SCOPE="${2:?--scope needs a team}"; shift 2 ;;
      *) echo "release: unknown argument $1" >&2; exit 2 ;;
    esac
  done
fi
cd "$(dirname "$0")/.."
VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
case "$MODE" in
  build | sign | page) ;;
  *) echo "release: a page is published only with its release's signature, so in three steps:" >&2
    echo "  scripts/release.sh build" >&2
    echo "  scripts/release.sh sign v$VERSION <commit> <the directory it printed>" >&2
    echo "  scripts/release.sh page <that directory>" >&2
    exit 2 ;;
esac

if [ "$MODE" = sign ]; then
  tag=${2:?sign needs a tag} commit=${3:?sign needs a commit} get=${4:?sign needs the site}/get
  version=V${tag#v}
  # With no such key gpg exits 2, which pipefail would make a silent exit.
  key=$(gpg --list-secret-keys --with-colons "svrngs releases" | awk -F: '/^fpr/ { print $10; exit }') || true
  if [ -z "$key" ]; then
    echo "release: no 'svrngs releases' key here; make it once with:"
    echo "  gpg --quick-generate-key 'svrngs releases' ed25519 sign 2y"
    exit 1
  fi
  echo "release: $tag ($commit) by $key from $get"
  (cd "$get" && shasum -a 256 -c SHA256SUMS)
  # The installer too, as the release's commit has it, so whatever installs
  # svrngs for someone can check it before running it, where the line on the
  # page trusts HTTPS alone.
  git show "$commit:web/install.sh" >"$get/install.sh"
  for file in "$get"/*.tar.gz "$get"/SHA256SUMS "$get"/install.sh; do
    gpg --yes --local-user "$key" --detach-sign --output "$file.sig" "$file"
    gpg --verify "$file.sig" "$file"
  done
  gpg --armor --export "$key" >"$get/svrngs-release-key.asc"
  notes=$(mktemp)
  # awk reads RELEASES.md to its end: an early exit would end git show by
  # SIGPIPE, which pipefail counts as a failure.
  git show "$commit:RELEASES.md" | awk -v head="## $version," 'index($0, head) == 1 { on = 1; next } /^## / { on = 0 } on' >"$notes"
  test -s "$notes" || { echo "release: RELEASES.md at $commit has no $version section"; exit 1; }
  printf '\nSigned by the svrngs release key, %s: each file has its `.sig`, and `svrngs-release-key.asc` is the key.\n' "$key" >>"$notes"
  git tag --sign --local-user "$key" --message "svrngs $version" "$tag" "$commit"
  git push origin "$tag"
  gh release create "$tag" --verify-tag --latest --title "$version" --notes-file "$notes" \
    "$get"/*.tar.gz "$get"/*.tar.gz.sig "$get"/SHA256SUMS "$get"/SHA256SUMS.sig \
    "$get"/install.sh "$get"/install.sh.sig "$get"/svrngs-release-key.asc
  echo "release: $tag released; read it back with gh release download $tag"
  exit 0
fi

# Where its computers meet their groups, the switch (PLAN §2): at n0's
# relays, chosen until the operator flips it; at svrngs's, whose relay
# answers before the build is published, or signed for the GitHub release
# (DESIGN §7); unread, refused.
case "$(sed -n 's/^pub const SHIPPED: MeetAt = MeetAt::\(.*\);$/\1/p' crates/svrngs/src/cli.rs)" in
  N0) echo "release: this build meets its groups at n0's relays, until the switch to svrngs's (PLAN §2)" >&2 ;;
  Svrngs) RELAY="$(sed -n 's/^pub const DEFAULT_RELAY: &str = "\(.*\)";$/\1/p' crates/svrngs/src/cli.rs)"
    scripts/relay-answers.sh "$RELAY" || { echo "release: ${RELAY:-no relay} does not answer, and this build meets its groups there: run scripts/deploy-relay.sh first" >&2; exit 1; } ;;
  *) echo "release: cannot read where this build meets its groups (SHIPPED, crates/svrngs/src/cli.rs)" >&2; exit 1 ;;
esac
if [ "$MODE" = build ]; then
  OUT="$(mktemp -d)/site"
  scripts/build-web.sh --out "$OUT" >&2
  scripts/build-release.sh --out "$OUT" >&2
  cp web/vercel.json "$OUT/vercel.json"
  echo "release: $(git rev-parse --short HEAD) built, not deployed; the operator signs it, then:" >&2
  echo "  scripts/release.sh sign v$VERSION $(git rev-parse HEAD) $OUT" >&2
  echo "  scripts/release.sh page $OUT" >&2
  echo "$OUT"
  exit 0
fi
# The signature, checked by gpg in a keyring of its own that holds only the
# release key, so no key this computer keeps counts.
command -v gpg >/dev/null || { echo "release: gpg checks the release's signature before the page is published: install GnuPG" >&2; exit 1; }
KEYRING="$(mktemp -d)"
trap 'rm -rf "$KEYRING"' EXIT
gpg --homedir "$KEYRING" --batch --quiet --import svrngs-release-key.asc 2>/dev/null
if ! gpg --homedir "$KEYRING" --batch --verify "$SITE/get/SHA256SUMS.sig" "$SITE/get/SHA256SUMS" 2>/dev/null; then
  echo "release: $SITE/get/SHA256SUMS has no valid signature by the release key (svrngs-release-key.asc) beside it, so it is not published: every svrngs would refuse the update. The operator signs it first: scripts/release.sh sign v$VERSION <commit> $SITE" >&2
  exit 1
fi
echo "release: $SITE, its sums signed by the release key, to $PROJECT ($SCOPE)" >&2
vercel link --yes --project "$PROJECT" --scope "$SCOPE" --cwd "$SITE" >/dev/null
vercel deploy --prod --yes --scope "$SCOPE" --cwd "$SITE"
