#!/usr/bin/env bash
# release.sh — release svrngs in three commands, typed as they are in the
# main checkout, each ending with the next one to type:
#
#   scripts/release.sh build    builds the release, publishing nothing
#   scripts/release.sh sign     signs it with the release key's passphrase, then makes the GitHub release
#   scripts/release.sh page     publishes the signed page
#
# What it publishes is the page where a group's link opens it: the page at
# https://<host>/ and every app at https://<label>.<host>/ (DESIGN §7), the
# same files at the root of every name under the host, so the host needs no
# rewrite. Ours (the operator, 2026-10-06) is the Vercel project
# svrngs-page, its domains group.svrnme.sh and *.group.svrnme.sh, apart from
# the svrnme.sh landing site. Beside the page, what a computer downloads
# from it (scripts/build-release.sh; DESIGN §7): at once for a build that
# meets its groups at n0's relays, and for one at svrngs's only once that
# relay answers (scripts/relay-answers.sh) — the switch, `SHIPPED` in the
# release commit's crates/svrngs/src/cli.rs, read by build and page. Needs
# the vercel CLI, logged in to the scope, gpg, gh, and build-release.sh's
# toolchain.
#
# build builds the commit that set Cargo.toml's version, the release's
# prepare commit, though main has moved on (9be0fcc7 for V1.13.2, with main
# past it, on 2026-11-21); `build <commit>` builds another. The tag is that
# commit's own version. It checks the commit out in one worktree kept from
# release to release, ../svrngs-release beside this checkout, so cargo's
# release targets stay warm. A tree with changes of its own is refused, so
# a build is only its commit's. Then web/node_modules from the lock (npm
# ci) and no target/debug, the 8 GB a check there leaves (the operator's 15
# GB disk floor), and that commit's own build-web.sh and build-release.sh,
# under the Mac's one gate lock, since four release targets are a heavy run
# (/tmp/svrngs-gate.lock, as check.sh takes it). It writes the site, its
# log and the state sign and page read under the worktree's target/, not a
# temporary folder, which macOS empties. The builds' output goes to stderr
# and the log, so stdout is only the next command.
#
# A page is published only with its release's signature beside its sums,
# since every svrngs updates only from sums the release key signed (DESIGN
# §7, "A computer updates only from a signed release"). sign refuses a
# commit not on origin/main, since its tag must name what everyone can
# fetch, and a tag that exists, since a version is signed once. It gives
# each download, SHA256SUMS and install.sh a detached OpenPGP signature
# (`<file>.sig`, what a PKGBUILD's validpgpkeys checks), signs the tag, and
# keeps the release on GitHub with the signatures and the public key. The
# operator runs it and types the release key's passphrase in gpg's dialog;
# no agent signs (RELEASES.md, "Signed releases"). The notes are
# RELEASES.md's section for the version, as the tagged commit has it.
set -euo pipefail
cd "$(dirname "$0")/.."
WORK="$(dirname "$PWD")/svrngs-release"
STATE="$WORK/target/release.state"
say() { echo "release: $*" >&2; }
stop() { say "$*"; exit 1; }
version() { git show "$1:Cargo.toml" | sed -n 's/^version = "\(.*\)"$/\1/p' | head -1; }
# Where the commit's computers meet their groups, the switch (PLAN §2): at
# n0's relays, chosen until the operator flips it; at svrngs's, whose relay
# answers before the build is signed for the GitHub release or published
# (DESIGN §7), so sign and page ask; unread, refused.
meets() {
  case "$(git show "$COMMIT:crates/svrngs/src/cli.rs" | sed -n 's/^pub const SHIPPED: MeetAt = MeetAt::\(.*\);$/\1/p')" in
    N0) say "$TAG meets its groups at n0's relays, until the switch to svrngs's (PLAN §2)" ;;
    Svrngs) RELAY="$(git show "$COMMIT:crates/svrngs/src/cli.rs" | sed -n 's/^pub const DEFAULT_RELAY: &str = "\(.*\)";$/\1/p')"
      scripts/relay-answers.sh "$RELAY" || stop "${RELAY:-no relay} does not answer, and $TAG meets its groups there: run scripts/deploy-relay.sh first" ;;
    *) stop "cannot read where $TAG meets its groups (SHIPPED, crates/svrngs/src/cli.rs at ${COMMIT:0:8})" ;;
  esac
}

build() {
  COMMIT="$(git rev-parse --quiet --verify "${1:-$(git log -1 --format=%H -G"^version = \"$(version HEAD)\"\$" -- Cargo.toml)}^{commit}")" \
    || stop "${1:-the commit that set version $(version HEAD)} is not a commit here, so nothing was built. Name one this checkout has: scripts/release.sh build <commit>"
  TAG="v$(version "$COMMIT")" SITE="$WORK/target/release-site" LOG="$WORK/target/release.log"
  [ -e "$WORK" ] || git worktree add --quiet --detach "$WORK" "$COMMIT"
  [ -z "$(git -C "$WORK" status --porcelain)" ] || stop "$WORK, where releases are built, has changes of its own, so a build there would not be ${COMMIT:0:8}'s alone. See them with git -C $WORK status; keep what you need elsewhere, then git -C $WORK stash --include-untracked and build again"
  git -C "$WORK" checkout --quiet --detach "$COMMIT"
  rm -rf "$STATE" "$SITE" "$WORK/target/debug"
  mkdir -p "$WORK/target"
  cd "$WORK"
  say "building $TAG from ${COMMIT:0:8} in $WORK, under the gate lock: if another heavy run holds it, this waits (lsof /tmp/svrngs-gate.lock names it)"
  { npm ci --prefix web && lockf -k /tmp/svrngs-gate.lock bash -c 'scripts/build-web.sh --out "$1" && scripts/build-release.sh --out "$1"' _ "$SITE"; } 2>&1 | tee "$LOG" >&2 \
    || stop "$TAG did not build, so there is nothing to sign. Its log, $LOG, says what failed; fix that, then scripts/release.sh build again"
  cp web/vercel.json "$SITE/vercel.json"
  printf 'TAG=%q\nCOMMIT=%q\nSITE=%q\nLOG=%q\n' "$TAG" "$COMMIT" "$SITE" "$LOG" >"$STATE"
  say "$TAG is built from ${COMMIT:0:8} into $SITE, not published yet (its log: $LOG). Next, sign it, typing the release key's passphrase when gpg asks:"
  echo "  scripts/release.sh sign"
}

sign() {
  git fetch --quiet origin || stop "origin did not answer, so whether ${COMMIT:0:8} is pushed is unknown. Check the network, then scripts/release.sh sign again"
  if git rev-parse --quiet --verify "refs/tags/$TAG" >/dev/null; then stop "$TAG is tagged already, so it was signed: a version is signed once. To release again, set the next version in Cargo.toml and RELEASES.md, push it, then scripts/release.sh build"; fi
  git merge-base --is-ancestor "$COMMIT" origin/main || stop "${COMMIT:0:8} is not on origin/main, so $TAG would name a commit nobody else can fetch. Push it (git push origin main), then scripts/release.sh sign again"
  local title=V${TAG#v} get=$SITE/get key notes
  # awk reads RELEASES.md to its end: an early exit would end git show by
  # SIGPIPE, which pipefail counts as a failure.
  notes="$(git show "$COMMIT:RELEASES.md" | awk -v head="## $title," 'index($0, head) == 1 { on = 1; next } /^## / { on = 0 } on')"
  [ -n "$notes" ] || stop "RELEASES.md at ${COMMIT:0:8} has no section headed \"## $title,\", which is the release's notes. Add it, push it, then scripts/release.sh build again"
  # With no such key gpg exits 2, which pipefail would make a silent exit.
  key=$(gpg --list-secret-keys --with-colons "svrngs releases" | awk -F: '/^fpr/ { print $10; exit }') || true
  [ -n "$key" ] || stop "there is no 'svrngs releases' key here to sign with. Make it once: gpg --quick-generate-key 'svrngs releases' ed25519 sign 2y"
  say "$TAG ($COMMIT) by $key from $get"
  (cd "$get" && shasum -a 256 -c SHA256SUMS) >&2
  # The installer too, as the release's commit has it, so whatever installs
  # svrngs for someone can check it before running it, where the line on the
  # page trusts HTTPS alone.
  git show "$COMMIT:web/install.sh" >"$get/install.sh"
  for file in "$get"/*.tar.gz "$get"/SHA256SUMS "$get"/install.sh; do
    gpg --yes --local-user "$key" --detach-sign --output "$file.sig" "$file"
    gpg --verify "$file.sig" "$file"
  done
  gpg --armor --export "$key" >"$get/svrngs-release-key.asc"
  git tag --sign --local-user "$key" --message "svrngs $title" "$TAG" "$COMMIT"
  git push origin "$TAG"
  gh release create "$TAG" --verify-tag --latest --title "$title" --notes "$notes

Signed by the svrngs release key, $key: each file has its \`.sig\`, and \`svrngs-release-key.asc\` is the key." \
    "$get"/*.tar.gz "$get"/*.tar.gz.sig "$get"/SHA256SUMS "$get"/SHA256SUMS.sig \
    "$get"/install.sh "$get"/install.sh.sig "$get"/svrngs-release-key.asc >&2
  say "$TAG is signed and released on GitHub (gh release download $TAG reads it back). Next, publish its page:"
  echo "  scripts/release.sh page"
}

page() {
  # The signature, checked by gpg in a keyring of its own that holds only the
  # release key, so no key this computer keeps counts.
  KEYRING="$(mktemp -d)"
  # And what the Vercel CLI leaves in the site, its link and a token
  # (.env.local) among them, deleted however this ends.
  trap 'rm -rf "$KEYRING" "$SITE/.vercel" "$SITE/.env.local" "$SITE/.gitignore"' EXIT
  git show "$COMMIT:svrngs-release-key.asc" | gpg --homedir "$KEYRING" --batch --quiet --import 2>/dev/null
  gpg --homedir "$KEYRING" --batch --verify "$SITE/get/SHA256SUMS.sig" "$SITE/get/SHA256SUMS" 2>/dev/null \
    || stop "$SITE/get/SHA256SUMS has no valid signature by the release key (svrngs-release-key.asc) beside it, so it is not published: every svrngs would refuse the update. Sign it first: scripts/release.sh sign"
  say "$TAG, its sums signed by the release key, to svrngs-page (svrnmesh)"
  vercel link --yes --project svrngs-page --scope svrnmesh --cwd "$SITE" >/dev/null
  vercel deploy --prod --yes --scope svrnmesh --cwd "$SITE"
  say "$TAG's page is published. Next, read it back, then make its Arch package: python3 -I scripts/arch-package.py ${TAG#v}"
}

case "${1:-}" in
  build) [ $# -le 2 ] || stop "build takes one commit at most: scripts/release.sh build [<commit>]"; shift; build "$@" ;;
  # What build saved, TAG, COMMIT, SITE and LOG, as shell assignments.
  sign | page) [ $# -eq 1 ] || stop "$1 takes nothing after it: it reads what scripts/release.sh build saved"
    . "$STATE" 2>/dev/null || stop "there is no release built for $1 to read: $STATE is missing. Build it first: scripts/release.sh build"
    meets
    "$1" ;;
  *) say "svrngs is released in three steps, typed as they are:"
    sed -n 's/^#   \(scripts\/release\.sh .*\)$/  \1/p' "$0" >&2
    exit 2 ;;
esac
