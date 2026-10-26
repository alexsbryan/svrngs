#!/bin/sh
# One world of crates/svrngs/tests/xenia/main.rs as its scene's command
# (xenia DESIGN §5, "Scene"): built first when it is not, then run by its
# name, $1. Its scene line is the one line on stdout; the test runner's own
# lines are kept off it. Stdin is left to the world, which stays up until
# it closes.
cd "$(dirname "$0")/../../.." || exit 2
cargo test --quiet -p svrngs --test xenia -- --exact "$1" --nocapture --test-threads 1 | grep --line-buffered '^{'
