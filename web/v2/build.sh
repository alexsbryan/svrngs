# SPDX-License-Identifier: AGPL-3.0-or-later
# web/v2/build.sh — the phone's v2 page (page-scaffold (e)), sourced by
# scripts/build-web.sh --ui v2 once v1's site is in $OUT, which it makes
# v2's, from the sources, so no v1 byte it wrote is kept:
# - index.html is web/index.html with web/v2/body.html for what lies between
#   <body> and its <script line, as 404.html is derived; its <head>, the one
#   page policy (scripts/trust_floor.txt), byte for byte or no build;
# - web/v2/app.js is served as /app.js, where the browser tests find
#   ./wasm/ and ./store.js from the module's address, the flows under /v2/,
#   and ui/ (crates/svrngs-node/src/home/ui/) and vendor/ with their paths;
# - every one of them is in VERSION, and stamped, the shared .js too;
# - sw.js's SHELL lists what was copied, so a new flow needs no edit.
UI_SRC=crates/svrngs-node/src/home/ui
mkdir -p "$OUT/v2" "$OUT/ui" "$OUT/vendor"
cp "${PAGE[@]/#/$APP/}" "$OUT/"
cp "$APP/v2/app.js" "$OUT/app.js"
find "$APP/v2" -name '*.js' ! -name app.js -exec cp {} "$OUT/v2/" \;
cp "$UI_SRC"/*.js "$OUT/ui/"
cp "$APP"/vendor/*.js "$OUT/vendor/"
awk -v body="$APP/v2/body.html" '/<script /{skip=0} !skip{print} /<body>/{while ((getline l <body) > 0) print l; skip=1}' "$APP/index.html" >"$OUT/index.html"
head_of() { sed -n '/<head>/,/<\/head>/p' "$1"; }
cmp -s <(head_of "$APP/index.html") <(head_of "$OUT/index.html") || { echo "build-web: v2's index.html must keep web/index.html's <head> byte for byte, its one policy; web/v2/body.html holds a body only" >&2; exit 1; }
MODULES=($(cd "$OUT" && echo v2/*.js ui/*.js vendor/*.js))
sed "s#^const SHELL = \[\(.*\)\];#const SHELL = [\1$(printf ", './%s?v=__VERSION__'" "${MODULES[@]}")];#" "$APP/sw.js" >"$OUT/sw.js"
grep -q "'./ui/view.js?v=" "$OUT/sw.js" || { echo "build-web: web/sw.js has no \`const SHELL = [...];\` line, so v2's modules would not be kept offline" >&2; exit 1; }
# PAGE is index.html and app.js, v2's own here, then the rest, v1's.
STAMPED=(index.html app.js "${PAGE[@]:2}" question.js "${MODULES[@]}")
VERSION="$(cat "$OUT/wasm/svrngs_web_bg.wasm" "$OUT/wasm/svrngs_web.js" "${STAMPED[@]/#/$OUT/}" "${SHARED[@]}" | $SHA_CMD | cut -c1-12)"
for f in "${STAMPED[@]}"; do sed "s/__VERSION__/${VERSION}/g" "$OUT/$f" >"$OUT/$f.v" && mv "$OUT/$f.v" "$OUT/$f"; done
left="$(grep -rl __VERSION__ "$OUT" --include='*.js' --include='*.html' || true)"
[ -z "$left" ] || { echo "build-web: v2's site still says __VERSION__ in:" $left "— each file served is stamped, so add it to web/v2/build.sh's STAMPED" >&2; exit 1; }
sed -e 's#="\./#="/#g' -e 's#<body>#<body id="missing">#' "$OUT/index.html" >"$OUT/404.html"
