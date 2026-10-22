#!/usr/bin/env bash
# oidc-jellyfin's run, second take (ralph/next/oidc/STATE.md;
# SIGN_IN_WITH_SVRNGS §5a, §7 2): a real Jellyfin 10.11.11 on this host with
# the stock SSO plugin 4.0.0.4, lent by Kim with
# `svrngs share 18096 --sign-in-with-svrngs`; Alex on another computer, Bo on
# a WebKit phone, Cy a member Kim removes.
#
# The first take (../2026-10-21-jellyfin/) failed bars 1-3 because the plugin
# built its callback page from `Request.Host` and the door asks the service at
# its own address. This take has the door telling the browser's origin in
# `X-Forwarded-Host`/`-Proto` (1efa9a2d) and Jellyfin's stock `KnownProxies`
# naming this computer, as for a reverse proxy. The predictions are
# PREDICTIONS.md, committed before this and before any data.
#
#   run.sh [W]        the whole run, its data under data/; W is the
#                     scratch (else /tmp/oidc-run)
#
# One host, one run of each: not a rate (AGENTS.md 7). A step that did not
# happen is null with what the page showed, never a number guessed.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../../.." && pwd)"
W="${1:-${W:-/tmp/oidc-run}}"; OUT="$HERE/data"
B="$REPO/target/debug/svrngs"
JF_PORT=18096; O="http://127.0.0.1:$JF_PORT"
JF_TAR='https://repo.jellyfin.org/files/server/macos/stable/v10.11.11/arm64/jellyfin_10.11.11-arm64.tar.xz'
JF_SHA=27c0bcbeac7a698cf0fd034e347f124f69083bc4463816099b52bebead614a47
SSO_ZIP='https://github.com/9p4/jellyfin-plugin-sso/releases/download/v4.0.0.4/sso-authentication_4.0.0.4.zip'
SSO_SHA=c09f16ba31059a434ddd7f811e4f9608d4b4c4514cc80a5bf1ca33bee61e1107
export NO_COLOR=1
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
pw() { head -c 18 /dev/urandom | base64 | tr -d '/+='; }
port_of() { for _ in $(seq 1 100); do grep -m1 -o 'port [0-9]*' "$1" 2>/dev/null | cut -d' ' -f2 && return; sleep 0.1; done; return 1; }
down() {
  for p in "$W"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done
  rm -f "$W"/*.pid
}
trap down EXIT

api() { curl -sS -f -X "$1" "$O$2" -H "Content-Type: application/json" -H "Authorization: $CLI${TOKEN:+, Token=\"$TOKEN\"}" ${3:+-d "$3"}; }
CLI='MediaBrowser Client="oidc-jellyfin run", Device="Kim computer", DeviceId="oidc-jellyfin-setup", Version="1"'
field() { python3 -c "import json,sys;print(json.load(sys.stdin)$1)"; }
secret() { (umask 077; printf '%s' "$2" >"$W/secrets/$1"); }
starts() { # starts <pidfile> <log> <command...>
  local pid="$1" log="$2"; shift 2
  "$@" >"$log" 2>&1 & echo $! >"$pid"
}

rm -rf "$OUT" "$W"/kim "$W"/alex "$W"/bo "$W"/cy "$W"/jf/data "$W"/jf/config "$W"/jf/cache "$W"/jf/log "$W"/env "$W"/env.json
mkdir -p "$OUT" "$W"/pkg "$W"/jf/{data,config,cache,log,media,kept} "$W/lib/Movies" "$W/site" "$W/secrets" "$W"/{kim,alex,bo,cy}
(umask 077; [ -f "$W/secrets/kim_pw" ] || pw >"$W/secrets/kim_pw")
: >"$OUT/steps.log"
say "host: $(uname -m) $(sw_vers -productVersion), load $(uptime | sed 's/.*load/load/')"
say "svrngs $(git -C "$REPO" rev-parse --short HEAD), $(ffmpeg -version | head -1)"

# --- the pinned packages ------------------------------------------------------
get() { # get <url> <sha256> <to>
  [ -s "$3" ] || curl -sSL --max-time 300 -o "$3.part" "$1"
  [ -s "$3" ] || mv "$3.part" "$3"
  local got; got="$(shasum -a 256 "$3" | cut -d' ' -f1)"
  [ "$got" = "$2" ] || { say "$3 is $got, not the pinned $2"; exit 3; }
}
get "$JF_TAR" "$JF_SHA" "$W/pkg/jellyfin.tar.xz"
get "$SSO_ZIP" "$SSO_SHA" "$W/pkg/sso.zip"
[ -d "$W/pkg/jellyfin" ] || tar -xf "$W/pkg/jellyfin.tar.xz" -C "$W/pkg"
# The tarball's apphost and its dlls carry no signature, which this arm64
# host refuses; signing them ad-hoc changes no code.
codesign --force --deep --sign - "$W/pkg/jellyfin" >/dev/null 2>&1 || true
[ -d "$W/pkg/sso" ] || { mkdir -p "$W/pkg/sso"; unzip -q -o "$W/pkg/sso.zip" -d "$W/pkg/sso"; }
say "pinned: $("$W/pkg/jellyfin/jellyfin" --version 2>&1 | head -1), SSO-Auth 4.0.0.4"

# --- the film and Jellyfin's dirs --------------------------------------------
[ -s "$W/lib/Movies/Test Pattern.mp4" ] || ffmpeg -v error -y -f lavfi -i testsrc2=size=640x360:rate=24 \
  -f lavfi -i sine=frequency=440:sample_rate=48000 -t 60 -c:v libx264 -preset veryfast -pix_fmt yuv420p \
  -c:a aac -b:a 96k -movflags +faststart "$W/lib/Movies/Test Pattern.mp4"
cat >"$W/jf/config/network.xml" <<XML
<?xml version="1.0" encoding="utf-8"?>
<NetworkConfiguration xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <InternalHttpPort>$JF_PORT</InternalHttpPort>
  <PublicHttpPort>$JF_PORT</PublicHttpPort>
  <EnableIPv6>false</EnableIPv6>
  <LocalNetworkAddresses><string>127.0.0.1</string></LocalNetworkAddresses>
  <EnableRemoteAccess>false</EnableRemoteAccess>
  <KnownProxies><string>127.0.0.1</string></KnownProxies>
</NetworkConfiguration>
XML
mkdir -p "$W/jf/data/plugins/SSO-Auth"
cp "$W/pkg/sso"/* "$W/jf/data/plugins/SSO-Auth/"
ffmpeg -version >"$OUT/ffmpeg.txt" 2>&1; shasum -a 256 "$W/lib/Movies/Test Pattern.mp4" >"$OUT/film.txt"

# --- Jellyfin up, Kim its admin, one library --------------------------------
starts "$W/jf.pid" "$W/jf/out" "$W/pkg/jellyfin/jellyfin" -d "$W/jf/data" -c "$W/jf/config" \
  -C "$W/jf/cache" -l "$W/jf/log" -w "$W/pkg/jellyfin/jellyfin-web" --nonetchange
info=""; ok=0
for _ in $(seq 1 90); do
  if info="$(curl -sS -f --max-time 2 "$O/System/Info/Public" 2>/dev/null)" && [[ "$info" == *StartupWizardCompleted* ]]; then
    ok=$((ok + 1)); [ $ok -ge 2 ] && break
  else ok=0; fi
  sleep 1
done
[ $ok -ge 2 ] || { say "Jellyfin did not come up: $(tail -5 "$W/jf/out")"; exit 3; }
SERVER="$(printf '%s' "$info" | field '["Id"]')"
say "jellyfin up: $(printf '%s' "$info" | field '["Version"]'), $SERVER"
TOKEN=""
api POST /Startup/Configuration '{"UICulture":"en-US","MetadataCountryCode":"US","PreferredMetadataLanguage":"en"}' >/dev/null
api GET /Startup/User >/dev/null
api POST /Startup/User "{\"Name\":\"kim\",\"Password\":\"$(cat "$W/secrets/kim_pw")\"}" >/dev/null
api POST "/Library/VirtualFolders?name=Movies&collectionType=movies&refreshLibrary=true" \
  "{\"LibraryOptions\":{\"PathInfos\":[{\"Path\":\"$W/lib/Movies\"}],\"EnableRealtimeMonitor\":false}}" >/dev/null
api POST /Startup/RemoteAccess '{"EnableRemoteAccess":false,"EnableAutomaticPortMapping":false}' >/dev/null
api POST /Startup/Complete >/dev/null
sleep 3
TOKEN="$(api POST /Users/AuthenticateByName "{\"Username\":\"kim\",\"Pw\":\"$(cat "$W/secrets/kim_pw")\"}" | field '["AccessToken"]')"
secret admin_token "$TOKEN"
# The lender's reverse-proxy trust, read back from Jellyfin's own config after
# its setup rewrote it: the one setting this take adds to the first.
cp "$W/jf/config/network.xml" "$OUT/network.xml"
say "jellyfin's KnownProxies: $(tr -d '\n' <"$OUT/network.xml" | sed -n 's/.*<KnownProxies>\(.*\)<\/KnownProxies>.*/\1/p')"
api POST /Library/Refresh >/dev/null
n=0
for _ in $(seq 1 60); do
  n="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie" | field '["TotalRecordCount"]')"; [ "$n" -ge 1 ] && break; sleep 1
done
FILM="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie" | field '["Items"][0]["Id"]')"
[ -n "$FILM" ] || { say "the film did not scan"; exit 3; }
say "film: $FILM"
api POST "/Auth/Keys?app=svrngs-oidc-run" >/dev/null
APIKEY="$(api GET /Auth/Keys | python3 -c 'import json,sys;print(next(k["AccessToken"] for k in json.load(sys.stdin)["Items"] if k["AppName"]=="svrngs-oidc-run"))')"
say "the plugin's key: made"

# --- the page and the group --------------------------------------------------
"$REPO/scripts/build-web.sh" --out "$W/site" >"$W/build.log"
starts "$W/site.pid" "$W/site.out" node "$REPO/web/tests/serve.mjs" "$W/site"
PAGE="http://localhost:$(port_of "$W/site.out")/"
say "the page: $PAGE"
starts "$W/kim.pid" "$W/kim/new.out" env SVRNGS_DIR="$W/kim" RUST_LOG=info,door=debug,oidc=debug NO_COLOR=1 "$B" new House --as Kim --origin "$PAGE"
for _ in $(seq 1 90); do grep -q "$PAGE#" "$W/kim/new.out" 2>/dev/null && break; sleep 1; done
LINK="$(grep -o "$PAGE#[A-Za-z0-9_-]*" "$W/kim/new.out" | head -1)"
CLIENT_LINK="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$W/kim/new.out" | head -1)"
say "kim: $(grep -i 'phone' "$W/kim/new.out" || grep -v '^$' "$W/kim/new.out" | head -1)"
for who in Alex Bo Cy; do
  lwho="$(printf '%s' "$who" | tr 'A-Z' 'a-z')"
  starts "$W/$lwho.pid" "$W/$lwho/join.out" env HOME="$W/$lwho" SVRNGS_DIR="$W/$lwho" \
    RUST_LOG=info NO_COLOR=1 "$B" join "$CLIENT_LINK" --as "$who"
  for _ in $(seq 1 120); do grep -q "You're in" "$W/$lwho/join.out" 2>/dev/null && break; sleep 1; done
  grep -q "You're in" "$W/$lwho/join.out" || { say "$who did not join: $(cat "$W/$lwho/join.out")"; exit 3; }
  say "$who: $(head -1 "$W/$lwho/join.out")"
done

# --- Kim lends it, and the plugin is told -----------------------------------
SVRNGS_DIR="$W/kim" NO_COLOR=1 RUST_LOG=debug "$B" share 18096 --name jellyfin --kind jellyfin \
  --sign-in-with-svrngs --limit 8 </dev/null >"$W/kim/share.out" 2>"$W/kim/share.log"
say "kim: $(grep -m1 'You lend' "$W/kim/share.out")"
ISSUER="$(grep -o 'issuer http://127.0.0.1:[0-9]*/oidc/[A-Za-z0-9-]*' "$W/kim/share.out" | head -1 | cut -d' ' -f2)"
CID="$(grep -o 'client id [A-Za-z0-9._~-]*' "$W/kim/share.out" | head -1 | cut -d' ' -f3)"
CSECRET="$(grep -o 'client secret [A-Za-z0-9._~-]*' "$W/kim/share.out" | head -1 | cut -d' ' -f3)"; CSECRET="${CSECRET%.}"
[ -n "$ISSUER" ] && [ -n "$CID" ] && [ -n "$CSECRET" ] || { say "share printed no issuer/client: $(cat "$W/kim/share.out")"; exit 3; }
say "the app's settings: issuer $ISSUER, client id $CID, a secret of ${#CSECRET} characters"

# Each computer's bare command, after the lend reaches it, prints the
# lend's own address on that computer; a phone reaches it through the page.
for who in Alex Cy; do
  lwho="$(printf '%s' "$who" | tr 'A-Z' 'a-z')"
  kill "$(cat "$W/$lwho.pid")" 2>/dev/null || true
  while kill -0 "$(cat "$W/$lwho.pid")" 2>/dev/null; do sleep 0.2; done
  starts "$W/$lwho.pid" "$W/$lwho/run.out" env SVRNGS_DIR="$W/$lwho" RUST_LOG=info NO_COLOR=1 "$B"
  for _ in $(seq 1 120); do grep -q 'jellyfin.*in your browser' "$W/$lwho/run.out" 2>/dev/null && break; sleep 1; done
  at="$(grep 'jellyfin.*in your browser' "$W/$lwho/run.out" | grep -o 'http://[a-z0-9-]*\.localhost:[0-9]*/' | head -1)"
  [ -n "$at" ] || { say "$who's bare command lists no jellyfin"; cat "$W/$lwho/run.out" >&2; exit 3; }
  say "$who's computer opens it at $at"
  printf '%s_AT=%s\n' "$(printf '%s' "$who" | tr 'a-z' 'A-Z')" "$at" >>"$W/env"
done
# Bo's phone is another device of Bo's: a link made on Bo's computer joins
# it as Bo, asked no name (`svrngs link --device`).
kill "$(cat "$W/bo.pid")" 2>/dev/null || true
while kill -0 "$(cat "$W/bo.pid")" 2>/dev/null; do sleep 0.2; done
starts "$W/bo.pid" "$W/bo/link.out" env SVRNGS_DIR="$W/bo" RUST_LOG=info NO_COLOR=1 "$B" link House --device --origin "$PAGE"
for _ in $(seq 1 90); do grep -q "$PAGE#[A-Za-z0-9_-]*" "$W/bo/link.out" 2>/dev/null && break; sleep 1; done
BO_LINK="$(grep -o "$PAGE#[A-Za-z0-9_-]*" "$W/bo/link.out" | head -1)"
[ -n "$BO_LINK" ] || { say "bo's device link was not made: $(cat "$W/bo/link.out")"; exit 3; }
ALEX_AT="$(sed -n 's/^ALEX_AT=//p' "$W/env" | tail -1)"
LABEL="$(printf '%s' "$ALEX_AT" | sed -E 's#^http://([^.]+)\..*#\1#')"
BO_PORT="$(cat "$W/bo/api")"; BO_PORT="${BO_PORT##*:}"
BO_AT="http://$LABEL.localhost:$BO_PORT/"
say "bo's phone opens $BO_LINK"
say "bo's computer's door is at $BO_AT"
printf 'BO_AT=%s\n' "$BO_AT" >>"$W/env"
PLUGIN="$(python3 - "$ISSUER" "$CID" "$CSECRET" <<'PY'
import json,sys
issuer,cid,secret=sys.argv[1:]
print(json.dumps({
 "oidEndpoint":issuer,"oidClientId":cid,"oidSecret":secret,"enabled":True,
 "enableAuthorization":True,"enableAllFolders":True,"enabledFolders":[],"adminRoles":[],"roles":[],
 "enableFolderRoles":False,"enableLiveTvRoles":False,"enableLiveTv":False,"enableLiveTvManagement":False,
 "liveTvRoles":[],"liveTvManagementRoles":[],"folderRoleMapping":[],"roleClaim":"",
 "oidScopes":[],"defaultProvider":"","schemeOverride":"","portOverride":None,"newPath":True,
 "defaultUsernameClaim":"preferred_username","avatarUrlFormat":None,"disableHttps":True,
 "disablePushedAuthorization":True,"doNotValidateEndpoints":True,"doNotValidateIssuerName":False,
 "doNotLoadProfile":False}))
PY
)"
printf '%s' "$PLUGIN" >"$OUT/plugin-settings.json"
curl -sS -f -X POST "$O/sso/OID/Add/svrngs?api_key=$APIKEY" -H "Content-Type: application/json" -d "$PLUGIN" >/dev/null
say "the plugin is told: $(curl -sS -f "$O/sso/OID/Get?api_key=$APIKEY" | head -c 300)"
BRAND="$(api GET /System/Configuration/branding | python3 -c '
import json,sys
d=json.load(sys.stdin)
d["LoginDisclaimer"]="<form action=\"/sso/OID/start/svrngs\"><button class=\"raised block emby-button button-submit\">Sign in with SSO</button></form>"
d["CustomCss"]=".disclaimerContainer{display:block}"
print(json.dumps(d))')"
api POST /System/Configuration/branding "$BRAND" >/dev/null
say "jellyfin's login page offers: Sign in with SSO"

cat >"$W/env.json" <<JSON
{"page":"$PAGE","link":"$LINK","issuer":"$ISSUER","client_id":"$CID","client_secret":"$CSECRET",
 "jf":"$O","jf_port":$JF_PORT,"server":"$SERVER","film":"$FILM","admin":"$TOKEN",
 "alex_at":"$(sed -n 's/^ALEX_AT=//p' "$W/env" | tail -1)","bo_at":"$(sed -n 's/^BO_AT=//p' "$W/env" | tail -1)","bo_link":"$BO_LINK",
 "cy_at":"$(sed -n 's/^CY_AT=//p' "$W/env" | tail -1)",
 "kim_pw":"$(cat "$W/secrets/kim_pw")","admin_key":"$APIKEY",
 "bin":"$B","kim_dir":"$W/kim","kim_log":"$W/kim/new.out"}
JSON

say "load at the drive's start: $(uptime | sed 's/.*load/load/')"
node "$HERE/people.mjs" "$W/env.json" "$OUT/raw.json" >"$OUT/driver.out" 2>"$W/driver.err" || true
say "load at the drive's end: $(uptime | sed 's/.*load/load/')"
mkdir -p "$OUT/jellyfin-log"; cp "$W/jf/log/"*.log "$OUT/jellyfin-log/" 2>/dev/null || true
cp "$W/kim/share.out" "$OUT/share.out"
python3 "$HERE/judge.py" "$OUT" | tee -a "$OUT/steps.log" || true
python3 scripts/lean_check.py >"$OUT/lean.txt" 2>&1 || true
if git -C "$REPO" diff --quiet -- scripts/trust_floor.txt; then FLOOR=unchanged; else FLOOR=changed; fi
python3 "$HERE/setup.py" "$OUT" "$FLOOR"
say "setup.json: pinned versions, the trust floor $FLOOR, lean in lean.txt"
