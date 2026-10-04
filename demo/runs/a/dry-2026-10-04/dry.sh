#!/usr/bin/env bash
# Demo A's dry run (demo/a.md "Before"; DESIGN §11 risk 7), on one host: a
# scratch Jellyfin on 127.0.0.1:8096, Sam's computer and Alex's as two data
# dirs, Alex's browser a headless Chrome recording a HAR. What it measures is
# what Jellyfin's own web player needs to play through the door without a
# member typing a password, and whether any byte received holds a secret.
#
# The Jellyfin here is the macOS app's server, run with its own scratch
# dirs (upstream commonwealth-ai scripts/cw-media-demo.sh:8-140 runs the
# podman image; this host's podman machine was stopped, and the app is the
# same 12.0.0 server). Nothing of it outlives the run but the data below.
#
# usage: dry.sh <scratch dir> <out dir>   (svrngs built: cargo build -p svrngs)
set -euo pipefail
R="$1"; OUT="$2"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../../.." && pwd)"
B="$REPO/target/debug/svrngs"
JF_APP="${JF_APP:-/Applications/Jellyfin.app/Contents}"
export CHROME="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
export PLAYWRIGHT_CORE="${PLAYWRIGHT_CORE:-$HOME/dev/commonwealth-ai/sovereign/crates/sovereign-desktop/node_modules/playwright-core}"
export SECRETS="$R/secrets"
O=http://127.0.0.1:8096
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$R/steps.log" >&2; }

stop() { for p in "$R"/alex/pid "$R"/sam/pid "$R"/jf/pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done; }
trap stop EXIT
stop; rm -rf "$R"; mkdir -p "$R"/jf/{data,config,cache,log,media} "$R"/sam "$R"/alex "$OUT"
(umask 077; mkdir -p "$SECRETS")

# --- Sam's Jellyfin: three films, Sam's admin account, one viewer account --
for n in One Two Three; do
  "$JF_APP/MacOS/ffmpeg" -v error -y -f lavfi -i "testsrc2=size=1280x720:rate=30" \
    -f lavfi -i "sine=frequency=440:sample_rate=48000" -t 90 -c:v libx264 -preset veryfast \
    -b:v 3M -pix_fmt yuv420p -c:a aac -b:a 128k -movflags +faststart "$R/jf/media/Test Pattern $n (2026).mp4"
done
cat > "$R/jf/config/network.xml" <<'EOF'
<?xml version="1.0" encoding="utf-8"?>
<NetworkConfiguration xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <EnableIPv6>false</EnableIPv6>
  <LocalNetworkAddresses><string>127.0.0.1</string></LocalNetworkAddresses>
  <EnableRemoteAccess>false</EnableRemoteAccess>
</NetworkConfiguration>
EOF
nohup "$JF_APP/MacOS/jellyfin" -d "$R/jf/data" -c "$R/jf/config" -C "$R/jf/cache" -l "$R/jf/log" \
  -w "$JF_APP/Resources/jellyfin-web" --ffmpeg "$JF_APP/MacOS/ffmpeg" --nonetchange >"$R/jf/stdout.log" 2>&1 &
echo $! >"$R/jf/pid"
ok=0; for _ in $(seq 1 90); do
  # Kestrel answers before the app is ready: the document twice in a row.
  if info="$(curl -sS -f --max-time 2 $O/System/Info/Public 2>/dev/null)" && [[ "$info" == *StartupWizardCompleted* ]]; then
    ok=$((ok + 1)); [ $ok -ge 2 ] && break
  else
    ok=0
  fi
  sleep 1
done
[ $ok -ge 2 ] || { say "Jellyfin did not come up"; exit 3; }
SERVER="$(printf '%s' "$info" | python3 -c 'import json,sys;print(json.load(sys.stdin)["Id"])')"
say "jellyfin up: $info"

CLI='MediaBrowser Client="dry-run", Device="Sam computer", DeviceId="dry-run-setup", Version="1"'
api() { curl -sS -f -X "$1" "$O$2" -H "Content-Type: application/json" -H "Authorization: $CLI${TOKEN:+, Token=\"$TOKEN\"}" ${3:+-d "$3"}; }
field() { python3 -c "import json,sys;print(json.load(sys.stdin)$1)"; }
(umask 077; head -c 18 /dev/urandom | base64 | tr -d '/+=' >"$SECRETS/sam_pw"
             head -c 18 /dev/urandom | base64 | tr -d '/+=' >"$SECRETS/viewer_pw")
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
(umask 077; printf '%s' "$TOKEN" >"$SECRETS/admin_token")
VIEWER="$(api POST /Users/New "{\"Name\":\"viewer\",\"Password\":\"$(cat "$SECRETS/viewer_pw")\"}" | field '["Id"]')"
api POST /Library/Refresh >/dev/null
for _ in $(seq 1 30); do
  n="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie" | field '["TotalRecordCount"]')"; [ "$n" -ge 3 ] && break; sleep 1
done
users() { api GET /Users | python3 -c 'import json,sys;print(json.dumps(sorted(u["Name"] for u in json.load(sys.stdin))))'; }
users >"$OUT/users-before.json"
ITEM="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie&SearchTerm=One" | field '["Items"][0]["Id"]')"
say "films: $n; users: $(cat "$OUT/users-before.json")"

# --- the lend's secret: Sam's computer signs the viewer in once ------------
# A viewer's token, not an API key: probe_api_key below is why.
LEND_DEVICE='MediaBrowser Client="svrngs lend", Device="Sam computer", DeviceId="svrngs-lend-sam", Version="1"'
mint() {
  local t
  t="$(curl -sS -f -X POST $O/Users/AuthenticateByName -H "Content-Type: application/json" -H "Authorization: $LEND_DEVICE" \
       -d "{\"Username\":\"viewer\",\"Pw\":\"$(cat "$SECRETS/viewer_pw")\"}" | field '["AccessToken"]')"
  (umask 077; echo "$t" >>"$SECRETS/lend_tokens"; printf '%s, Token="%s"' "$LEND_DEVICE" "$t" >"$SECRETS/lend_secret")
  SVRNGS_DIR="$R/sam" NO_COLOR=1 "$B" share 8096 --upload 20 <"$SECRETS/lend_secret" >>"$R/sam/share.out" 2>&1
}

# --- Sam makes House; Alex joins by the client's link ----------------------
SVRNGS_DIR="$R/sam" RUST_LOG=info NO_COLOR=1 nohup "$B" new House --as Sam >"$R/sam/new.out" 2>"$R/sam/run.log" &
echo $! >"$R/sam/pid"
for _ in $(seq 1 30); do grep -q 'svrngs://join/' "$R/sam/new.out" 2>/dev/null && break; sleep 1; done
LINK="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$R/sam/new.out")"
say "sam: $(head -1 "$R/sam/new.out")"
SVRNGS_DIR="$R/alex" RUST_LOG=info NO_COLOR=1 nohup "$B" join "$LINK" --as Alex >"$R/alex/join.out" 2>"$R/alex/join.log" &
echo $! >"$R/alex/pid"
for _ in $(seq 1 60); do grep -q "You're in" "$R/alex/join.out" 2>/dev/null && break; sleep 1; done
say "alex: $(head -1 "$R/alex/join.out")"
mint
say "sam shared 8096: $(grep -c Lent "$R/sam/share.out") lend line(s)"
# The bare command prints the lends, asked as it starts: Alex runs it now.
kill "$(cat "$R/alex/pid")"; sleep 2
SVRNGS_DIR="$R/alex" RUST_LOG=info NO_COLOR=1 nohup "$B" >"$R/alex/run.out" 2>"$R/alex/run.log" &
echo $! >"$R/alex/pid"
for _ in $(seq 1 60); do grep -q '/lend/' "$R/alex/run.out" 2>/dev/null && break; sleep 1; done
U="$(grep -o 'http://127.0.0.1:[0-9]*/lend/jellyfin/' "$R/alex/run.out")"
say "alex's local address: $U"
door() { curl -sS -o /dev/null -w '%{http_code}' "$U$1"; }

# --- Alex's browser, three ways --------------------------------------------
export ITEM SERVER
node "$HERE/drive.mjs" none "$U" "$R/runs/none" >/dev/null
SEED="{\"address\":\"${U%/}\",\"serverName\":\"jellyfin\",\"serverId\":\"$SERVER\",\"userId\":\"$VIEWER\"}" \
  node "$HERE/drive.mjs" seed "$U" "$R/runs/seed" >/dev/null
before_signin="$(door Users/Me)"
node "$HERE/drive.mjs" signin "$U" "$R/runs/signin" >/dev/null
after_signin="$(door Users/Me)"
mint
after_remint="$(door Users/Me)"
say "door /Users/Me: before sign-in $before_signin, after $after_signin, after Sam declares again $after_remint"

# --- an API key as the lend's secret instead -------------------------------
api POST "/Auth/Keys?app=svrngs-lend" >/dev/null
KEY="$(api GET /Auth/Keys | python3 -c 'import json,sys;print([k["AccessToken"] for k in json.load(sys.stdin)["Items"] if k["AppName"]=="svrngs-lend"][0])')"
(umask 077; printf '%s' "$KEY" >"$SECRETS/api_key")
printf 'MediaBrowser Token="%s"' "$KEY" | SVRNGS_DIR="$R/sam" "$B" share 8096 --upload 20 >/dev/null 2>&1
key_me="$(door Users/Me)"
key_keys="$(curl -sS "${U}Auth/Keys" | grep -c "$KEY" || true)"
made="$(curl -sS -X POST -H 'content-type: application/json' -d '{"Name":"made-by-a-member"}' "${U}Users/New" | field '.get("Id","")' || true)"
users >"$OUT/users-with-api-key-lend.json"
[ -n "$made" ] && api DELETE "/Users/$made" >/dev/null
api DELETE "/Auth/Keys/$KEY" >/dev/null
SVRNGS_DIR="$R/sam" "$B" share 8096 --upload 20 <"$SECRETS/lend_secret" >/dev/null 2>&1
say "api key lend: /Users/Me $key_me; /Auth/Keys bodies holding the key $key_keys; a member made a user: ${made:+yes}"

# --- Jellyfin's live socket through the door -------------------------------
WS=(-H "Connection: Upgrade" -H "Upgrade: websocket" -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==")
ws_door="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 3 "${WS[@]}" "${U}socket" || true)"
ws_direct="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 3 "${WS[@]}" -H "Authorization: $(cat "$SECRETS/lend_secret")" $O/socket || true)"
say "websocket upgrade: through the door $ws_door, on Sam's computer directly $ws_direct"
users >"$OUT/users-after.json"

python3 - "$OUT/probes.json" <<EOF
import json, sys
json.dump({
  "door_users_me": {"before_signin": "$before_signin", "after_signin": "$after_signin", "after_sam_declares_again": "$after_remint"},
  "api_key_lend": {"users_me": "$key_me", "auth_keys_bodies_holding_the_key": int("$key_keys" or 0), "member_made_a_user": bool("$made")},
  "websocket_upgrade": {"through_the_door": "$ws_door", "direct_with_the_lend_secret": "$ws_direct"},
  "users_unchanged": open("$OUT/users-before.json").read() == open("$OUT/users-after.json").read(),
}, open(sys.argv[1], "w"), indent=1)
EOF
python3 "$HERE/scan.py" "$R" "$OUT" >/dev/null
cp "$R"/runs/*/*.png "$OUT/"
cp "$R/alex/run.out" "$OUT/alex-bare-command.out"; cp "$R/alex/join.out" "$OUT/alex-join.out"
sed 's#svrngs://join/[A-Za-z0-9_-]*#svrngs://join/<the link>#' "$R/sam/new.out" >"$OUT/sam-new.out"
cp "$R/sam/share.out" "$OUT/sam-share.out"; cp "$R/steps.log" "$OUT/steps.log"
say "done: $OUT"
