#!/bin/bash
# Why the Linux release was EXEC: what rustc asks the linker for, and what
# zig's cc makes of it, on an empty Rust program and a C file in a
# directory of their own. Writes 00-why.log beside it.
#   demo/runs/hardening/2026-11-18/why.sh
set -uo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(git -C "$here" rev-parse --show-toplevel)
T=$(mktemp -d)
readelf=/opt/homebrew/opt/llvm/bin/llvm-readelf
run() { printf '$ %s\n' "$*"; bash -c "$*" 2>&1; printf '[exit %s]\n\n' "$?"; }
LINUX="x86_64-unknown-linux-musl aarch64-unknown-linux-musl"
{
  printf '# why, %s\n# %s; zig %s; %s\n\n' "$(date -u +%FT%TZ)" \
    "$(cd "$repo" && rustc --version)" "$(zig version)" "$(cargo-zigbuild --version)"
  # rustc's own word on each target: x86_64's musl can link static-pie,
  # aarch64's leaves it out, which means it cannot.
  for t in $LINUX; do
    run "cd $repo && RUSTC_BOOTSTRAP=1 rustc -Z unstable-options --print target-spec-json --target $t | grep -E 'position-independent|crt-static-default'"
  done
  # What rustc hands cargo-zigbuild's zig cc to link, the link mode only.
  run "cd $T && cargo init -q --name hello && cp $repo/rust-toolchain.toml ."
  for t in $LINUX; do
    run "cd $T && RUSTFLAGS=--print=link-args cargo zigbuild -q --release --target $t | tr ' ' '\n' | grep -xE '\"-(static|static-pie|pie|no-pie)\"|\"-Wl,-z,relro,-z,now\"'; $readelf -h target/$t/release/hello | grep Type:"
  done
  # zig's cc alone: -static-pie is dropped, a -pie after it is taken.
  printf 'int main(void) { return 0; }\n' >"$T/m.c"
  for flags in "-static-pie -Wl,-z,relro,-z,now" "-static-pie -Wl,-z,relro,-z,now -pie"; do
    run "cd $T && rm -f m && zig cc -target x86_64-linux-musl $flags -o m m.c && $readelf -h m | grep Type: && $readelf -d m | grep -E 'FLAGS|NEEDED'; $readelf -l m | grep -E 'INTERP|GNU_RELRO'"
  done
} >"$here/00-why.log"
rm -rf "$T"
