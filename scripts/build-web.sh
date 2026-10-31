#!/usr/bin/env bash
# ported from commonwealth-ai scripts/build-ring-runtime.sh@e1a43bcd2
# build-web.sh — build the page into a deployable static site.
#
# The output is ONE directory to upload to the guest runtime's static origin
# (today group.svrnme.sh, by scripts/deploy-page.sh): index.html + the wasm module and its glue. No server, no
# certificate, no data.
#
#   target/web/site/   <- upload this
#
# Needs the wasm-bindgen CLI matching the crate's wasm-bindgen pin (0.2.128)
# and the wasm32-unknown-unknown target. Both are checked, not assumed. On
# macOS the wasm C sources also need an LLVM clang (Apple's has no wasm32
# backend); Homebrew's is detected and used automatically.
set -euo pipefail
cd "$(dirname "$0")/.."

# The output directory is the deployable static site. Overridable so a consumer
# (landing/scripts/build-ring-runtime.sh → landing/ring/) can place it where
# its host serves it; default is this repo's target/.
OUT="target/web/site"
# --lint holds the crate to rustfmt and clippy instead of building it: it is
# its own workspace, which scripts/check.sh's workspace steps never reach.
LINT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --out) OUT="${2:?--out needs a directory}"; shift 2 ;;
    --lint) LINT=1; shift ;;
    *) echo "build-web: unknown argument $1 (only --out <dir>, --lint)" >&2; exit 2 ;;
  esac
done

APP="web"
WASM_BINDGEN_MIN="0.2.128"

if ! rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$'; then
  echo "build-web: the wasm target is not installed — run:" >&2
  echo "  rustup target add wasm32-unknown-unknown" >&2
  exit 3
fi
if ! command -v wasm-bindgen >/dev/null; then
  echo "build-web: wasm-bindgen is not on PATH — run:" >&2
  echo "  cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_MIN}" >&2
  exit 3
fi
# A version skew between the CLI and the crate's wasm-bindgen is a loud,
# confusing failure at run time; say it here where it is cheap to fix.
cli="$(wasm-bindgen --version | awk '{print $2}')"
if [ "$cli" != "$WASM_BINDGEN_MIN" ]; then
  echo "build-web: wasm-bindgen CLI is ${cli}, the crate pins \
=${WASM_BINDGEN_MIN} — install the matching CLI:" >&2
  echo "  cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_MIN} --force" >&2
  exit 3
fi

if [ -n "$LINT" ]; then echo "linting $APP for wasm32-unknown-unknown…"; else echo "building $APP for wasm32-unknown-unknown (release)…"; fi
# The dependency tree compiles C for the wasm target through cc-rs, and
# Apple's clang has no wasm32 backend: without this the build dies inside
# ring's build script with "No available targets are compatible with triple
# \"wasm32-unknown-unknown\"" — watched live 2026-09-22, mid-demo, with no
# clue in the message that a compiler choice was the problem. Homebrew's
# LLVM does have the backend; default to it on macOS so `npm run deploy`
# needs no remembered environment. An explicit CC and Linux are untouched.
if [ "$(uname -s)" = "Darwin" ] && [ -z "${CC_wasm32_unknown_unknown:-}" ]; then
  found=""
  for p in /opt/homebrew/opt/llvm/bin /usr/local/opt/llvm/bin; do
    if [ -x "$p/clang" ]; then
      export CC_wasm32_unknown_unknown="$p/clang"
      if [ -x "$p/llvm-ar" ]; then export AR_wasm32_unknown_unknown="$p/llvm-ar"; fi
      echo "macOS: using $p/clang for the wasm C sources (Apple clang cannot target wasm32)"
      found=1
      break
    fi
  done
  if [ -z "$found" ]; then
    echo "build-web: on macOS the wasm build needs an LLVM clang with a" >&2
    echo "  wasm32 backend (brew install llvm), or pass one explicitly:" >&2
    echo "  CC_wasm32_unknown_unknown=<path-to-clang> $(basename "$0")" >&2
    exit 3
  fi
fi
# Upstream passed `--cfg getrandom_backend="wasm_js"` here for kernel-types'
# getrandom 0.3; svrngs-core pulls no getrandom, and iroh's 0.4 takes its
# browser backend from the `wasm_js` feature its manifest sets.
if [ -n "$LINT" ]; then
  ( cd "$APP" && cargo fmt --check && cargo clippy --quiet --release --target wasm32-unknown-unknown -- -D warnings )
  exit 0
fi
( cd "$APP" && cargo build --release --target wasm32-unknown-unknown )

rm -rf "$OUT"
mkdir -p "$OUT/wasm"
wasm-bindgen --target web --out-dir "$OUT/wasm" \
  "$APP/target/wasm32-unknown-unknown/release/svrngs_web.wasm"

# Shrink the module when binaryen is present. Optional, not required: the page
# works without it, and a missing optimizer must not fail a build. The size
# matters because the guest downloads it — a CDN will gzip it again on top.
if command -v wasm-opt >/dev/null; then
  wasm-opt -Oz -o "$OUT/wasm/svrngs_web_bg.wasm.opt" "$OUT/wasm/svrngs_web_bg.wasm" \
    && mv "$OUT/wasm/svrngs_web_bg.wasm.opt" "$OUT/wasm/svrngs_web_bg.wasm"
  echo "wasm-opt: -Oz applied"
else
  echo "wasm-opt: not on PATH (binaryen) — shipping the unoptimized module"
fi

# The version is the content hash of the module and of the page's own files,
# stamped into every shell URL and the service-worker cache name, so a rebuild
# invalidates a guest's cache instead of serving a stale runtime (web/sw.js).
# The page's files are in it since u4-10, which changed only them: hashing the
# module alone gave that build u4-9's version, which a host may have cached
# for good (demo/u4.md).
# sha256sum is GNU coreutils; stock macOS has only shasum. Either works.
if command -v sha256sum >/dev/null; then SHA_CMD="sha256sum"; else SHA_CMD="shasum -a 256"; fi
# The home page's files (crates/svrngs-node/src/home/) given to the group's
# page too: question.js, svrngs's question and what apps use, so both ask in
# one wording (#110), look.css, svrngs's one look, so both look one way, and
# away.html, a lend's computer that did not answer, so both say it one way;
# and the tab's icon, the svrngs mark from its one home (docs/brand/).
SHARED=(crates/svrngs-node/src/home/question.js crates/svrngs-node/src/home/look.css crates/svrngs-node/src/home/away.html docs/brand/favicon.svg)
PAGE=(index.html app.js store.js sw.js open.html open.js lend.js shim.js)
VERSION="$(cat "$OUT/wasm/svrngs_web_bg.wasm" "$OUT/wasm/svrngs_web.js" "${PAGE[@]/#/$APP/}" "${SHARED[@]}" | $SHA_CMD | cut -c1-12)"
cp "${SHARED[@]}" "$OUT/"
# install.sh, the line a computer pastes, served at the page's address with
# what scripts/build-release.sh puts under get/ (DESIGN §7).
cp "$APP/install.sh" "$OUT/"
for f in "${PAGE[@]}"; do
  cp "$APP/$f" "$OUT/"
  sed -i.bak "s/__VERSION__/${VERSION}/g" "$OUT/$f"
  rm -f "$OUT/$f.bak"
done
# 404.html, what a static host serves for a path it has no file for, such as
# a lend's own deep path where its worker is not (DESIGN §3, "Lent apps",
# call 1): the page, from the root, with its policy, marked `missing`.
sed -e 's#="\./#="/#g' -e 's#<body>#<body id="missing">#' "$OUT/index.html" >"$OUT/404.html"

echo
echo "built: $OUT  (version ${VERSION})"
echo "  size: $(du -h "$OUT/wasm/svrngs_web_bg.wasm" | cut -f1) wasm"
echo "  1. upload that directory to the guest runtime's static origin"
echo "     (today svrnme.sh, beside the landing page and installers)"
echo "  2. point a grant at it:"
echo "       svrn mesh grant --all-apps --ttl 2h --url https://<origin>/ --qr-svg wall-qr.svg"
echo "       (--model may be omitted: it defaults to the daemon's primary slot)"
echo "  Serve it with gzip/brotli on (the wasm compresses well) and, if the"
echo "  host allows, the headers index.html's meta tags ask for — a CDN gives"
echo "  both; nothing here depends on them."
