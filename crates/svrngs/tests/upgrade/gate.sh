#!/bin/sh
# The upgrade gate (upgrade/main.rs), as scripts/check.sh's "upgrade" step
# runs it: the last release's test binary made, its release key the tests'
# own (scenes/last-release.sh); then this build and its Keeper built
# trusting that key alone, as a fork pins its own (SVRNGS_RELEASE_KEY,
# update/signed.rs `RELEASE_KEYS`), under target/upgrade/, so no other
# step's build trusts it; then the test, by name. A few minutes cold.
# Built with no debug info: the release it serves is this build's test
# binary, and with Linux's debug info inside it that is past the 64 MiB an
# updater reads of a download (update.rs), which a release build is far
# under; on macOS debug info is a file apart, so it passed there.
set -eu
root=$(git rev-parse --show-toplevel)
cd "$root"
sh crates/svrngs/tests/scenes/last-release.sh >/dev/null
SVRNGS_RELEASE_KEY=$(cat crates/svrngs/tests/upgrade/release-key)
CARGO_TARGET_DIR=$root/target/upgrade
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
CARGO_PROFILE_DEV_STRIP=debuginfo CARGO_PROFILE_TEST_STRIP=debuginfo
export SVRNGS_RELEASE_KEY CARGO_TARGET_DIR CARGO_PROFILE_DEV_DEBUG CARGO_PROFILE_TEST_DEBUG
export CARGO_PROFILE_DEV_STRIP CARGO_PROFILE_TEST_STRIP
cargo build --quiet -p svrngs-keeper
exec cargo test --quiet -p svrngs --test upgrade -- "$@"
