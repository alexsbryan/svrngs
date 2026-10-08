#!/usr/bin/env bash
# Movie Night's run (ralph/next/compose/STATE.md mn-1; the rule, bars and
# predictions in demo/runs/mn/2026-10-08/README.md, mn-0): three libraries
# in one group on this host, through iroh's public relays, as
# apps/jellyfin/check/check.sh `up` and `mw` set one (forked here, as a
# check of its own, so that one's runs stay as they ran):
#
# - Sam's and Dee's: a scratch Jellyfin 12.0.0 each, data dirs of its own,
#   each lent `share <its port> --kind jellyfin --sign-in <its middleware's
#   port>` by its lender, the middleware (apps/jellyfin/mw) beside it. Blade
#   Runner (1982) in both, a short test film with a .nfo naming its TMDb and
#   IMDb ids; Alien (1979) Sam's alone, Arrival (2016) Dee's alone; Brazil
#   (1985) in a second library of Sam's, which the check keeps from one
#   person by their account's policy once the middleware has made it.
# - Cy's: a lend of kind jellyfin whose lender's computer is asleep — its
#   svrngs stopped — before the group's page lists it, or just after.
# - Movie Night (apps/movie-night), a copy of it served by a plain file
#   server, lent by Sam `share <its port> --kind movie-night --reads jellyfin`.
#
#   check.sh up       <scratch> <out>          all of the above; Cy asleep; Alex's bare command
#                                              prints each lend's origin on his computer
#   check.sh phone    <scratch> <out> <engine> two people new to the group, the journey on a phone (mn.mjs
#                                              `phone`): Ana and Ben in chromium, Cleo and Dara in webkit,
#                                              the second kept from Sam's Brazil; writes <out>/phone-<engine>.json
#   check.sh asleep   <scratch> <out> <engine> Cy woken, a person new lists the group, Cy put to sleep, then
#                                              Movie Night searched (mn.mjs `asleep`); <out>/asleep-<engine>.json
#   check.sh computer <scratch> <out>          Alex on his computer, Chromium (mn.mjs `computer`)
#   check.sh limit    <scratch> <out>          two phones in Chromium, one searching once and the other every
#                                              ten seconds after (mn.mjs `limit`): what each door counted
#   check.sh plant    <scratch> <out> <plant>  Movie Night served with plant.js naming <plant> — per-copy,
#                                              asleep-empty, lender-token (Sam's own token) — one journey on
#                                              a phone in Chromium, the plant taken back after
#   check.sh judge    <scratch> <out>          each person's Jellyfin tokens, scan.py over every byte each
#                                              browser received, the doors' logs, then judge.py
#   check.sh down     <scratch> <out>          stops everything; the scratch dir stays for the secrets
#
# Each step stays inside ten minutes on this host. svrngs built first:
# cargo build -p svrngs. Needs Jellyfin.app and Playwright's browsers
# (web/tests). scripts/check.sh runs none of this: it needs Jellyfin.
set -euo pipefail
STEP="$1"; R="$2"; OUT="$3"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../.." && pwd)"
JF_CHECK="$REPO/apps/jellyfin/check"
B="$REPO/target/debug/svrngs"
JF_APP="${JF_APP:-/Applications/Jellyfin.app/Contents}"
# Ports of their own, apart from apps/jellyfin/check's 8096 and 8097.
SAM_JF=8296; DEE_JF=8396; CY_PORT=8496; MN_PORT=8596
SECRETS="$R/secrets"; mkdir -p "$OUT"
LOG='info,door=debug,origin=debug'
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
field() { python3 -c "import json,sys;print(json.load(sys.stdin)$1)"; }
port_of() { for _ in $(seq 1 100); do grep -m1 -o 'port [0-9]*' "$1" 2>/dev/null | cut -d' ' -f2 && return; sleep 0.1; done; return 1; }
secret() { (umask 077; printf '%s' "$2" >"$SECRETS/$1"); }
pw() { head -c 18 /dev/urandom | base64 | tr -d '/+='; }
# Jellyfin's API at <port>, as its lender: api <port> <method> <path> [body]
api() {
  local port="$1" m="$2" p="$3" body="${4:-}" who
  who="$( [ "$port" = $SAM_JF ] && echo sam || echo dee)"
  local token=""; [ -s "$SECRETS/${who}_admin_token" ] && token="$(cat "$SECRETS/${who}_admin_token")"
  curl -sS -f -X "$m" "http://127.0.0.1:$port$p" -H "Content-Type: application/json" \
    -H "Authorization: MediaBrowser Client=\"mn check\", Device=\"$who computer\", DeviceId=\"mn-$who-setup\", Version=\"1\"${token:+, Token=\"$token\"}" \
    ${body:+-d "$body"}
}

down() {
  for p in "$R"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done
  rm -f "$R"/*.pid
}

# A test film, 60 s at 640x360, and its .nfo naming the film's ids, made
# once in <scratch>/films and copied into each library that has it.
film() {
  local title="$1" year="$2" tmdb="$3" imdb="$4" hue="$5" ff="$JF_APP/MacOS/ffmpeg" dir="$R/films/$1 ($2)"
  mkdir -p "$dir"
  [ -s "$dir/$title ($year).mp4" ] || "$ff" -v error -y -f lavfi -i "testsrc2=size=640x360:rate=24" \
    -f lavfi -i "sine=frequency=440:sample_rate=48000" -t 60 -vf "hue=h=$hue" -c:v libx264 -preset veryfast \
    -b:v 1M -g 48 -pix_fmt yuv420p -c:a aac -b:a 96k -movflags +faststart "$dir/$title ($year).mp4"
  cat >"$dir/$title ($year).nfo" <<EOF
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<movie>
  <title>$title</title>
  <year>$year</year>
  <uniqueid type="tmdb" default="true">$tmdb</uniqueid>
  <uniqueid type="imdb">$imdb</uniqueid>
</movie>
EOF
}
shelve() { mkdir -p "$2/$1"; cp "$R/films/$1"/* "$2/$1/"; }

# <who>'s Jellyfin at <port>, <n> films in its libraries, the dirs named,
# each `<name>=<dir>`; then the middleware, at <port>+1, with a key minted once.
jellyfin() {
  local who="$1" port="$2" want="$3"; shift 3
  local d="$R/jf-$who"
  mkdir -p "$d"/{data,config,cache,log}
  cat >"$d/config/network.xml" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<NetworkConfiguration xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <InternalHttpPort>$port</InternalHttpPort>
  <PublicHttpPort>$port</PublicHttpPort>
  <AutoDiscovery>false</AutoDiscovery>
  <EnableIPv6>false</EnableIPv6>
  <LocalNetworkAddresses><string>127.0.0.1</string></LocalNetworkAddresses>
  <EnableRemoteAccess>false</EnableRemoteAccess>
</NetworkConfiguration>
EOF
  nohup "$JF_APP/MacOS/jellyfin" -d "$d/data" -c "$d/config" -C "$d/cache" -l "$d/log" \
    -w "$JF_APP/Resources/jellyfin-web" --ffmpeg "$JF_APP/MacOS/ffmpeg" --nonetchange >"$d/stdout.log" 2>&1 &
  echo $! >"$R/jf-$who.pid"
  local ok=0 info=""
  for _ in $(seq 1 120); do
    # Kestrel answers before the app is ready: the document twice in a row.
    if info="$(curl -sS -f --max-time 2 "http://127.0.0.1:$port/System/Info/Public" 2>/dev/null)" && [[ "$info" == *StartupWizardCompleted* ]]; then
      ok=$((ok + 1)); [ $ok -ge 2 ] && break
    else ok=0; fi
    sleep 1
  done
  [ $ok -ge 2 ] || { say "$who's Jellyfin did not come up"; exit 3; }
  say "$who's jellyfin up at $port: $info"
  secret "${who}_pw" "$(pw)"
  api "$port" POST /Startup/Configuration '{"UICulture":"en-US","MetadataCountryCode":"US","PreferredMetadataLanguage":"en"}' >/dev/null
  api "$port" GET /Startup/User >/dev/null
  api "$port" POST /Startup/User "{\"Name\":\"$who\",\"Password\":\"$(cat "$SECRETS/${who}_pw")\"}" >/dev/null
  local lib
  for lib in "$@"; do
    # No fetcher from the internet: each film's ids are its .nfo's.
    api "$port" POST "/Library/VirtualFolders?name=${lib%%=*}&collectionType=movies&refreshLibrary=true" \
      "{\"LibraryOptions\":{\"PathInfos\":[{\"Path\":\"${lib#*=}\"}],\"EnableRealtimeMonitor\":false,\"TypeOptions\":[{\"Type\":\"Movie\",\"MetadataFetchers\":[],\"ImageFetchers\":[]}]}}" >/dev/null
  done
  api "$port" POST /Startup/RemoteAccess '{"EnableRemoteAccess":false,"EnableAutomaticPortMapping":false}' >/dev/null
  api "$port" POST /Startup/Complete >/dev/null
  sleep 3
  secret "${who}_admin_token" "$(api "$port" POST /Users/AuthenticateByName "{\"Username\":\"$who\",\"Pw\":\"$(cat "$SECRETS/${who}_pw")\"}" | field '["AccessToken"]')"
  api "$port" POST /Library/Refresh >/dev/null
  local n=0
  for _ in $(seq 1 90); do
    n="$(api "$port" GET "/Items?Recursive=true&IncludeItemTypes=Movie" | field '["TotalRecordCount"]')"; [ "$n" -ge "$want" ] && break; sleep 1
  done
  api "$port" GET "/Items?Recursive=true&IncludeItemTypes=Movie&Fields=ProviderIds,ProductionYear" |
    python3 -c 'import json,sys;print(json.dumps([(i["Name"],i.get("ProductionYear"),i.get("ProviderIds")) for i in json.load(sys.stdin)["Items"]]))' >"$OUT/films-$who.json"
  say "$who's films: $(cat "$OUT/films-$who.json")"
  api "$port" POST "/Auth/Keys?app=svrngs-mw" >/dev/null
  secret "${who}_mw_key" "$(api "$port" GET /Auth/Keys | python3 -c 'import json,sys;print(next(k["AccessToken"] for k in json.load(sys.stdin)["Items"] if k["AppName"]=="svrngs-mw"))')"
  NO_COLOR=1 RUST_LOG=info,jellyfin_mw=debug nohup "$REPO/apps/jellyfin/mw/target/debug/jellyfin-mw" \
    --listen "127.0.0.1:$((port + 1))" --jellyfin "127.0.0.1:$port" <"$SECRETS/${who}_mw_key" >"$R/mw-$who.out" 2>"$R/mw-$who.log" &
  echo $! >"$R/mw-$who.pid"
  for _ in $(seq 1 50); do grep -q 'listening on port' "$R/mw-$who.out" 2>/dev/null && break; sleep 0.1; done
  say "$who's middleware: $(head -1 "$R/mw-$who.out")"
}

# <who>'s computer, the bare command, running until stopped; waits for
# <line> in what it prints.
computer() {
  local who="$1" line="$2"
  SVRNGS_DIR="$R/$who" RUST_LOG="$LOG" NO_COLOR=1 nohup "$B" >"$R/$who/run.out" 2>>"$R/$who/run.log" &
  echo $! >"$R/$who.pid"
  for _ in $(seq 1 90); do grep -q "$line" "$R/$who/run.out" 2>/dev/null && return; sleep 1; done
  say "$who's computer did not say \"$line\""; cat "$R/$who/run.out" >&2; exit 3
}

# Movie Night as the check serves it: apps/movie-night, its check apart,
# with plant.js naming PLANT (else none).
site() {
  rm -rf "$R/mn-site"; mkdir -p "$R/mn-site"
  rsync -a --exclude check/ "$REPO/apps/movie-night/" "$R/mn-site/"
  if [ -n "${PLANT:-}" ]; then
    local token=null
    [ "$PLANT" = lender-token ] && token="'$(cat "$SECRETS/sam_admin_token")'"
    printf "export const PLANT = '%s';\nexport const TOKEN = %s;\n" "$PLANT" "$token" >"$R/mn-site/plant.js"
  fi
}

up() {
  down; rm -rf "$R"; mkdir -p "$R"/{sam,dee,cy,alex} "$OUT"
  (umask 077; mkdir -p "$SECRETS")
  : >"$OUT/steps.log"
  film "Blade Runner" 1982 78 tt0083658 0 & film "Alien" 1979 348 tt0078748 90 &
  film "Arrival" 2016 329865 tt2543164 180 & film "Brazil" 1985 68 tt0088846 270 & wait
  shelve "Blade Runner (1982)" "$R/lib/sam/movies"; shelve "Alien (1979)" "$R/lib/sam/movies"
  shelve "Brazil (1985)" "$R/lib/sam/kept"
  shelve "Blade Runner (1982)" "$R/lib/dee/movies"; shelve "Arrival (2016)" "$R/lib/dee/movies"
  jellyfin sam $SAM_JF 3 "Movies=$R/lib/sam/movies" "Kept=$R/lib/sam/kept"
  jellyfin dee $DEE_JF 2 "Movies=$R/lib/dee/movies"

  # The page, built and served on localhost, where every name answers.
  "$REPO/scripts/build-web.sh" --out "$R/site" >"$R/build.log"
  node "$REPO/demo/runs/u4/2026-10-05/drive.mjs" serve "$R/site" >"$R/site.out" 2>&1 & echo $! >"$R/site.pid"
  local page; page="http://localhost:$(port_of "$R/site.out")/"
  say "the page: $page"

  # Movie Night's files, a plain file server.
  site
  nohup python3 -m http.server "$MN_PORT" --bind 127.0.0.1 --directory "$R/mn-site" >"$R/mn-server.log" 2>&1 &
  echo $! >"$R/mn-server.pid"

  # Sam makes House; Dee, Cy and Alex join by the client's link.
  SVRNGS_DIR="$R/sam" RUST_LOG="$LOG" NO_COLOR=1 nohup "$B" new House --as Sam --origin "$page" >"$R/sam/new.out" 2>"$R/sam/run.log" &
  echo $! >"$R/sam.pid"
  for _ in $(seq 1 60); do grep -q "$page#" "$R/sam/new.out" 2>/dev/null && break; sleep 1; done
  local link client who name
  link="$(grep -o "$page#[A-Za-z0-9_-]*" "$R/sam/new.out")"
  client="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$R/sam/new.out")"
  for who in dee cy alex; do
    name="$(echo "${who:0:1}" | tr a-z A-Z)${who:1}"
    SVRNGS_DIR="$R/$who" RUST_LOG="$LOG" NO_COLOR=1 nohup "$B" join "$client" --as "$name" >"$R/$who/join.out" 2>"$R/$who/run.log" &
    echo $! >"$R/$who.pid"
    for _ in $(seq 1 60); do grep -q "You're in" "$R/$who/join.out" 2>/dev/null && break; sleep 1; done
    say "$who: $(head -1 "$R/$who/join.out")"
  done

  # Each lends: Sam's and Dee's Jellyfins through their middlewares, Cy's
  # a jellyfin its computer is asleep before anyone asks, and Sam Movie
  # Night, which reads every jellyfin.
  lend() { local who="$1"; shift; SVRNGS_DIR="$R/$who" NO_COLOR=1 RUST_LOG=debug "$B" share "$@" --limit 8 </dev/null >>"$R/$who/share.out" 2>>"$R/$who/share.log"; say "$who: $(tail -n +1 "$R/$who/share.out" | grep Lent | tail -1)"; }
  lend sam $SAM_JF --kind jellyfin --sign-in $((SAM_JF + 1))
  lend dee $DEE_JF --kind jellyfin --sign-in $((DEE_JF + 1))
  # share lends only a port something answers: a stand-in answers Cy's
  # while it is lent, and nothing after.
  python3 -m http.server "$CY_PORT" --bind 127.0.0.1 >/dev/null 2>&1 & local stand=$!
  sleep 1; lend cy $CY_PORT --kind jellyfin; kill $stand
  lend sam $MN_PORT --kind movie-night --reads jellyfin
  say "sam is told: $(grep -v '^Lent' "$R/sam/share.out" | tr '\n' ' ')"

  # Dee's computer admits only those it knows stand in House, and learns
  # of those who joined after it by its ring sync round, each 60 s
  # (crates/svrngs-node/src/ring_sync.rs:103): Alex's computer waits for it.
  for _ in $(seq 1 150); do
    grep 'round membership exchanged_with=' "$R/dee/run.log" | grep 'Alex/' | grep -q 'Cy/' && break; sleep 1
  done
  say "dee: $(grep -o 'round membership exchanged_with=.*' "$R/dee/run.log" | tail -1)"
  # Cy's computer goes to sleep; Alex's bare command lists the rest.
  kill "$(cat "$R/cy.pid")"; rm -f "$R/cy.pid"
  kill "$(cat "$R/alex.pid")"; sleep 2
  computer alex "Dee's jellyfin, in your browser"
  sleep 3
  local at; at() { grep "$1, in your browser" "$R/alex/run.out" | grep -o 'http://[a-z0-9-]*\.localhost:[0-9]*/'; }
  cat >"$R/env" <<EOF
PAGE=$page
LINK=$link
MN_AT=$(at "Sam's movie-night")
SAM_AT=$(at "Sam's jellyfin")
DEE_AT=$(at "Dee's jellyfin")
MOVIES=$(api $SAM_JF GET /Library/VirtualFolders | python3 -c 'import json,sys;print(next(f["ItemId"] for f in json.load(sys.stdin) if f["Name"]=="Movies"))')
EOF
  cp "$R/alex/run.out" "$OUT/alex-bare-command.out"
  sed -E 's#(svrngs://join/|'"$page"'\#)[A-Za-z0-9_-]+#\1<the link>#g' "$R/sam/new.out" >"$OUT/sam-new.out"
  for who in sam dee cy; do cp "$R/$who/share.out" "$OUT/$who-share.out"; done
  say "alex's computer: $(grep -c 'in your browser' "$R/alex/run.out") lends listed; Cy asleep"
}

# The label of the lend whose origin on Alex's computer is <at>.
label() { echo "$1" | cut -d/ -f3 | cut -d. -f1; }

# Cy's computer awake: its bare command, until it has listed the group's lends.
wake() { computer cy 'Leave this running'; say "cy: awake"; }

drive() {
  local tag="$1" json="$2"
  # shellcheck disable=SC1091
  . "$R/env"
  json="${json%\}},\"labels\":{\"Sam\":\"$(label "$SAM_AT")\",\"Dee\":\"$(label "$DEE_AT")\"}}"
  rm -rf "$R/dump/$tag"
  DUMP="$R/dump/$tag" JF_SAM="http://127.0.0.1:$SAM_JF" SAM_ADMIN="$(cat "$SECRETS/sam_admin_token")" MOVIES="$MOVIES" \
    SLEEP="kill \$(cat '$R/cy.pid') && rm -f '$R/cy.pid'" \
    node "$HERE/mn.mjs" "$json" >"$OUT/$tag.json" 2>"$R/$tag.err" || say "$tag: the driver exited $?: $(tail -3 "$R/$tag.err")"
  say "$tag: $(python3 "$HERE/judge.py" --line "$OUT/$tag.json")"
}

phone() {
  . "$R/env"
  local engine="$1" a=Ana b=Ben
  [ "$engine" = webkit ] && { a=Cleo; b=Dara; }
  drive "phone-$engine" "{\"journey\":\"phone\",\"engine\":\"$engine\",\"link\":\"$LINK\",
    \"people\":[{\"name\":\"$a\",\"choose\":\"Sam\"},{\"name\":\"$b\",\"choose\":\"Dee\",\"kept_from\":true}]}"
}

asleep() {
  . "$R/env"
  local engine="$1" p=Eli; [ "$engine" = webkit ] && p=Fay
  [ -n "${PLANT:-}" ] && p=Jo
  wake
  drive "asleep-$engine${PLANT:+-plant-$PLANT}" "{\"journey\":\"asleep\",\"engine\":\"$engine\",\"link\":\"$LINK\",\"people\":[{\"name\":\"$p\"}]}"
  [ -f "$R/cy.pid" ] && { kill "$(cat "$R/cy.pid")"; rm -f "$R/cy.pid"; say "cy: put to sleep again, the driver having not"; }
  return 0
}

case "$STEP" in
  up) up ;;
  phone) phone "$4" ;;
  asleep) asleep "$4" ;;
  computer) . "$R/env"
    drive computer "{\"journey\":\"computer\",\"engine\":\"chromium\",\"at\":\"$MN_AT\",
      \"lends\":\"$(label "$SAM_AT"),$(label "$DEE_AT")\",
      \"people\":[{\"name\":\"Alex\",\"choose\":\"Sam\"}]}" ;;
  limit) . "$R/env"
    drive limit "{\"journey\":\"limit\",\"engine\":\"chromium\",\"link\":\"$LINK\",\"people\":[{\"name\":\"Gus\"},{\"name\":\"Hal\"}],\"after_s\":[5,15,25,35,45]}" ;;
  plant)
    PLANT="$4"; export PLANT; site
    say "plant $PLANT: Movie Night served with it"
    case "$PLANT" in
      asleep-empty) asleep chromium ;;
      per-copy|lender-token) . "$R/env"
        drive "plant-$PLANT" "{\"journey\":\"once\",\"engine\":\"chromium\",\"link\":\"$LINK\",\"people\":[{\"name\":\"$( [ "$PLANT" = per-copy ] && echo Ivy || echo Kai)\"}]}" ;;
      *) echo "plant: per-copy|asleep-empty|lender-token" >&2; exit 2 ;;
    esac
    PLANT=""; site; say "plant taken back: Movie Night served as it is kept" ;;
  judge)
    python3 "$HERE/tokens.py" "$R" | tee -a "$OUT/steps.log"
    python3 "$JF_CHECK/scan.py" "$R" "$OUT" | tee -a "$OUT/steps.log"
    python3 "$HERE/doors.py" "$R" "$OUT" | tee -a "$OUT/steps.log"
    python3 "$HERE/judge.py" "$OUT" | tee -a "$OUT/steps.log" ;;
  down) down ;;
  *) echo "usage: check.sh up|phone|asleep|computer|limit|plant|judge|down <scratch> <out> [engine | plant]" >&2; exit 2 ;;
esac
