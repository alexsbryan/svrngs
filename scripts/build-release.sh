#!/usr/bin/env bash
# build-release.sh — build what a computer downloads from the group's page
# (DESIGN §7, "A computer gets svrngs from the group's link"): `svrngs` in
# release for each platform, one tarball each beside one SHA256SUMS, under
# get/ of the page's site, where install.sh fetches them.
#
#   scripts/build-release.sh [--out target/web/site]
#
# macOS's two by cargo; Linux's two static (musl) by zig through
# cargo-zigbuild, the open-source cross toolchain (brew install zig;
# cargo install cargo-zigbuild). A platform that does not build is said by
# name and the run exits 1 after the rest: never a site that lacks one
# silently.
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
# svrngs-<uname -s, lower case>-<uname -m, arm64 read as aarch64>.
TARGETS=(
  "darwin-aarch64 aarch64-apple-darwin cargo build"
  "darwin-x86_64 x86_64-apple-darwin cargo build"
  "linux-x86_64 x86_64-unknown-linux-musl cargo zigbuild"
  "linux-aarch64 aarch64-unknown-linux-musl cargo zigbuild"
)
GET="$OUT/get"
rm -rf "$GET"
mkdir -p "$GET"
failed=()
for t in "${TARGETS[@]}"; do
  read -r platform triple build <<<"$t"
  echo "build-release: $platform ($triple) by ${build}…"
  # shellcheck disable=SC2086 # $build is a command and its subcommand
  if ! CARGO_INCREMENTAL=0 $build --release --locked -p svrngs --target "$triple"; then
    echo "build-release: $platform did not build" >&2
    failed+=("$platform")
    continue
  fi
  COPYFILE_DISABLE=1 tar --no-xattrs -czf "$GET/svrngs-$platform.tar.gz" \
    -C "target/$triple/release" svrngs
done
if command -v sha256sum >/dev/null; then SHA_CMD="sha256sum"; else SHA_CMD="shasum -a 256"; fi
if [ ${#failed[@]} -lt ${#TARGETS[@]} ]; then
  ( cd "$GET" && $SHA_CMD svrngs-*.tar.gz >SHA256SUMS )
  echo
  echo "built: $GET"
  cat "$GET/SHA256SUMS"
fi
if [ ${#failed[@]} -gt 0 ]; then
  echo "build-release: not built: ${failed[*]}" >&2
  exit 1
fi
