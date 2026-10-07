#!/usr/bin/env bash
# u6-2 (DESIGN §3, "The room", call 4): Jellyfin's own client for a
# screen through the room door, on this host, as
# demo/runs/u6/2026-10-06-jellyfin-screen/README.md says. Stands on
# u5-4's check.sh: its `up` makes Sam's scratch Jellyfin, lent as `share`
# lends it, with accounts for `dee` and `ana` made in Jellyfin's pages;
# then Dee's computer, a data dir of its own, joins Sam's group by the
# link and runs `svrngs room jellyfin`, the room door (u6-1).
#
#   screen.sh up   <scratch> <out>          all of the above; MEDIA as check.sh's
#   screen.sh run  <scratch> <out> <part>   screen.mjs's part a|b|c|d|control, writes <out>/<part>[-$RUN].json:
#                                           a and d on the screen that signed in typing (profile screen-a),
#                                           b and c on the one Quick Connect signed in (screen-b); c's phone
#                                           signed in as PHONE_AS (else ana), and with DIRECT=1 c's baseline
#                                           with no svrngs: the screen (screen-direct) and the phone straight
#                                           at Sam's Jellyfin
#   screen.sh night <scratch> <out> <part>  Demo B's dry run (u6-9), night.mjs's part tv|five, after up with
#                                           PEOPLE=room: tv signs the TV (profile tv) in by Quick Connect from
#                                           Dee's phone as room; five has Ana, Bo, Cy, Di and Ed, one at a time,
#                                           put a film on it; writes <out>/night-<part>[-$RUN].json, the TV's
#                                           requests and each phone's responses, by time, status, host and path;
#                                           WHO names some of the five (else all), and with DIRECT=1 five is the
#                                           baseline with no svrngs: the TV (profile tv-direct) and the phones
#                                           straight at Sam's Jellyfin
#   screen.sh bounce <scratch> <out>        the room door stopped GAP s (else 15) and started again; d runs
#                                           it with BOUNCE_DOOR=1
#   screen.sh logs <scratch> <out>          the room door's, Sam's door's and Jellyfin's logs, and Jellyfin's
#                                           sessions, into <out>, the secrets masked
#   screen.sh down <scratch> <out>          stops everything
#
# Each step stays inside ten minutes on this host. svrngs built first:
# cargo build -p svrngs. scripts/check.sh runs none of this: it needs
# Jellyfin.
set -euo pipefail
STEP="$1"; R="$2"; OUT="$3"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../.." && pwd)"
B="$REPO/target/debug/svrngs"
mkdir -p "$OUT"
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
# Every token and secret this run holds, out of what is kept.
mask() { python3 -c '
import re, sys, pathlib
s = sys.stdin.read()
for f in pathlib.Path(sys.argv[1]).iterdir():
    v = f.read_text().strip()
    if v: s = s.replace(v, "<" + f.name + ">")
s = re.sub(r"(?i)(api_key|apikey|token|secret|pw)([=:\" ]+)[A-Za-z0-9_-]{16,}", r"\1\2<masked>", s)
s = re.sub(r"(svrngs://join/|#)[A-Za-z0-9_-]{40,}", r"\1<the link>", s)
sys.stdout.write(s)' "$R/secrets"; }

up() {
  PEOPLE="${PEOPLE:-dee ana}" "$HERE/check.sh" up "$R" "$OUT"
  mkdir -p "$R/dee"
  local client; client="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$R/sam/new.out")"
  SVRNGS_DIR="$R/dee" NO_COLOR=1 nohup "$B" join "$client" --as Dee >"$R/dee/join.out" 2>"$R/dee/join.log" &
  echo $! >"$R/dee.pid"
  for _ in $(seq 1 60); do grep -q "You're in" "$R/dee/join.out" 2>/dev/null && break; sleep 1; done
  say "dee: $(head -1 "$R/dee/join.out")"
  kill "$(cat "$R/dee.pid")"; sleep 2
  room_up
  cp "$R/dee/room.out" "$OUT/dee-room.out"
  echo "ROOM=$ROOM" >>"$R/env"
}

# The room door, its every request traced (room=debug), the lend's name
# as `share` named it; its log kept across a bounce.
room_up() {
  : >"$R/dee/room.out"
  SVRNGS_DIR="$R/dee" NO_COLOR=1 RUST_LOG=info,room=debug,door=debug nohup "$B" room jellyfin >"$R/dee/room.out" 2>>"$R/dee/room.log" &
  echo $! >"$R/dee.pid"
  for _ in $(seq 1 60); do grep -q 'http://' "$R/dee/room.out" 2>/dev/null && break; sleep 1; done
  ROOM="$(grep -o 'http://[0-9.]*:[0-9]*/' "$R/dee/room.out" | head -1)"
  [ -n "$ROOM" ] || { cat "$R/dee/room.out" "$R/dee/room.log" >&2; exit 3; }
  say "dee's computer serves it to the room at $ROOM"
}

# The room door stopped, and started again GAP s later (else 15), as Dee's
# computer asleep a while: part d runs it while the screen is open.
bounce() {
  kill "$(cat "$R/dee.pid")"
  say "the room door stopped"
  sleep "${GAP:-15}"
  room_up
}

run() {
  # shellcheck disable=SC1091
  . "$R/env"
  local part="$1" tag="$1${RUN:+-$RUN}" profile="$R/screen-a" room="$ROOM" direct=false
  case "$part" in b|c) profile="$R/screen-b" ;; esac
  [ -n "${DIRECT:-}" ] && { profile="$R/screen-direct"; room=http://127.0.0.1:8096/web/; direct=true; }
  BOUNCE="${BOUNCE_DOOR:+$HERE/screen.sh bounce $R $OUT}" SECRETS="$R/secrets" JF=http://127.0.0.1:8096 ADMIN="$(cat "$R/secrets/admin_token")" DUMP="$R/dump/$tag" \
    node "$HERE/screen.mjs" "$part" "{\"room\":\"$room\",\"link\":\"$LINK\",\"server\":\"$SERVER\",\"film\":\"$ONE\",
      \"profile\":\"$profile\",\"as\":\"ana\",\"phone_as\":\"${PHONE_AS:-ana}\",\"direct\":$direct}" \
    2>"$R/$tag.err" | mask >"$OUT/$tag.json" || true
  [ -f "$R/dump/$tag/screen-$part.jsonl" ] && mask <"$R/dump/$tag/screen-$part.jsonl" >"$OUT/$tag-screen-requests.jsonl"
  say "$tag: $(head -c 600 "$OUT/$tag.json")"
}

# Demo B's dry run (u6-9; demo/runs/b/2026-10-06/README.md): the TV a
# profile of its own at the room address, all signed in as the room's one
# account; the five in WebKit as an iPhone and Chromium as a Pixel 7, by
# turns, each picking the next of Sam's three films.
night() {
  # shellcheck disable=SC1091
  . "$R/env"
  local part="$1" tag="night-$1${RUN:+-$RUN}" i=0 films=("Test Pattern One" "Test Pattern Two" "Test Pattern Three") people=""
  local room="$ROOM" profile="$R/tv" direct=false
  [ -n "${DIRECT:-}" ] && { room=http://127.0.0.1:8096/web/; profile="$R/tv-direct"; direct=true; }
  for p in Ana:webkit: Bo:chromium:Pixel\ 7 Cy:webkit: Di:chromium:Pixel\ 7 Ed:webkit:; do
    IFS=: read -r name engine device <<<"$p"
    i=$((i + 1))
    [ -z "${WHO:-}" ] || [[ " $WHO " == *" $name "* ]] || continue
    device="${device:+\"$device\"}"
    people="$people${people:+,}{\"name\":\"$name\",\"user\":\"$(tr A-Z a-z <<<"$name")\",\"engine\":\"$engine\",\"device\":${device:-null},\"film\":\"${films[$(((i - 1) % 3))]}\"}"
  done
  rm -rf "$R/dump/$tag"
  SECRETS="$R/secrets" JF=http://127.0.0.1:8096 ADMIN="$(cat "$R/secrets/admin_token")" DUMP="$R/dump/$tag" \
    node "$HERE/night.mjs" "$part" "{\"room\":\"$room\",\"link\":\"$LINK\",\"server\":\"$SERVER\",\"profile\":\"$profile\",
      \"account\":\"room\",\"direct\":$direct,\"people\":[$people]}" 2>"$R/$tag.err" | mask >"$OUT/$tag.json" || true
  [ -f "$R/dump/$tag/screen-$part.jsonl" ] && mask <"$R/dump/$tag/screen-$part.jsonl" >"$OUT/$tag-tv-requests.jsonl"
  for d in "$R/dump/$tag"/*/; do
    [ -f "$d/index.jsonl" ] || continue
    python3 -c '
import json, sys, urllib.parse as u
for line in open(sys.argv[1]):
    e = json.loads(line); x = u.urlsplit(e["url"])
    if x.scheme not in ("http", "https"): continue
    print(json.dumps({"ms": e["ms"], "status": e["status"], "host": x.netloc, "path": x.path + ("?" + x.query if x.query else "")}))' "$d/index.jsonl" \
      | sed -E 's#[a-z0-9]{20,}\.localhost#<label>.localhost#g' | mask >"$OUT/$tag-$(basename "$d")-requests.jsonl"
  done
  say "$tag: $(head -c 600 "$OUT/$tag.json")"
}

logs() {
  mask <"$R/dee/room.log" >"$OUT/dee-room.log"
  # Sam's at INFO and above: the person and path of each request its door
  # let in; its carriage trace (DEBUG) runs to megabytes and stays behind.
  grep -v ' DEBUG ' "$R/sam/run.log" | mask >"$OUT/sam-run.log"
  cat "$R"/jf/log/*.log | mask >"$OUT/jellyfin.log"
  curl -sS -H "Authorization: MediaBrowser Client=\"u6-2\", Device=\"Sam computer\", DeviceId=\"u5-4-setup\", Version=\"1\", Token=\"$(cat "$R/secrets/admin_token")\"" \
    http://127.0.0.1:8096/Sessions | mask >"$OUT/jellyfin-sessions.json"
  say "logs kept: $(wc -l <"$OUT/dee-room.log") lines of the room door's, $(wc -l <"$OUT/sam-run.log") of Sam's, $(wc -l <"$OUT/jellyfin.log") of Jellyfin's"
}

case "$STEP" in
  up) up ;;
  run) run "$4" ;;
  night) night "$4" ;;
  logs) logs ;;
  bounce) bounce ;;
  down) "$HERE/check.sh" down "$R" "$OUT" ;;
  *) echo "usage: screen.sh up|run|night|logs|down <scratch> <out> [part]" >&2; exit 2 ;;
esac
