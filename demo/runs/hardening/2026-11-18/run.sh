#!/bin/bash
# The hardening run of 2026-11-18, driven from the Mac: svrngs and give for
# Linux, built before and after scripts/build-release.sh asked zig for
# -pie, read by readelf, and run. Each step's commands and output are
# <n>-<step>.log here; every heavy build holds the gate lock.
#   demo/runs/hardening/2026-11-18/run.sh [work dir]
# 01-before builds as build-release.sh did at 3b9a0506, by its own line for
# Linux, since running that whole script would build macOS's two as well.
set -uo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(git -C "$here" rev-parse --show-toplevel)
W=${1:-$(mktemp -d)}
cd "$repo" || exit 1
readelf=/opt/homebrew/opt/llvm/bin/llvm-readelf
lock() { lockf -k /tmp/svrngs-gate.lock "$@"; }
run() { printf '$ %s\n' "$*"; bash -c "$*" 2>&1; printf '[exit %s]\n\n' "$?"; }
# A build's output is hundreds of lines: its last few, and its exit status.
build() { printf '$ %s\n' "$*"; lock bash -c "$*" 2>&1 | tail -n "${TAIL:-3}"; printf '[exit %s]\n\n' "${PIPESTATUS[0]}"; }
# What namcap reads: the ELF type, the dynamic section's flags, the
# program headers (GNU_RELRO; INTERP would mean a dynamic loader).
elf() {
  for b in "$@"; do
    run "$readelf -h $b | grep -E 'Type:|Machine:'"
    run "$readelf -d $b"
    run "$readelf -lW $b | sed -n '/Program Headers/,/^\$/p'"
  done
}
header() { printf '# %s, %s\n# commit: %s; %s; %s; %s\n\n' "$1" "$(date -u +%FT%TZ)" \
  "$(git rev-parse --short HEAD)" "$(rustc --version)" "zig $(zig version)" "$(cargo-zigbuild --version)"; }
LINUX="x86_64-unknown-linux-musl aarch64-unknown-linux-musl"
commit="$(git rev-parse --short HEAD)$(git diff --quiet HEAD || echo -dirty)"

{
  header "before: build-release.sh's Linux line as at 3b9a0506, no -pie"
  for t in $LINUX; do
    build "SVRNGS_COMMIT=$commit CARGO_INCREMENTAL=0 cargo zigbuild --release --locked -p svrngs -p give --target $t"
    elf "target/$t/release/svrngs" "target/$t/release/give"
  done
} >"$here/01-before.log"

# After: the script itself, and readelf on the bytes its tarballs carry.
{
  header "after: scripts/build-release.sh with -pie for Linux"
  run "git diff HEAD -- scripts/build-release.sh"
  TAIL=12 build "scripts/build-release.sh --out $W/after"
  for p in linux-x86_64 linux-aarch64; do
    run "mkdir -p $W/x/$p && tar -xzf $W/after/get/svrngs-$p.tar.gz -C $W/x/$p && tar -xzf $W/after/get/give-$p.tar.gz -C $W/x/$p"
    elf "$W/x/$p/svrngs" "$W/x/$p/give"
  done
} >"$here/02-after.log"

# The plant: RUSTFLAGS in the environment, even empty, replaces the
# per-target -pie, and the script refuses both Linux platforms.
{
  header "plant: RUSTFLAGS= in the environment"
  TAIL=8 build "RUSTFLAGS= scripts/build-release.sh --out $W/plant"
  run "ls $W/plant/get"
  for t in $LINUX; do run "$readelf -h target/$t/release/svrngs | grep Type:"; done
} >"$here/03-plant.log"

# Run where each is native: x86_64 on the Fedora box, aarch64 in the Mac's
# Podman machine (Linux on Apple silicon), from a busybox image.
{
  header "x86_64: the after tarballs on the Fedora box"
  run "ssh alexbryan@fedora 'rm -rf /tmp/hardening && mkdir /tmp/hardening' && scp -q $W/after/get/svrngs-linux-x86_64.tar.gz $W/after/get/give-linux-x86_64.tar.gz alexbryan@fedora:/tmp/hardening/"
  run "ssh alexbryan@fedora 'cd /tmp/hardening && for f in *.tar.gz; do tar -xzf \$f; done && uname -srm && file svrngs give && readelf -h svrngs | grep Type: && readelf -d svrngs | grep -E \"FLAGS|BIND\" && readelf -lW svrngs | grep -E \"GNU_RELRO|INTERP\"; ./svrngs version; echo \"[svrngs exit \$?]\"; ./give; echo \"[give exit \$?]\"; rm -rf /tmp/hardening'"
} >"$here/04-x86_64.log"
{
  header "aarch64: the after tarballs in the Podman machine"
  run "podman run --rm -v $W/x/linux-aarch64:/x:ro ad4f11f7c85f sh -c 'uname -srm; /x/svrngs version; echo \"[svrngs exit \$?]\"; /x/give; echo \"[give exit \$?]\"'"
} >"$here/05-aarch64.log"
