#!/bin/sh
# Build vendor/doc-bundle.js from this directory's pins, in a directory of
# its own outside the repository, so nothing is installed here; then say
# its sha256, which vendor/README.md pins (apps/pinned.mjs reads it).
#   apps/doc/bundle/build.sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp "$here/package.json" "$here/package-lock.json" "$here/entry.js" "$work/"
(cd "$work" && npm ci --no-audit --no-fund --loglevel=error >/dev/null &&
  npx esbuild entry.js --bundle --format=esm --minify --legal-comments=none --log-level=warning --outfile=doc-bundle.js)
cp "$work/doc-bundle.js" "$here/../vendor/doc-bundle.js"
shasum -a 256 "$here/../vendor/doc-bundle.js"
