#!/usr/bin/env bash
# dev.sh — run this checkout's svrngs as your login svrngs, on a Mac.
#
#   scripts/dev.sh        build, then point the login service at the build
#   scripts/dev.sh off    point it back at the installed ~/.local/bin/svrngs
#
# The home page is compiled into the binary, and only one svrngs can hold
# your data at a time, so seeing a change with your real groups means the
# login service runs the build. The build is copied to target/dev, so test
# builds in target/debug never swap what runs, and it starts again after a
# restart until `off`. It runs over your real ~/.svrngs: a build newer than
# the installed release may write data that release can't read.
set -euo pipefail
cd "$(dirname "$0")/.."
plist="$HOME/Library/LaunchAgents/sh.svrnme.svrngs.plist"
bin="$HOME/.local/bin/svrngs"
if [[ ${1:-} != off ]]; then
  cargo build -p svrngs -p svrngs-keeper
  mkdir -p target/dev
  # A new file each time: macOS kills a signed binary rewritten in place.
  for b in svrngs svrngs-keeper; do
    cp "target/debug/$b" "target/dev/.$b" && mv -f "target/dev/.$b" "target/dev/$b"
  done
  bin="$PWD/target/dev/svrngs"
fi
/usr/libexec/PlistBuddy -c "Set :ProgramArguments:0 $bin" "$plist"
launchctl bootout "gui/$(id -u)/sh.svrnme.svrngs" 2>/dev/null || true
launchctl bootstrap "gui/$(id -u)" "$plist"
echo "Your login svrngs is now $("$bin" version), from $bin."
