#!/bin/sh
# The upgrade gate (upgrade/main.rs), as scripts/check.sh's "upgrade" step
# runs it: the last release's test binary made, its release key the tests'
# own (scenes/last-release.sh); then this build and its Keeper built
# trusting that key alone, as a fork pins its own (SVRNGS_RELEASE_KEY,
# update/signed.rs `RELEASE_KEYS`), under target/upgrade/, so no other
# step's build trusts it; then the test, by name. A few minutes cold.
set -eu
root=$(git rev-parse --show-toplevel)
cd "$root"
sh crates/svrngs/tests/scenes/last-release.sh >/dev/null
SVRNGS_RELEASE_KEY=$(cat crates/svrngs/tests/upgrade/release-key)
CARGO_TARGET_DIR=$root/target/upgrade
export SVRNGS_RELEASE_KEY CARGO_TARGET_DIR
cargo build --quiet -p svrngs-keeper
exec cargo test --quiet -p svrngs --test upgrade -- "$@"
