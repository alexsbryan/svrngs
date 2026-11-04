#!/usr/bin/env bash
# build.sh — give's fold to plain JavaScript, for give's carried page
# (give-app-keep; svrn-docs rings/current/GIVE.md §7a decision 2), as
# apps/canon/build.sh builds canon-core: the keeper program links
# apps/give/fold, and the page runs this build of the same source, so the
# keeper fact has one decider.
#
#   apps/give/build.sh            build, then prove: fold/cases.json answered
#                                 alike by the wasm and the JavaScript
#   apps/give/build.sh --vendor   also copy the proven build into
#                                 apps/give/page/vendor/ and print the sha256s
#                                 its README.md pins, the source's among them
#
# A carried page gets no 'wasm-unsafe-eval' (scripts/trust_floor.txt), so the
# wasm goes through binaryen's wasm2js. Needs Rust 1.95.0 with
# wasm32-unknown-unknown, binaryen 133 and node; each is checked, not assumed.
set -euo pipefail
cd "$(dirname "$0")/../.."
ROOT=$PWD

TOOLCHAIN=1.95.0
BINARYEN=133
OUT="$ROOT/target/give-fold"
VENDOR="$ROOT/apps/give/page/vendor"

DO_VENDOR=""
for a in "$@"; do
  case "$a" in
    --vendor) DO_VENDOR=1 ;;
    *) echo "build.sh: unknown argument $a (only --vendor)" >&2; exit 2 ;;
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
rustup target list --installed --toolchain "$TOOLCHAIN" 2>/dev/null | grep -q '^wasm32-unknown-unknown$' || {
  echo "build.sh: run  rustup target add wasm32-unknown-unknown --toolchain $TOOLCHAIN" >&2; exit 3; }

# Small first, as canon's build: the page downloads this. Panics abort, as a
# trap. Set here, for this build alone, so the workspace's own release
# profile is untouched.
echo "the fold, for wasm32-unknown-unknown (release)…"
CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=true CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
  CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_STRIP=true \
  cargo "+$TOOLCHAIN" build --quiet --locked --release -p give-fold \
  --target wasm32-unknown-unknown --target-dir "$OUT/cargo"
WASM="$OUT/cargo/wasm32-unknown-unknown/release/give_fold.wasm"

# wasm2js reads MVP wasm, and Rust 1.95 writes bulk memory, sign extension
# and saturating float-to-int for this target (apps/canon/build.sh): lowered
# first by wasm-opt, then translated.
echo "through wasm-opt's lowerings and wasm2js -O…"
wasm-opt "$WASM" --enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int \
  --enable-mutable-globals --enable-multivalue --enable-reference-types \
  --llvm-memory-copy-fill-lowering --signext-lowering --llvm-nontrapping-fptoint-lowering \
  -o "$OUT/lowered.wasm"
wasm-opt "$OUT/lowered.wasm" --mvp-features -o /dev/null 2>/dev/null
wasm2js -O "$OUT/lowered.wasm" -o "$OUT/fold.js"
if grep -q 'WebAssembly\|eval(\|new Function' "$OUT/fold.js"; then
  echo "build.sh: fold.js names WebAssembly or eval; a carried page could not run it" >&2; exit 1
fi

echo "the proof: every case in fold/cases.json, answered by the wasm and the JavaScript…"
node apps/give/fold.test.mjs --wasm "$WASM" --js "$OUT/fold.js"

size() { printf "%-12s %9d bytes, gzip -9 %8d\n" "$1" "$(wc -c <"$2")" "$(gzip -9c "$2" | wc -c)"; }
size "wasm" "$WASM"
size "JavaScript" "$OUT/fold.js"

if [ -n "$DO_VENDOR" ]; then
  mkdir -p "$VENDOR"
  cp "$OUT/fold.js" "$VENDOR/fold.js"
  echo "vendored into apps/give/page/vendor/; pin these in its README.md:"
  (cd "$VENDOR" && shasum -a 256 fold.js call.mjs ../../fold/src/lib.rs ../../fold/Cargo.toml)
fi
