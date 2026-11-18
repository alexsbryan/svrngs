#!/usr/bin/env bash
# build-release.sh — build what a computer downloads from the group's page
# (DESIGN §7, "A computer gets svrngs from the group's link"): `svrngs` in
# release for each platform, and Give's program (`give`, apps/give), which
# install.sh gets when asked with --give, one tarball each beside one
# SHA256SUMS, under get/ of the page's site, where install.sh fetches them.
# Give is built here so it comes from the same page and is checked by the
# same sums: a coupling of distribution, not of code — it ships when svrngs
# releases, by svrngs's toolchain and targets, and still links no svrngs
# crate (apps/give/tests/surface.rs).
#
#   scripts/build-release.sh [--out target/web/site]
#
# macOS's two by cargo; Linux's two static (musl) by zig through
# cargo-zigbuild, the open-source cross toolchain (brew install zig;
# cargo install cargo-zigbuild), on mimalloc rather than the malloc of
# zig's musl, which kept what svrngs frees (crates/svrngs/src/main.rs;
# demo/runs/leak/2026-10-17/README.md). A platform that does not build is
# said by name and the run exits 1 after the rest: never a site that lacks
# one silently.
set -euo pipefail
cd "$(dirname "$0")/.."
OUT="target/web/site"
while [ $# -gt 0 ]; do
  case "$1" in
    --out) OUT="${2:?--out needs a directory}"; shift 2 ;;
    *) echo "build-release: unknown argument $1 (only --out <dir>)" >&2; exit 2 ;;
  esac
done
# Each tarball's name is install.sh's for the platform it names:
# <program>-<uname -s, lower case>-<uname -m, arm64 read as aarch64>.
TARGETS=(
  "darwin-aarch64 aarch64-apple-darwin cargo build"
  "darwin-x86_64 x86_64-apple-darwin cargo build"
  "linux-x86_64 x86_64-unknown-linux-musl cargo zigbuild"
  "linux-aarch64 aarch64-unknown-linux-musl cargo zigbuild"
)
# Linux's two are static-pie with full RELRO: loaded at a random address,
# relocated before main, then made read-only. namcap warns of a package's
# program without them (demo/runs/pacman/2026-11-18/06-namcap.log). rustc
# asks zig for -static-pie, which zig's cc drops as unused, linking a
# fixed-address EXEC; a -pie after it zig takes, and with musl still links
# nothing dynamic (demo/runs/hardening/2026-11-18/). rustc's own
# -z relro -z now then gives BIND_NOW. To check a binary: `readelf -h` says
# DYN, `readelf -d` BIND_NOW and FLAGS_1 NOW PIE, `readelf -l` GNU_RELRO and
# no INTERP. A RUSTFLAGS in the environment, even empty, replaces these
# flags, so a Linux binary that is not DYN is refused below.
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C link-arg=-pie"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C link-arg=-pie"
# `svrngs version` names the commit each binary is built from.
SVRNGS_COMMIT="$(git rev-parse --short HEAD)$(git diff --quiet HEAD || echo -dirty)"
export SVRNGS_COMMIT
GET="$OUT/get"
rm -rf "$GET"
mkdir -p "$GET"
failed=()
for t in "${TARGETS[@]}"; do
  read -r platform triple build <<<"$t"
  echo "build-release: $platform ($triple) by ${build}…"
  # shellcheck disable=SC2086 # $build is a command and its subcommand
  CARGO_INCREMENTAL=0 $build --release --locked -p svrngs -p give --target "$triple" || { echo "build-release: $platform did not build" >&2; failed+=("$platform"); continue; }
  # The ELF header's e_type, its bytes 16 and 17: 3 is DYN, 2 is EXEC.
  if [[ $triple == *-linux-* ]]; then
    for p in svrngs give; do
      [ "$(od -An -tu2 -j16 -N2 "target/$triple/release/$p" | tr -d ' \n')" = 3 ] || { echo "build-release: $platform's $p is not position-independent (readelf -h says EXEC, not DYN)" >&2; failed+=("$platform"); continue 2; }
    done
  fi
  for p in svrngs give; do
    COPYFILE_DISABLE=1 tar --no-xattrs -czf "$GET/$p-$platform.tar.gz" -C "target/$triple/release" $p
  done
done
if command -v sha256sum >/dev/null; then SHA_CMD="sha256sum"; else SHA_CMD="shasum -a 256"; fi
if [ ${#failed[@]} -lt ${#TARGETS[@]} ]; then
  ( cd "$GET" && $SHA_CMD svrngs-*.tar.gz give-*.tar.gz >SHA256SUMS )
  echo
  echo "built: $GET"
  cat "$GET/SHA256SUMS"
fi
[ ${#failed[@]} -eq 0 ] || { echo "build-release: not built: ${failed[*]}" >&2; exit 1; }
