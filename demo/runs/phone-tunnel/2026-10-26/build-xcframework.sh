#!/bin/sh
# The tunnel's Rust, for a phone, as ios/Tunnel.xcframework: the static
# library and the header its two functions are called by.
set -eu
here=$(cd "$(dirname "$0")" && pwd -P)
target=${CARGO_TARGET_DIR:-$here/tunnel/target}
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT

# The C in ring and blake3 is built for the SDK's own iOS unless told the
# project's, and the linker then warns of every object.
export IPHONEOS_DEPLOYMENT_TARGET=16.0
# No compiler cache: sccache served this library with the C of an earlier
# build in it, built for the SDK's iOS (otool -l on its blake3_neon.o, 2026-10-26).
export RUSTC_WRAPPER=
# IROH_PATCH, NETWATCH_PATCH: patched copies of iroh 1.0.2 and netwatch 0.19.1
# to build against instead, as for ../../phone-replica/2026-10-27/*.patch.
# Cargo rewrites the lock to name them, so the lock is kept and put back.
if [ -n "${IROH_PATCH:-}${NETWATCH_PATCH:-}" ]; then
    cp "$here/tunnel/Cargo.lock" "$out/Cargo.lock"
    trap 'cp "$out/Cargo.lock" "$here/tunnel/Cargo.lock"; rm -rf "$out"' EXIT
    CARGO_TARGET_DIR=$target cargo build --manifest-path "$here/tunnel/Cargo.toml" --target aarch64-apple-ios --release \
        ${IROH_PATCH:+--config "patch.crates-io.iroh.path='$IROH_PATCH'"} \
        ${NETWATCH_PATCH:+--config "patch.crates-io.netwatch.path='$NETWATCH_PATCH'"}
else
    CARGO_TARGET_DIR=$target cargo build --manifest-path "$here/tunnel/Cargo.toml" --target aarch64-apple-ios --release --locked
fi

mkdir -p "$out/include"
cat > "$out/include/tunnel.h" <<'H'
#include <stdint.h>
// 0 running, 1 no log_dir or no trace file in it, 2 refused (traced), 3 already running.
int32_t tunnel_start(const char *config_json);
void tunnel_stop(void);
// The provider's sleep and wake, counted for the probe.
void tunnel_slept(void);
void tunnel_woke(void);
H
cat > "$out/include/module.modulemap" <<'M'
module Tunnel {
    header "tunnel.h"
    export *
}
M

rm -rf "$here/ios/Tunnel.xcframework"
xcodebuild -create-xcframework \
    -library "$target/aarch64-apple-ios/release/libphone_tunnel.a" -headers "$out/include" \
    -output "$here/ios/Tunnel.xcframework"
