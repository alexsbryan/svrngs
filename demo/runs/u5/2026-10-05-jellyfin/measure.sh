#!/usr/bin/env bash
# u5-3 (README.md, the rule and bars stated before the data): stock
# Jellyfin through a lend, on one host. A scratch Jellyfin on
# 127.0.0.1:8096 — U3's dry run's (demo/runs/a/dry-2026-10-04/dry.sh), the
# macOS app's 12.0.0 server with its own data dirs — with two films, the
# lender's admin account and two accounts the lender made; Sam's computer
# and Alex's as two data dirs running the `svrngs` command through iroh's
# public relays; the page built and served on localhost, where every name
# answers, so a phone's apps each open at an address of their own (u5-2).
#
#   measure.sh up    <scratch> <out>                 Jellyfin, the page, Sam lends 8096 with no secret, Alex joins
#   measure.sh pair  <scratch> <out> <way> <engine>  (a)-(c): two people through the lend, way computer|phone,
#                                                    engine chromium|webkit; writes <out>/pair-<way>-<engine>.json,
#                                                    or with RUN=<n> pair-<way>-<engine>-<n>.json, a run again kept apart
#   measure.sh probe <scratch> <out>                 (c) on a lend holding a token, and (d): writes <out>/probe.json
#   measure.sh down  <scratch> <out>                 stops everything; the scratch dir stays for the secrets
#   measure.sh all   <scratch> <out>                 every step above, in order
#
# Each step stays inside ten minutes on this host; `all` does not. svrngs
# built first: cargo build -p svrngs. Nothing of Jellyfin outlives `down`
# but what is under <out>; the secrets stay in <scratch>/secrets.
set -euo pipefail
STEP="$1"; R="$2"; OUT="$3"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../../.." && pwd)"
B="$REPO/target/debug/svrngs"
JF_APP="${JF_APP:-/Applications/Jellyfin.app/Contents}"
O=http://127.0.0.1:8096
SECRETS="$R/secrets"; mkdir -p "$OUT"
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
field() { python3 -c "import json,sys;print(json.load(sys.stdin)$1)"; }
port_of() { for _ in $(seq 1 100); do grep -m1 -o 'port [0-9]*' "$1" 2>/dev/null | cut -d' ' -f2 && return; sleep 0.1; done; return 1; }
CLI='MediaBrowser Client="u5-3 measure", Device="Sam computer", DeviceId="u5-3-setup", Version="1"'
api() { curl -sS -f -X "$1" "$O$2" -H "Content-Type: application/json" -H "Authorization: $CLI${TOKEN:+, Token=\"$TOKEN\"}" ${3:+-d "$3"}; }
users() { api GET /Users | python3 -c 'import json,sys;print(json.dumps(sorted(u["Name"] for u in json.load(sys.stdin))))'; }
secret() { (umask 077; printf '%s' "$2" >"$SECRETS/$1"); }
pw() { head -c 18 /dev/urandom | base64 | tr -d '/+='; }

down() {
  for p in "$R"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done
  rm -f "$R"/*.pid
}

up() {
  down; rm -rf "$R"; mkdir -p "$R"/jf/{data,config,cache,log,media} "$R"/sam "$R"/alex "$OUT"
  (umask 077; mkdir -p "$SECRETS")
  : >"$OUT/steps.log"

  # --- Sam's Jellyfin: one film it plays as it is, one it must convert -----
  # 120 s each at 720p, a keyframe each 2 s. Two is MPEG-2 in Matroska,
  # which neither engine plays, so Jellyfin converts it for both.
  local ff="$JF_APP/MacOS/ffmpeg"
  "$ff" -v error -y -f lavfi -i "testsrc2=size=1280x720:rate=24" -f lavfi -i "sine=frequency=440:sample_rate=48000" \
    -t 120 -c:v libx264 -preset veryfast -b:v 3M -g 48 -pix_fmt yuv420p -c:a aac -b:a 128k -movflags +faststart \
    "$R/jf/media/Test Pattern One (2026).mp4"
  "$ff" -v error -y -f lavfi -i "testsrc=size=1280x720:rate=24" -f lavfi -i "sine=frequency=660:sample_rate=48000" \
    -t 120 -c:v mpeg2video -b:v 4M -g 48 -c:a mp2 -b:a 192k "$R/jf/media/Test Pattern Two (2026).mkv"
  cat >"$R/jf/config/network.xml" <<'EOF'
<?xml version="1.0" encoding="utf-8"?>
<NetworkConfiguration xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <EnableIPv6>false</EnableIPv6>
  <LocalNetworkAddresses><string>127.0.0.1</string></LocalNetworkAddresses>
  <EnableRemoteAccess>false</EnableRemoteAccess>
</NetworkConfiguration>
EOF
  nohup "$JF_APP/MacOS/jellyfin" -d "$R/jf/data" -c "$R/jf/config" -C "$R/jf/cache" -l "$R/jf/log" \
    -w "$JF_APP/Resources/jellyfin-web" --ffmpeg "$ff" --nonetchange >"$R/jf/stdout.log" 2>&1 &
  echo $! >"$R/jf.pid"
  local ok=0 info
  for _ in $(seq 1 90); do
    # Kestrel answers before the app is ready: the document twice in a row.
    if info="$(curl -sS -f --max-time 2 $O/System/Info/Public 2>/dev/null)" && [[ "$info" == *StartupWizardCompleted* ]]; then
      ok=$((ok + 1)); [ $ok -ge 2 ] && break
    else
      ok=0
    fi
    sleep 1
  done
  [ $ok -ge 2 ] || { say "Jellyfin did not come up"; exit 3; }
  SERVER="$(printf '%s' "$info" | field '["Id"]')"
  say "jellyfin up: $info"

  secret sam_pw "$(pw)"; secret ana_pw "$(pw)"; secret ben_pw "$(pw)"
  TOKEN=""
  api POST /Startup/Configuration '{"UICulture":"en-US","MetadataCountryCode":"US","PreferredMetadataLanguage":"en"}' >/dev/null
  api GET /Startup/User >/dev/null
  api POST /Startup/User "{\"Name\":\"sam\",\"Password\":\"$(cat "$SECRETS/sam_pw")\"}" >/dev/null
  api POST "/Library/VirtualFolders?name=Movies&collectionType=movies&refreshLibrary=true" \
    "{\"LibraryOptions\":{\"PathInfos\":[{\"Path\":\"$R/jf/media\"}],\"EnableRealtimeMonitor\":false}}" >/dev/null
  api POST /Startup/RemoteAccess '{"EnableRemoteAccess":false,"EnableAutomaticPortMapping":false}' >/dev/null
  api POST /Startup/Complete >/dev/null
  sleep 3
  TOKEN="$(api POST /Users/AuthenticateByName "{\"Username\":\"sam\",\"Pw\":\"$(cat "$SECRETS/sam_pw")\"}" | field '["AccessToken"]')"
  secret admin_token "$TOKEN"
  # The two accounts the lender makes, as sign in once asks of the lender.
  for who in ana ben; do
    api POST /Users/New "{\"Name\":\"$who\",\"Password\":\"$(cat "$SECRETS/${who}_pw")\"}" >/dev/null
  done
  api POST /Library/Refresh >/dev/null
  local n=0
  for _ in $(seq 1 60); do
    n="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie" | field '["TotalRecordCount"]')"; [ "$n" -ge 2 ] && break; sleep 1
  done
  ONE="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie&SearchTerm=One" | field '["Items"][0]["Id"]')"
  TWO="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie&SearchTerm=Two" | field '["Items"][0]["Id"]')"
  users >"$OUT/users-before.json"
  say "films: $n; users: $(cat "$OUT/users-before.json")"

  # --- the page, served on localhost: every name under it answers ----------
  "$REPO/scripts/build-web.sh" --out "$R/site" >"$R/build.log"
  node "$REPO/demo/runs/u4/2026-10-05/drive.mjs" serve "$R/site" >"$R/site.out" 2>&1 & echo $! >"$R/site.pid"
  PAGE="http://localhost:$(port_of "$R/site.out")/"

  # --- Sam makes House, its links on the page; Alex joins by the client's --
  SVRNGS_DIR="$R/sam" RUST_LOG=info NO_COLOR=1 nohup "$B" new House --as Sam --origin "$PAGE" >"$R/sam/new.out" 2>"$R/sam/run.log" &
  echo $! >"$R/sam.pid"
  for _ in $(seq 1 60); do grep -q "$PAGE#" "$R/sam/new.out" 2>/dev/null && break; sleep 1; done
  LINK="$(grep -o "$PAGE#[A-Za-z0-9_-]*" "$R/sam/new.out")"
  local client; client="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$R/sam/new.out")"
  say "sam: $(grep -i 'phone' "$R/sam/new.out" || head -1 "$R/sam/new.out")"
  SVRNGS_DIR="$R/alex" RUST_LOG=info NO_COLOR=1 nohup "$B" join "$client" --as Alex >"$R/alex/join.out" 2>"$R/alex/join.log" &
  echo $! >"$R/alex.pid"
  for _ in $(seq 1 60); do grep -q "You're in" "$R/alex/join.out" 2>/dev/null && break; sleep 1; done
  say "alex: $(head -1 "$R/alex/join.out")"
  # No secret: sign in once's lend. Eight at once, for the phones.
  SVRNGS_DIR="$R/sam" NO_COLOR=1 "$B" share 8096 --limit 8 </dev/null >"$R/sam/share.out" 2>&1
  say "sam: $(grep -m1 Lent "$R/sam/share.out")"
  # The bare command prints the lends as it starts: Alex runs it now.
  kill "$(cat "$R/alex.pid")"; sleep 2
  SVRNGS_DIR="$R/alex" RUST_LOG=info NO_COLOR=1 nohup "$B" >"$R/alex/run.out" 2>"$R/alex/run.log" &
  echo $! >"$R/alex.pid"
  for _ in $(seq 1 60); do grep -q 'localhost:[0-9]*/' "$R/alex/run.out" 2>/dev/null && break; sleep 1; done
  ORIGIN="$(grep -o 'http://[a-z0-9-]*\.localhost:[0-9]*/' "$R/alex/run.out" | head -1)"
  say "alex's computer opens it at $ORIGIN"
  [ -n "$ORIGIN" ] || { cat "$R/alex/run.out" >&2; exit 3; }

  cat >"$R/env" <<EOF
SERVER=$SERVER
ONE=$ONE
TWO=$TWO
PAGE=$PAGE
LINK=$LINK
ORIGIN=$ORIGIN
EOF
  cp "$R/alex/run.out" "$OUT/alex-bare-command.out"
  sed -E 's#(svrngs://join/|'"$PAGE"'\#)[A-Za-z0-9_-]+#\1<the link>#g' "$R/sam/new.out" >"$OUT/sam-new.out"
  cp "$R/sam/share.out" "$OUT/sam-share.out"
}

pair() {
  # shellcheck disable=SC1091
  . "$R/env"
  local way="$1" engine="$2"; local tag="$way-$engine${RUN:+-$RUN}"
  local at="$ORIGIN"; [ "$way" = phone ] && at="$LINK"
  # Each phone joins as a person of its own: each engine's two are new names.
  local a=Ana b=Ben; [ "$engine" = webkit ] && { a=Cleo; b=Dara; }
  [ -n "${RUN:-}" ] && { a="$a$RUN"; b="$b$RUN"; }
  rm -rf "$R/dump/$tag"
  SECRETS="$SECRETS" JF="$O" ADMIN="$(cat "$SECRETS/admin_token")" DUMP="$R/dump/$tag" \
    node "$HERE/drive.mjs" "{\"way\":\"$way\",\"engine\":\"$engine\",\"at\":\"$at\",\"server\":\"$SERVER\",
      \"people\":[{\"name\":\"$a\",\"user\":\"ana\",\"film\":\"$ONE\",\"seeks\":[50,15,80,30,65]},
                  {\"name\":\"$b\",\"user\":\"ben\",\"film\":\"$TWO\",\"seeks\":[60,20,75]}]}" \
    >"$OUT/pair-$tag.json" 2>"$R/pair-$tag.err" || true
  say "pair $tag: $(python3 "$HERE/summary.py" --line "$OUT/pair-$tag.json")"
}

case "$STEP" in
  up) up ;;
  pair) pair "$4" "$5" ;;
  probe) . "$R/env"; SECRETS="$SECRETS" JF="$O" B="$B" R="$R" OUT="$OUT" python3 "$HERE/probe.py" ;;
  down) TOKEN="$(cat "$SECRETS/admin_token")"; users >"$OUT/users-after.json" 2>/dev/null || true; down ;;
  all)
    up
    for way in computer phone; do for engine in chromium webkit; do pair "$way" "$engine"; done; done
    . "$R/env"; SECRETS="$SECRETS" JF="$O" B="$B" R="$R" OUT="$OUT" python3 "$HERE/probe.py"
    TOKEN="$(cat "$SECRETS/admin_token")"; users >"$OUT/users-after.json"; down ;;
  *) echo "usage: measure.sh up|pair|probe|down|all <scratch> <out> [way engine]" >&2; exit 2 ;;
esac
