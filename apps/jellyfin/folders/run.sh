#!/usr/bin/env bash
# gf-1's run (ralph/next/folders/STATE.md; demo/runs/gf/2026-10-09/README.md
# §2-§4): three people's computers as three `svrngs` data dirs on this host,
# through iroh's public relays, in one group. Ana shares a folder of an MP4,
# an MKV and a link to a file beside it; Cy a copy of the MP4; Ben's computer
# is the meeting point, its endpoint on 127.0.0.1:$PORT, under a Jellyfin of
# the run's own (the macOS app, its own dirs, port $JF_PORT) over one .strm
# stub per film (strm.py). Playwright's Chromium and WebKit play as the first
# run's driver did (driver.cjs).
#
#   run.sh <out>      every phase, its data under <out>; PHASES picks some
#                     (else "A B D C P"); W is the scratch dir (else /tmp/gf-run)
#
# Phases, Ben restarted between each with his blob store moved aside and kept
# for the scan (blobs-<phase>), each plant with its guard out first:
#   A  the scan (B3), Chromium's MP4 (B2, B4, B5), WebKit's MP4, WebKit's MKV
#   B  Ana killed with -9 (B6): Ben with SVRNGS_GF_PLANT=unbounded, then not
#   D  Cy killed, a byte of Ana's MP4 flipped on her disk after her store took
#      it (B1): the first run's range server over the flipped file, then Ben
#   C  Ana's `unshare --folder` while probe.py asks Ben for the MKV (B7): Ana
#      with SVRNGS_GF_PLANT=unchecked, then not
#   P  the file beside the folder (B7): Ana with SVRNGS_GF_PLANT=follow-links
#      shares again, and Ben fetches what her act then names; its store is the
#      scan's own plant, kept apart (blobs-P-plant)
# Then analyze.py judges each bar. Takes about 15 minutes on this host; svrngs
# is built first. scripts/check.sh runs none of this: it needs Jellyfin.
set -euo pipefail
OUT="$(mkdir -p "$1" && cd "$1" && pwd)"
W="${W:-/tmp/gf-run}"; PHASES="${PHASES:-A B D C P}"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../.." && pwd)"
B="$REPO/target/debug/svrngs"
JF_APP="${JF_APP:-/Applications/Jellyfin.app/Contents}"; JF_PORT=18096; PORT=18733
LOGS="folder=info,rail:folder=debug"; export NO_COLOR=1
FLIP=500000  # the byte of Ana's MP4 flipped in D, in its first MiB
PHASE=setup

now() { python3 -c 'import time;print(round(time.time(),3))'; }
say() { printf '%s %s %s\n' "$(date -u +%H:%M:%S)" "$PHASE" "$*" | tee -a "$OUT/steps.log" >&2; }
ev() { printf '{"t":%s,"phase":"%s","event":"%s"}\n' "$(now)" "$PHASE" "$1" >>"$OUT/events.jsonl"; say "$1"; }
field() { python3 -c "import json,sys;print(json.load(sys.stdin)$1)"; }
api() { cat "$W/$1/api" | tr -d '\n'; }
# Asked with the token beside the api file, on stdin so `ps` never shows it (give-token).
listed() { printf 'Authorization: Bearer %s' "$(cat "$W/$1/token")" | curl -sf --max-time 5 -H @- "$(api "$1")/v1/folders"; }

# up <who> <plant or -> <svrngs args...>: a computer's node, its log the run's.
up() {
  local who=$1 plant=$2; shift 2
  local port=0; [ "$who" = ben ] && port=$PORT
  rm -f "$W/$who/api"
  SVRNGS_DIR="$W/$who" SVRNGS_FOLDER_PORT=$port SVRNGS_GF_PLANT="${plant#-}" RUST_LOG="$LOGS" \
    "$B" "$@" >>"$W/$who/out" 2>>"$OUT/$who.log" &
  echo $! >"$W/$who.pid"
  ev "$who up, plant ${plant}: svrngs $(printf '%s' "$*" | sed -E 's#svrngs://join/[A-Za-z0-9_-]+#<the link>#')"
  for _ in $(seq 1 150); do [ -s "$W/$who/api" ] && listed "$who" >/dev/null && return 0; sleep 0.2; done
  say "$who's node did not come up"; exit 3
}
# down <who> [signal]: stopped, and gone before the next starts on its dir.
down() {
  local who=$1 sig=${2:-TERM} pid; pid="$(cat "$W/$who.pid" 2>/dev/null)" || return 0
  kill -"$sig" "$pid" 2>/dev/null || true
  while kill -0 "$pid" 2>/dev/null; do sleep 0.1; done
  rm -f "$W/$who.pid"; ev "$who stopped ($sig)"
}
# fresh_ben <plant>: Ben restarted, what his store held moved aside for the scan.
fresh_ben() {
  down ben; [ -d "$W/ben/blobs" ] && mv "$W/ben/blobs" "$W/ben/blobs-$LAST"
  LAST=$PHASE; up ben "$1"
}
# lists <who> <python condition on the list, `l`>: waits up to 90 s.
lists() {
  for _ in $(seq 1 450); do
    listed "$1" | python3 -c "import json,sys;l=json.load(sys.stdin);sys.exit(0 if ($2) else 1)" 2>/dev/null && return 0
    sleep 0.2
  done
  say "$1's list never held: $2"; exit 3
}
drive() {
  PHASE="$PHASE" QUIET_LOG="$OUT/ben.log" node "$HERE/driver.cjs" "$OUT/state.json" "$OUT" "$1" 2>&1 | tee -a "$OUT/driver.out"
}
cleanup() {
  for p in "$W"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null; done; rm -f "$W"/*.pid
}
trap cleanup EXIT

rm -rf "$W"; mkdir -p "$W"/{ana,ben,cy,flip} "$W"/jf/{data,config,cache,log} "$W/lib/Movies" "$W"/media/{ana,cy}/films
: >"$OUT/steps.log"; : >"$OUT/events.jsonl"; LAST=A
say "load: $(uptime)"
say "svrngs $(git -C "$REPO" rev-parse --short HEAD), $(ffmpeg -version | head -1)"

# --- the media: the first run's (its run.sh:24-30), and the planted file ---
A="$W/media/ana/films"
ffmpeg -hide_banner -loglevel error -y -f lavfi -i testsrc2=size=1280x720:rate=30:duration=120 \
  -f lavfi -i sine=frequency=440:sample_rate=48000:duration=120 \
  -c:v libx264 -profile:v high -level 4.0 -pix_fmt yuv420p -preset veryfast -b:v 3M -maxrate 3M \
  -bufsize 6M -g 60 -c:a aac -b:a 128k -ac 2 -movflags +faststart "$A/exp.mp4"
ffmpeg -hide_banner -loglevel error -y -i "$A/exp.mp4" -c copy "$A/exp.mkv"
cp "$A/exp.mp4" "$W/media/cy/films/exp.mp4"
python3 - "$W/media/ana/beside.bin" <<'EOF'
import os, sys
marker = b"GF-PLANTED-BESIDE-THE-FOLDER-0001"
block = bytearray(os.urandom(1 << 20))
for at in range(0, len(block), 4096):
    block[at:at + len(marker)] = marker
open(sys.argv[1], "wb").write(block)
EOF
ln -s ../beside.bin "$A/link.bin"
python3 - "$A/exp.mp4" "$A/exp.mkv" "$W/media/ana/beside.bin" >"$OUT/media.jsonl" <<'EOF'
import blake3, json, os, sys
for p in sys.argv[1:]:
    print(json.dumps({"path": p, "size": os.path.getsize(p),
                      "blake3": blake3.blake3(open(p, "rb").read()).hexdigest()}))
EOF
say "media: $(python3 -c "import json;print([(json.loads(l)['path'].split('/')[-1], json.loads(l)['size']) for l in open('$OUT/media.jsonl')])")"

# --- the group: Ana makes it; Ben and Cy join by its link -------------------
up ana - new Films --as Ana
for _ in $(seq 1 60); do grep -q 'svrngs://join/' "$W/ana/out" 2>/dev/null && break; sleep 1; done
LINK="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$W/ana/out" | head -1)"
for who in ben cy; do
  up "$who" - join "$LINK" --as "$(printf '%s' "$who" | python3 -c 'import sys;print(sys.stdin.read().title())')"
  for _ in $(seq 1 60); do grep -q "You're in" "$W/$who/out" 2>/dev/null && break; sleep 1; done
  grep -q "You're in" "$W/$who/out" || { say "$who did not join: $(cat "$W/$who/out")"; exit 3; }
done
SVRNGS_DIR="$W/ana" "$B" share --folder "$A" >"$OUT/ana-share.json"
SVRNGS_DIR="$W/cy" "$B" share --folder "$W/media/cy/films" >"$OUT/cy-share.json"
ev "ana and cy shared"
lists ben 'len(l["folders"]) == 2'
listed ben >"$OUT/setup-ben-folders.json"
python3 "$HERE/strm.py" "$(api ben)" "$W/ben/token" "$W/lib/Movies" | tee "$OUT/stubs.jsonl"
MP4="$(grep '"ExpMp4' "$OUT/stubs.jsonl" | field '["hash"]')"
MKV="$(grep '"ExpMkv' "$OUT/stubs.jsonl" | field '["hash"]')"

# --- the run's own Jellyfin, as apps/jellyfin/check/check.sh starts one -----
cat >"$W/jf/config/network.xml" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<NetworkConfiguration xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <InternalHttpPort>$JF_PORT</InternalHttpPort>
  <PublicHttpPort>$JF_PORT</PublicHttpPort>
  <EnableIPv6>false</EnableIPv6>
  <LocalNetworkAddresses><string>127.0.0.1</string></LocalNetworkAddresses>
  <EnableRemoteAccess>false</EnableRemoteAccess>
</NetworkConfiguration>
EOF
"$JF_APP/MacOS/jellyfin" -d "$W/jf/data" -c "$W/jf/config" -C "$W/jf/cache" -l "$W/jf/log" \
  -w "$JF_APP/Resources/jellyfin-web" --ffmpeg "$JF_APP/MacOS/ffmpeg" --nonetchange >"$W/jf/stdout.log" 2>&1 &
echo $! >"$W/jf.pid"
ok=0
for _ in $(seq 1 120); do
  if info="$(curl -sS -f --max-time 2 "http://127.0.0.1:$JF_PORT/System/Info/Public" 2>/dev/null)" && [[ "$info" == *StartupWizardCompleted* ]]; then
    ok=$((ok + 1)); [ $ok -ge 2 ] && break
  else ok=0; fi
  sleep 1
done
[ $ok -ge 2 ] || { say "Jellyfin did not come up"; exit 3; }
say "jellyfin: $info"
python3 "$HERE/jf_setup.py" "http://127.0.0.1:$JF_PORT" "$OUT/state.json" "$W/lib/Movies" | tee "$OUT/setup.out"

phase() { case " $PHASES " in *" $1 "*) PHASE=$1; return 0 ;; *) return 1 ;; esac; }

if phase A; then
  drive chromium:mp4,webkit:mp4,webkit:mkv
  listed ben >"$OUT/A-ben-folders.json"
fi

if phase B; then
  PHASE=B-plant; down ana KILL; ev "ana killed (-9), no goodbye"
  fresh_ben unbounded
  drive chromium:mp4
  listed ben >"$OUT/B-plant-ben-folders.json"
  PHASE=B; fresh_ben -
  drive chromium:mp4
  listed ben >"$OUT/B-ben-folders.json"
fi

if phase D; then
  PHASE=D-plant; down cy KILL; ev "cy killed (-9)"
  [ -f "$W/ana.pid" ] || up ana -
  python3 - "$A/exp.mp4" "$FLIP" <<'EOF'
import sys
with open(sys.argv[1], "r+b") as f:
    f.seek(int(sys.argv[2])); b = f.read(1); f.seek(int(sys.argv[2])); f.write(bytes([b[0] ^ 0xFF]))
EOF
  ev "a byte of ana's mp4 flipped at $FLIP, after her store took it by reference"
  down ben; mv "$W/ben/blobs" "$W/ben/blobs-$LAST"; LAST=D
  mkdir -p "$W/flip/$MP4"; ln -sf "$A/exp.mp4" "$W/flip/$MP4/exp.mp4"
  python3 "$HERE/rangeserver.py" "$W/flip" "$PORT" "$OUT/D-plant-range.jsonl" >"$W/flip/out" 2>&1 &
  echo $! >"$W/flip.pid"; ev "the first run's range server on $PORT over the flipped file, unverified"
  sleep 1
  drive chromium:mp4
  kill "$(cat "$W/flip.pid")"; rm -f "$W/flip.pid"; sleep 1
  PHASE=D; up ben -
  drive chromium:mp4
  listed ben >"$OUT/D-ben-folders.json"
  python3 - "$A/exp.mp4" "$FLIP" <<'EOF'
import sys
with open(sys.argv[1], "r+b") as f:
    f.seek(int(sys.argv[2])); b = f.read(1); f.seek(int(sys.argv[2])); f.write(bytes([b[0] ^ 0xFF]))
EOF
  ev "the byte flipped back"
fi

# stop <plant>: Ana's node with the plant; Ben fresh; the probe beside `unshare`.
stop_watched() {
  down ana; up ana "$1"
  if ! listed ben >/dev/null 2>&1 || ! listed ben | grep -q "$MKV"; then
    SVRNGS_DIR="$W/ana" "$B" share --folder "$A" >"$OUT/$PHASE-share.json"; ev "ana shared again"
  fi
  fresh_ben -
  lists ben "any(f['person'] == 'Ana' for f in l['folders'])"
  local size; size="$(python3 -c "import json;print([json.loads(l)['size'] for l in open('$OUT/media.jsonl') if l and json.loads(l)['path'].endswith('.mkv')][0])")"
  python3 "$HERE/probe.py" "http://127.0.0.1:$PORT/" "$MKV" "$size" 8 "$OUT/$PHASE-probe.jsonl" &
  local probe=$!
  sleep 1
  ev "unshare called"
  SVRNGS_DIR="$W/ana" "$B" unshare --folder films >"$OUT/$PHASE-unshare.json"
  ev "unshare returned"
  lists ben "not any(f['person'] == 'Ana' for f in l['folders'])"
  ev "ben's list no longer names ana's folder"
  wait $probe || true
}
if phase C; then
  PHASE=C-plant; stop_watched unchecked
  PHASE=C; stop_watched -
fi

# B7's scan: Ben's data dir, every phase's store, before P writes an act
# naming the planted file to the group's record.
PHASE=scan; python3 "$HERE/analyze.py" scan "$OUT" "$W/ben" ben | cut -c1-300 >&2
if phase P; then
  PHASE=P-plant; down ana; up ana follow-links
  SVRNGS_DIR="$W/ana" "$B" share --folder "$A" >"$OUT/P-plant-share.json"; ev "ana shared again, following links"
  LINKED="$(python3 -c "import json;print([f['hash'] for f in json.load(open('$OUT/P-plant-share.json'))['files'] if f['path'] == 'link.bin'][0])")"
  fresh_ben -
  lists ben "any(x['hash'] == '$LINKED' for f in l['folders'] for x in f['files'])"
  curl -sS -A gf-plant -o "$W/P-plant-got" -w '%{http_code} %{size_download}\n' "http://127.0.0.1:$PORT/$LINKED" | tee "$OUT/P-plant-fetch.txt"
  down ben; mv "$W/ben/blobs" "$W/ben/blobs-P-plant"; LAST=done
fi

PHASE=down
cp "$W/jf/log/"*.log "$OUT/" 2>/dev/null || true
cleanup
python3 "$HERE/analyze.py" "$OUT" "$W" | tee "$OUT/summary.txt"
