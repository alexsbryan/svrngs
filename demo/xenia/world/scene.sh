#!/bin/sh
# Every scene's command (xenia DESIGN §5, "Scene"): each scene in
# demo/xenia/ is a link to this file, and its name, $0's, names its world
# in crates/svrngs/tests/xenia/main.rs, built first when it is not. The
# world's line is the only one on stdout, cargo's own going to stderr;
# stdin is left to the world, which stays up until it closes.
cd "$(dirname "$0")/../.." && exec cargo test --quiet -p svrngs --test xenia -- "$0"
