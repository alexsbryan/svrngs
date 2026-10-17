#!/usr/bin/env bash
# build.sh — canon-core to plain JavaScript, for House rules (row canon-spike;
# svrn-docs rings/current/CANON_IN_SVRNGS.md §3, WHAT_RUNS_AS_YOU.md §6).
#
#   apps/canon/build.sh            build, then prove: canon's fixtures replayed
#                                  by canon-cli, the wasm and the JavaScript
#   apps/canon/build.sh --measure  also time a house of 300 acts
#   apps/canon/build.sh --vendor   also copy the proven build into
#                                  apps/canon/vendor/ and pin its sha256s
#
# canon is read from a checkout ($CANON, else ~/dev/canon, as scripts/ports.py
# reads commonwealth-ai's) by `git archive` of the pinned commit alone, so
# the checkout's working tree never reaches the build. The svrngs workspace
# never builds any of this: apps/canon/build/ is a workspace of its own.
#
# Needs Rust 1.95.0 with wasm32-unknown-unknown (canon's rust-toolchain.toml),
# binaryen 133 (wasm-opt, wasm2js) and node. Each is checked, not assumed.
set -euo pipefail
cd "$(dirname "$0")/../.."
ROOT=$PWD

# canon release 0.3.0, as adopted: "contract: adopt 3e5828b… (format v4,
# release 0.3.0) as the standard".
PIN=49eaf31b1e2c6d2f2ca31bee6d220da79e264bda
BINARYEN=133
CANON="${CANON:-$HOME/dev/canon}"
OUT="$ROOT/target/canon"
VENDOR="$ROOT/apps/canon/vendor"

MEASURE="" DO_VENDOR=""
for a in "$@"; do
  case "$a" in
    --measure) MEASURE=1 ;;
    --vendor) DO_VENDOR=1 ;;
    *) echo "build.sh: unknown argument $a (only --measure, --vendor)" >&2; exit 2 ;;
  esac
done

need() { command -v "$1" >/dev/null || { echo "build.sh: $1 is not on PATH — $2" >&2; exit 3; }; }
need wasm2js "brew install binaryen (version $BINARYEN)"
need wasm-opt "brew install binaryen (version $BINARYEN)"
need node "install node 20 or later"
for t in wasm2js wasm-opt; do
  v=$($t --version | awk '{print $3}')
  [ "$v" = "$BINARYEN" ] || { echo "build.sh: $t is version $v, the build pins $BINARYEN" >&2; exit 3; }
done
rustup target list --installed --toolchain 1.95.0 2>/dev/null | grep -q '^wasm32-unknown-unknown$' || {
  echo "build.sh: run  rustup target add wasm32-unknown-unknown --toolchain 1.95.0" >&2; exit 3; }
git -C "$CANON" cat-file -e "$PIN^{commit}" 2>/dev/null || {
  echo "build.sh: $CANON has no commit $PIN (canon 0.3.0); set CANON to a canon checkout" >&2; exit 3; }

echo "canon at $PIN, from $CANON"
if [ "$(cat "$OUT/src/.commit" 2>/dev/null)" != "$PIN" ]; then
  rm -rf "$OUT/src" && mkdir -p "$OUT/src"
  git -C "$CANON" archive "$PIN" | tar -x -C "$OUT/src"
  echo "$PIN" >"$OUT/src/.commit"
fi

echo "the wrapper, for wasm32-unknown-unknown (release), held to rustfmt and clippy…"
(
  cd apps/canon/build
  export CARGO_TARGET_DIR="$OUT/cargo"
  cargo fmt --check
  cargo clippy --quiet --locked --target wasm32-unknown-unknown --all-targets -- -D warnings
  cargo build --quiet --locked --release --target wasm32-unknown-unknown
)
WASM="$OUT/cargo/wasm32-unknown-unknown/release/canon_page.wasm"

# wasm2js reads MVP wasm, and Rust 1.95 writes bulk memory, sign extension
# and saturating float-to-int for this target: refused as written (755
# validator errors, demo/runs/canon/2026-10-16/MEASURE.md). wasm-opt lowers
# the three first, then wasm2js translates.
echo "through wasm-opt's lowerings and wasm2js -O…"
wasm-opt "$WASM" --enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int \
  --enable-mutable-globals --enable-multivalue --enable-reference-types \
  --llvm-memory-copy-fill-lowering --signext-lowering --llvm-nontrapping-fptoint-lowering \
  -o "$OUT/lowered.wasm"
wasm-opt "$OUT/lowered.wasm" --mvp-features -o /dev/null 2>/dev/null
wasm2js -O "$OUT/lowered.wasm" -o "$OUT/canon.js"
if grep -q 'WebAssembly\|eval(\|new Function' "$OUT/canon.js"; then
  echo "build.sh: canon.js names WebAssembly or eval; a carried page could not run it" >&2; exit 1
fi

echo "canon-cli at the same commit, and canon's own governance bar there…"
(
  cd "$OUT/src"
  export CARGO_TARGET_DIR="$OUT/cli"
  cargo build --quiet --locked --release -p canon-cli
  cargo test --quiet --locked --release -p canon-cli --test governance_bar -- --nocapture
)

echo "the proof: every fixture replayed by canon-cli, the wasm and the JavaScript…"
node apps/canon/build/prove.mjs "$OUT/cli/release/canon" "$OUT/src" "$WASM" "$OUT/canon.js"

size() { printf "%-16s %9d bytes, gzip -9 %8d\n" "$1" "$(wc -c <"$2")" "$(gzip -9c "$2" | wc -c)"; }
size "wasm" "$WASM"
size "JavaScript" "$OUT/canon.js"

if [ -n "$MEASURE" ]; then
  (cd apps/canon/build && CARGO_TARGET_DIR="$OUT/slim" \
    cargo build --quiet --locked --release --target wasm32-unknown-unknown --no-default-features)
  SLIM="$OUT/slim/wasm32-unknown-unknown/release/canon_page.wasm"
  wasm-opt "$SLIM" --enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int \
    --enable-mutable-globals --enable-multivalue --enable-reference-types \
    --llvm-memory-copy-fill-lowering --signext-lowering --llvm-nontrapping-fptoint-lowering \
    -o "$OUT/slim.wasm"
  wasm2js -O "$OUT/slim.wasm" -o "$OUT/slim.js"
  size "wasm, no replay" "$SLIM"
  size "JS, no replay" "$OUT/slim.js"
  node apps/canon/build/house.mjs "$WASM" "$OUT/canon.js" "$OUT/house.jsonl"
  now=$(( $(tail -1 "$OUT/house.jsonl" | sed 's/.*"ts_unix":\([0-9]*\).*/\1/') + 3 * 86400 ))
  (cd apps/canon/build && CARGO_TARGET_DIR="$OUT/cargo" \
    cargo run --quiet --locked --release --example time -- "$OUT/house.jsonl" "$now")
fi

if [ -n "$DO_VENDOR" ]; then
  mkdir -p "$VENDOR"
  cp "$OUT/canon.js" "$VENDOR/canon.js"
  cp apps/canon/build/call.mjs "$VENDOR/call.mjs"
  cp "$OUT/src/LICENSE" "$VENDOR/LICENSE"
  echo "vendored into apps/canon/vendor/; pin these in its README.md:"
  (cd "$VENDOR" && shasum -a 256 canon.js call.mjs LICENSE)
fi
