#!/usr/bin/env bash
# v1-onboarding (f) (ralph/next/v1/STATE.md, its row): a newcomer's
# computer from the line to the group, on this Mac. The page host is
# scripts/build-web.sh's site with scripts/build-release.sh's get/ in it,
# served on localhost (web/tests/serve.mjs). Sam makes Trip there with a
# photo in its folder, on n0 as an installed svrngs is. The newcomer reads
# the line off the group's page in Chromium on a computer (onboard.mjs
# line) and runs it with a HOME of its own, so nothing of this host's is
# touched but one LaunchAgent, booted out at the end; `open` is stubbed to
# say what it was given and when, and Chromium opens that (onboard.mjs
# home). Timed from the paste to the home page listing Trip with its
# folder, and to Sam's photo in that folder.
#
#   run.sh <scratch> <out>        writes <out>/run.json, steps.log and what each step printed
set -euo pipefail
R="$1"; OUT="$2"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../../.." && pwd)"
B="$REPO/target/debug/svrngs"
LABEL=sh.svrnme.svrngs
ms() { python3 -c 'import time;print(int(time.time()*1000))'; }
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
port_of() { for _ in $(seq 1 100); do grep -m1 -o 'port [0-9]*' "$1" 2>/dev/null | cut -d' ' -f2 && return; sleep 0.1; done; return 1; }
field() { python3 -c "import json,sys;print(json.load(sys.stdin)$1)"; }
down() {
  launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null && say "booted out $LABEL" || true
  for p in "$R"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done
  rm -f "$R"/*.pid
}
mkdir -p "$OUT"; : >"$OUT/steps.log"
if launchctl print "gui/$(id -u)/$LABEL" >/dev/null 2>&1; then
  say "a svrngs is kept running on this host already ($LABEL): this run would replace it, so it stops"; exit 3
fi
rm -rf "$R"; mkdir -p "$R/sam" "$R/newcomer" "$R/stub"
trap down EXIT

# --- the page host: the page, install.sh, and the four tarballs ----------
"$REPO/scripts/build-web.sh" --out "$R/site" >"$R/build-web.log" 2>&1
t=$(ms); "$REPO/scripts/build-release.sh" --out "$R/site" >"$R/build-release.log" 2>&1 || true
say "build-release, $(( ($(ms) - t) / 1000 )) s: $(tail -1 "$R/build-release.log")"
cp "$R/site/get/SHA256SUMS" "$OUT/SHA256SUMS"
( cd "$R/site/get" && shasum -a 256 -c SHA256SUMS ) >"$OUT/sums-checked.txt" 2>&1
ls -l "$R/site/get" | awk 'NR>1 {print $5, $9}' >"$OUT/tarballs.txt"
node "$REPO/web/tests/serve.mjs" "$R/site" >"$R/site.out" 2>&1 & echo $! >"$R/site.pid"
PAGE="http://localhost:$(port_of "$R/site.out")/"
say "the page host: $PAGE"

# --- Sam: Trip, and a photo in its folder ---------------------------------
SVRNGS_DIR="$R/sam" RUST_LOG=info NO_COLOR=1 nohup "$B" new Trip --as Sam --origin "$PAGE" >"$R/sam/new.out" 2>"$R/sam/run.log" &
echo $! >"$R/sam.pid"
for _ in $(seq 1 60); do grep -q "$PAGE#" "$R/sam/new.out" 2>/dev/null && break; sleep 1; done
LINK="$(grep -o "$PAGE#[A-Za-z0-9_-]*" "$R/sam/new.out")"
mkdir -p "$R/sam-disk/Trip"
"/Applications/Jellyfin.app/Contents/MacOS/ffmpeg" -v error -y -f lavfi -i "testsrc2=size=1600x1200" -frames:v 1 "$R/sam-disk/Trip/beach.jpg"
SVRNGS_DIR="$R/sam" NO_COLOR=1 "$B" share --folder "$R/sam-disk/Trip" --group Trip >"$OUT/sam-share-folder.out" 2>&1
say "sam: $(head -1 "$R/sam/new.out"); $(head -1 "$OUT/sam-share-folder.out"); beach.jpg $(wc -c <"$R/sam-disk/Trip/beach.jpg" | tr -d ' ') bytes"

# --- the newcomer ----------------------------------------------------------
cat >"$R/stub/open" <<STUB
#!/bin/sh
printf '%s %s\n' "\$(python3 -c 'import time;print(int(time.time()*1000))')" "\$1" >>"$R/opened"
STUB
chmod +x "$R/stub/open"
node "$HERE/onboard.mjs" line "$LINK" >"$OUT/line.json"
LINE="$(field '["line"]' <"$OUT/line.json")"
say "the group's page shows, on a computer: $(sed -E 's/#[A-Za-z0-9_-]+/#<the token>/' <<<"$LINE")"
T0=$(ms)
say "pasted"
env -u SVRNGS_DIR HOME="$R/newcomer" PATH="$R/stub:$PATH" sh -c "$LINE" >"$OUT/install.out" 2>&1 && code=0 || code=$?
T_INSTALLED=$(ms)
say "the line exited $code after $(( T_INSTALLED - T0 )) ms: $(tail -1 "$OUT/install.out")"
OPENED="$(cut -d' ' -f2 "$R/opened" 2>/dev/null | head -1)"
T_OPENED="$(cut -d' ' -f1 "$R/opened" 2>/dev/null | head -1)"
DIR="$R/newcomer/svrngs/Trip"
node "$HERE/onboard.mjs" home "${OPENED:-none}" Trip "$DIR" "$T0" >"$OUT/home.json" || true
say "home page: $(cat "$OUT/home.json" | head -c 400)"
T_FILE=null
while [ $(( $(ms) - T0 )) -lt 300000 ]; do
  if [ -f "$DIR/beach.jpg" ] && cmp -s "$DIR/beach.jpg" "$R/sam-disk/Trip/beach.jpg"; then T_FILE=$(( $(ms) - T0 )); break; fi
  sleep 0.2
done
say "Sam's beach.jpg in the newcomer's $DIR: ${T_FILE} ms after the paste"
cp "$R/newcomer/.svrngs/svrngs.log" "$OUT/newcomer-svrngs.log" 2>/dev/null || true
sed -E 's/#[A-Za-z0-9_-]+/#<the link>/g' "$R/sam/new.out" >"$OUT/sam-new.out"
sed -i '' -E 's/#[A-Za-z0-9_-]+/#<the token>/g' "$OUT/install.out" "$OUT/line.json"
python3 - "$OUT" "$T0" "$code" "$T_INSTALLED" "${T_OPENED:-null}" "$T_FILE" "$(sw_vers -productVersion)" "$(uname -m)" <<'PY'
import json, sys
out, t0, code, installed, opened, file, macos, arch = sys.argv[1:]
t0 = int(t0)
home = json.load(open(f"{out}/home.json"))
rel = lambda v: None if v == "null" else int(v) - t0
run = {"host": {"macos": macos, "arch": arch}, "line_exit": int(code),
       "ms_from_paste": {"line_done": rel(installed), "home_page_opened": rel(opened),
                         "home_page_lists_trip_and_its_folder": home.get("listed_ms"),
                         "sams_photo_in_the_folder": None if file == "null" else int(file)},
       "home": home}
json.dump(run, open(f"{out}/run.json", "w"), indent=1)
print(json.dumps(run["ms_from_paste"]))
PY
