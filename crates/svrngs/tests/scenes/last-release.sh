#!/bin/sh
# The last release with no app databases, v1.13.3, as the mixed scene runs
# it (mixed.rs `last_release`): the tag's tree taken whole from git into
# target/last-release/, its `address` test binary built there, and kept at
# target/last-release/svrngs-v1.13.3-test-key. That binary names its
# src/main.rs's `main` in the ignored test `svrngs_here` (its
# tests/common/as_main.rs, the file this build's scenes run as), so the
# scene and the upgrade gate run v1.13.3 as they run this build, at a relay on 127.0.0.1. Its release key is the tests' own
# (SVRNGS_RELEASE_KEY, as a fork pins its own: update/signed.rs
# `RELEASE_KEYS`), upgrade/release-key, so the upgrade gate's page, which
# that key signs, is one its own updater takes (upgrade/main.rs). Run once
# a checkout, a few minutes cold; it prints where the binary is.
set -eu
tag=v1.13.3
root=$(git rev-parse --show-toplevel)
at=$root/target/last-release
made=$at/svrngs-$tag-test-key
if [ -x "$made" ]; then
    echo "$made"
    exit 0
fi
src=$at/$tag
rm -rf "$src"
mkdir -p "$src"
git -C "$root" archive "$tag" | tar -x -C "$src"
npm ci --prefix "$src/web" --no-audit --no-fund >&2
# The tag's commit and its time, as a release says them (build-release.sh),
# and no git above the tree, so the outer checkout's HEAD is never read as
# its time (svrngs-node/build.rs `BUILT`).
SVRNGS_COMMIT=$(git -C "$root" rev-parse --short=8 "$tag^{commit}")
SOURCE_DATE_EPOCH=$(git -C "$root" log -1 --format=%ct "$tag")
SVRNGS_RELEASE_KEY=$(cat "$root/crates/svrngs/tests/upgrade/release-key")
export SVRNGS_COMMIT SOURCE_DATE_EPOCH SVRNGS_RELEASE_KEY GIT_CEILING_DIRECTORIES="$at" CARGO_TARGET_DIR="$at/target"
cd "$src"
built=$(cargo test -p svrngs --test address --no-run --message-format=json)
exe=$(printf '%s\n' "$built" | sed -n 's/.*"executable":"\([^"]*\/address-[^"]*\)".*/\1/p' | tail -1)
[ -x "$exe" ] || {
    echo "last-release.sh: cargo named no address test binary" >&2
    exit 1
}
cp "$exe" "$made"
echo "$made"
