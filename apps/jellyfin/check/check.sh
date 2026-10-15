#!/usr/bin/env bash
# u5-4's check (ralph/next/u5/STATE.md, its row; DESIGN §3, "Lent apps",
# call 4, sign in once): stock Jellyfin as `svrngs share 8096` lends it,
# against a scratch Jellyfin on this host, in Chromium and WebKit. Forked
# from u5-3's measure.sh (demo/runs/u5/2026-10-05-jellyfin/, at 4b23bb4).
#
# A scratch Jellyfin on 127.0.0.1:8096 — U3's dry run's
# (demo/runs/a/dry-2026-10-04/dry.sh), the macOS app's 12.0.0 server with
# its own data dirs — with three films and the lender's own account; Sam's
# computer and Alex's as two data dirs running the `svrngs` command through
# iroh's public relays; the page built and served on localhost, where
# every name answers, so a phone's apps each open at an address of their
# own (u5-2). Sam lends 8096 as it comes, and makes Ana and Ben accounts
# in Jellyfin's own pages by the line `share` printed (lender.mjs).
#
#   check.sh films <dir> <seconds>                 Sam's three films, that long, made in <dir> unless there
#   check.sh up    <scratch> <out>                 all of the above; with MEDIA=<dir>, Sam's library is <dir>'s
#                                                  films; PEOPLE names who the lender makes accounts for (else
#                                                  ana ben; empty, nobody); SHARE_FLAGS is what share is given (else --limit 8);
#                                                  with KEEP_BACK=<name>, one more library, of home videos, so named
#                                                  (one word: the env file is read by words)
#   check.sh pair  <scratch> <out> <way> <engine>  two people new to it through the lend, way computer|phone,
#                                                  engine chromium|webkit: from the list to a playing film,
#                                                  seeks, the other watched; writes <out>/pair-<way>-<engine>.json.
#                                                  With MW=1 (mw-2, mwp-3), after `mw`: through the lend whose
#                                                  sign-in is the middleware's, which is started afresh
#                                                  (JELLYFIN_MW_PLANT passed on), nothing typed —
#                                                  on a computer Alex and Bo, each from their own; each person's
#                                                  Jellyfin tokens read from its database after (tokens.py);
#                                                  writes <out>/pair-mw-<way>-<engine>[-plant-<plant>].json
#   check.sh plant <scratch> <out> <plant> <engine> the lend made to keep a token, then a computer pair, then
#                                                  the token taken back: plant `token`, a token of its own (the
#                                                  lend's token reaching a browser); `one-device`, Ben's, on the
#                                                  lend's one device (one person's sign-in ending another's);
#                                                  writes <out>/pair-computer-<engine>-plant-<plant>.json
#   check.sh trio  <scratch> <out>                 Demo A's three (trio.mjs), after up with PEOPLE="alex bo cy":
#                                                  Alex on the computer plays One, Bo on a WebKit phone Two, Cy on
#                                                  a Chromium phone Three, WINDOW s at once (else 600), seeking
#                                                  10, 22 and 22 times; writes <out>/trio[-$RUN].json, its
#                                                  summary (trio.py) and scan.json; takes over ten minutes.
#                                                  With MW=1, after `mw` with COMPUTERS="alex bo cy": Demo A's
#                                                  steps with nothing typed (demo/a.md), the middleware started
#                                                  afresh, each of the three on their own computer, Bo in
#                                                  WebKit; then the libraries each account sees, in
#                                                  <out>/views-<tag>.json
#   check.sh mw    <scratch> <out> <tag>           mw-1's (ralph/next/mw/STATE.md), after up: the middleware
#                                                  (apps/jellyfin/mw) on MW_PORT (else 8097), given a key from
#                                                  Jellyfin's API Keys once; Sam lends 8096 again as jellyfin-mw
#                                                  with it as its sign-in (mwp-3), Bo joined as
#                                                  a second computer; each run starts it afresh, JELLYFIN_MW_PLANT
#                                                  passed on, the users it made before deleted; then mw.py, which
#                                                  writes <out>/mw-<tag>.json and exits 0 only if every bar passed.
#                                                  KEEP_BACK=<name> is given it as --keep-back; COMPUTERS names
#                                                  who joins on a computer (else alex bo); MW_SHARE_FLAGS is what
#                                                  share is given (else --limit 8)
#   check.sh mw2   <scratch> <out>                 mw-2's runs in their order (demo/runs/mw/2026-10-08/mw-2/run.sh),
#                                                  through the sign-in since mwp-3: up, mw, each plant on a computer
#                                                  in Chromium, judged; the four pairs, judged together; down
#   check.sh judge <scratch> <out> <run>...        scan.py, then judge.py over the runs named (pair-*.json stems):
#                                                  exit 0 only if every bar of every one passed
#   check.sh down  <scratch> <out>                 stops everything; the scratch dir stays for the secrets
#
# Each step but `trio` stays inside ten minutes on this host. svrngs built first:
# cargo build -p svrngs. Nothing of Jellyfin outlives `down` but what is
# under <out>; the secrets stay in <scratch>/secrets. scripts/check.sh runs
# none of this: it needs Jellyfin (DESIGN §8, U5).
set -euo pipefail
STEP="$1"; R="$2"; OUT="$3"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../.." && pwd)"
B="$REPO/target/debug/svrngs"
JF_APP="${JF_APP:-/Applications/Jellyfin.app/Contents}"
O=http://127.0.0.1:8096
SECRETS="$R/secrets"; [ "$STEP" = films ] || mkdir -p "$OUT"
# Who the lender makes accounts for, and what share is told: u5-4's pair
# unless a run says otherwise (Demo A's, `trio`).
PEOPLE="${PEOPLE-ana ben}"; SHARE_FLAGS="${SHARE_FLAGS---limit 8}"
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
field() { python3 -c "import json,sys;print(json.load(sys.stdin)$1)"; }
port_of() { for _ in $(seq 1 100); do grep -m1 -o 'port [0-9]*' "$1" 2>/dev/null | cut -d' ' -f2 && return; sleep 0.1; done; return 1; }
CLI='MediaBrowser Client="u5-4 check", Device="Sam computer", DeviceId="u5-4-setup", Version="1"'
api() { curl -sS -f -X "$1" "$O$2" -H "Content-Type: application/json" -H "Authorization: $CLI${TOKEN:+, Token=\"$TOKEN\"}" ${3:+-d "$3"}; }
users() { api GET /Users | python3 -c 'import json,sys;print(json.dumps(sorted(u["Name"] for u in json.load(sys.stdin))))'; }
secret() { (umask 077; printf '%s' "$2" >"$SECRETS/$1"); }
pw() { head -c 18 /dev/urandom | base64 | tr -d '/+='; }

down() {
  for p in "$R"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done
  rm -f "$R"/*.pid
}

# Sam's films, each LENGTH s at 720p, a keyframe each 2 s, made in <dir>
# unless there already: One, H.264 in MP4, which both engines play as it
# is; Two, MPEG-2 in Matroska, which neither plays, so Jellyfin converts
# it for both; Three, H.264 in Matroska, which Jellyfin repackages for
# the engines that cannot open Matroska. A 1,200 s one took 5.5 min at a
# load of 65 here (2026-10-05), so Demo A's are made once, by `films`.
films() {
  local dir="$1" len="$2" ff="$JF_APP/MacOS/ffmpeg"
  mkdir -p "$dir"
  # Each written aside and moved in whole, so a film cut short is made again.
  make() {
    local to="$dir/$1" part="$dir/.part-$1"; shift
    [ -s "$to" ] || { "$ff" -v error -y "$@" "$part" && mv "$part" "$to"; }
  }
  make "Test Pattern One (2026).mp4" -f lavfi -i "testsrc2=size=1280x720:rate=24" -f lavfi -i "sine=frequency=440:sample_rate=48000" \
    -t "$len" -c:v libx264 -preset veryfast -b:v 3M -g 48 -pix_fmt yuv420p -c:a aac -b:a 128k -movflags +faststart -f mp4 &
  make "Test Pattern Two (2026).mkv" -f lavfi -i "testsrc=size=1280x720:rate=24" -f lavfi -i "sine=frequency=660:sample_rate=48000" \
    -t "$len" -c:v mpeg2video -b:v 4M -g 48 -c:a mp2 -b:a 192k -f matroska &
  make "Test Pattern Three (2026).mkv" -f lavfi -i "testsrc2=size=1280x720:rate=24" -f lavfi -i "sine=frequency=550:sample_rate=48000" \
    -t "$len" -vf hue=h=120 -c:v libx264 -preset veryfast -b:v 3M -g 48 -pix_fmt yuv420p -c:a aac -b:a 128k -f matroska &
  wait
  for f in "One (2026).mp4" "Two (2026).mkv" "Three (2026).mkv"; do
    [ -s "$dir/Test Pattern $f" ] || { echo "the film Test Pattern $f was not made" >&2; exit 3; }
  done
}

up() {
  down; rm -rf "$R"; mkdir -p "$R"/jf/{data,config,cache,log,media} "$R"/sam "$R"/alex "$OUT"
  (umask 077; mkdir -p "$SECRETS")
  : >"$OUT/steps.log"

  # --- Sam's Jellyfin, its films made here, or MEDIA's (`films`) ----------
  local ff="$JF_APP/MacOS/ffmpeg" lib="${MEDIA:-$R/jf/media}"
  [ -n "${MEDIA:-}" ] || films "$lib" 120
  # Sam's home videos, a library Sam may keep back (the mw README §7, item 5).
  if [ -n "${KEEP_BACK:-}" ]; then
    mkdir -p "$R/jf/kept"
    "$ff" -v error -y -f lavfi -i "testsrc2=size=640x360:rate=24" -t 20 -c:v libx264 -preset veryfast \
      -pix_fmt yuv420p "$R/jf/kept/Picnic.mp4"
  fi
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

  secret sam_pw "$(pw)"; for who in $PEOPLE; do secret "${who}_pw" "$(pw)"; done
  TOKEN=""
  api POST /Startup/Configuration '{"UICulture":"en-US","MetadataCountryCode":"US","PreferredMetadataLanguage":"en"}' >/dev/null
  api GET /Startup/User >/dev/null
  api POST /Startup/User "{\"Name\":\"sam\",\"Password\":\"$(cat "$SECRETS/sam_pw")\"}" >/dev/null
  api POST "/Library/VirtualFolders?name=Movies&collectionType=movies&refreshLibrary=true" \
    "{\"LibraryOptions\":{\"PathInfos\":[{\"Path\":\"$lib\"}],\"EnableRealtimeMonitor\":false}}" >/dev/null
  [ -z "${KEEP_BACK:-}" ] || api POST "/Library/VirtualFolders?name=$KEEP_BACK&collectionType=homevideos&refreshLibrary=true" \
    "{\"LibraryOptions\":{\"PathInfos\":[{\"Path\":\"$R/jf/kept\"}],\"EnableRealtimeMonitor\":false}}" >/dev/null
  api POST /Startup/RemoteAccess '{"EnableRemoteAccess":false,"EnableAutomaticPortMapping":false}' >/dev/null
  api POST /Startup/Complete >/dev/null
  sleep 3
  TOKEN="$(api POST /Users/AuthenticateByName "{\"Username\":\"sam\",\"Pw\":\"$(cat "$SECRETS/sam_pw")\"}" | field '["AccessToken"]')"
  secret admin_token "$TOKEN"
  api POST /Library/Refresh >/dev/null
  local n=0
  for _ in $(seq 1 60); do
    n="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie" | field '["TotalRecordCount"]')"; [ "$n" -ge 3 ] && break; sleep 1
  done
  ONE="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie&SearchTerm=One" | field '["Items"][0]["Id"]')"
  TWO="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie&SearchTerm=Two" | field '["Items"][0]["Id"]')"
  THREE="$(api GET "/Items?Recursive=true&IncludeItemTypes=Movie&SearchTerm=Three" | field '["Items"][0]["Id"]')"
  users >"$OUT/users-before.json"
  say "films: $n; users: $(cat "$OUT/users-before.json")"

  # --- the page: a hosted one when PAGE names it, as for real phones (the
  # operator's Demo B, https://group.svrnme.sh/); else web/ served on
  # localhost, where every name under it answers ---------------------------
  if [ -z "${PAGE:-}" ]; then
    "$REPO/scripts/build-web.sh" --out "$R/site" >"$R/build.log"
    node "$REPO/web/tests/serve.mjs" "$R/site" >"$R/site.out" 2>&1 & echo $! >"$R/site.pid"
    PAGE="http://localhost:$(port_of "$R/site.out")/"
  fi
  say "the page: $PAGE"

  # --- Sam makes House, its links on the page; Alex joins by the client's --
  SVRNGS_DIR="$R/sam" RUST_LOG=info,carriage=debug NO_COLOR=1 nohup "$B" new House --as Sam --origin "$PAGE" >"$R/sam/new.out" 2>"$R/sam/run.log" &
  echo $! >"$R/sam.pid"
  for _ in $(seq 1 60); do grep -q "$PAGE#" "$R/sam/new.out" 2>/dev/null && break; sleep 1; done
  LINK="$(grep -o "$PAGE#[A-Za-z0-9_-]*" "$R/sam/new.out")"
  local client; client="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$R/sam/new.out")"
  say "sam: $(grep -i 'phone' "$R/sam/new.out" || head -1 "$R/sam/new.out")"
  # Each join with a HOME of its own: joining places the group's folder
  # under it (cli.rs `folder_place`), never in this host's.
  HOME="$R/alex" SVRNGS_DIR="$R/alex" RUST_LOG=info NO_COLOR=1 nohup "$B" join "$client" --as Alex >"$R/alex/join.out" 2>"$R/alex/join.log" &
  echo $! >"$R/alex.pid"
  for _ in $(seq 1 60); do grep -q "You're in" "$R/alex/join.out" 2>/dev/null && break; sleep 1; done
  say "alex: $(head -1 "$R/alex/join.out")"
  # Sam lends it as it comes, eight at once for the phones unless a run
  # says otherwise (SHARE_FLAGS); a password typed is not asked for, and
  # not kept.
  printf 'MediaBrowser Token="%s"\n' "$TOKEN" | SVRNGS_DIR="$R/sam" NO_COLOR=1 RUST_LOG=debug "$B" share 8096 $SHARE_FLAGS >"$R/sam/share.out" 2>"$R/sam/share.log"
  say "sam: $(grep -m1 'You lend' "$R/sam/share.out")"
  local how; how="$(grep -m1 'asks each person to sign in once' "$R/sam/share.out")" || { say "share told Sam nothing of accounts"; exit 1; }
  say "sam is told: $how"
  [ ! -e "$R/sam/secrets/lends/jellyfin" ] || { say "the lend keeps a secret"; exit 1; }
  # Sam makes Ana and Ben accounts, as that line says, in Jellyfin's pages.
  SECRETS="$SECRETS" node "$HERE/lender.mjs" "$how" "$O" sam $PEOPLE >"$OUT/lender.json" || { say "lender: $(cat "$OUT/lender.json")"; exit 1; }
  users >"$OUT/users-made.json"
  say "sam made: $(cat "$OUT/users-made.json")"
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
THREE=$THREE
PAGE=$PAGE
LINK=$LINK
ORIGIN=$ORIGIN
KEPT=${KEEP_BACK:-}
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
  local mw="" a_at="" b_at=""
  if [ -n "${MW:-}" ]; then
    # Through the middleware's lend, started afresh: two people new to it.
    # On a computer they are Alex and Bo, each from their own computer's
    # address of the lend, as the door names them; a phone is whoever it
    # joined as.
    [ -n "${MW_PORT_LENT:-}" ] || { say "pair: MW=1 needs \`check.sh mw\` run first"; exit 2; }
    mw_up "pair $way $engine"
    [ "$way" = computer ] && mw_forget
    tag="mw-$tag${JELLYFIN_MW_PLANT:+-plant-$JELLYFIN_MW_PLANT}"
    mw=',"mw":true,"lend":"jellyfin-mw"'
    [ "$way" = computer ] && { a=Alex; b=Bo; a_at=",\"at\":\"$ALEX_AT\""; b_at=",\"at\":\"$BO_AT\""; }
  fi
  rm -rf "$R/dump/$tag"
  SECRETS="$SECRETS" JF="$O" ADMIN="$(cat "$SECRETS/admin_token")" DUMP="$R/dump/$tag" \
    node "$HERE/drive.mjs" "{\"way\":\"$way\",\"engine\":\"$engine\",\"at\":\"$at\",\"server\":\"$SERVER\"$mw,
      \"people\":[{\"name\":\"$a\",\"user\":\"ana\",\"film\":\"$ONE\",\"seeks\":[50,15,80,30,65]$a_at${A_EXTRA:-}},
                  {\"name\":\"$b\",\"user\":\"ben\",\"film\":\"$TWO\",\"seeks\":[60,20,75]$b_at${B_EXTRA:-}}]}" \
    >"$OUT/pair-$tag.json" 2>"$R/pair-$tag.err" || true
  if [ -n "$mw" ]; then
    python3 "$HERE/tokens.py" "$R/jf/data/data/jellyfin.db" "$OUT/pair-$tag.json" "$R/dump/$tag" | tee -a "$OUT/steps.log"
    cp "$R/mw.log" "$R/mw-$tag.log"
  fi
  say "pair $tag: $(python3 "$HERE/summary.py" --line "$OUT/pair-$tag.json")"
}

# The lend made to keep a token, as a lender who pasted one would have
# before u5-4 (crates/svrngs/src/cli.rs, prompt_secret): written where
# `share` keeps one (svrngs_node::lend::declare), which the door reads on
# every request, so every request carries the lend's one device. With it,
# nobody signing in as themselves gets past their sign-in (the first
# token plant, 2026-10-05: 403 on each person's own settings), so each
# plant also gives the browsers what the failure it plants was seen with:
#   token       a user of the lend's own, `zed`, and both browsers holding a
#               stored sign-in as zed (drive.mjs `seed`), as u5-3's probe
#               ran (demo/runs/u5/2026-10-05-jellyfin/README.md, finding 3);
#   one-device  Ana's own token on the lend's device, Ana's browser holding
#               a stored sign-in as her, and Ben signing in with Ana's
#               account, the one account for everyone, as U3's dry run's
#               member signed in as its viewer (demo/runs/a/dry-2026-10-04/
#               README.md:39-50). Jellyfin 12 ends a sign-in only when the
#               same account signs in again on the same device (this host,
#               2026-10-05: one device, two accounts, 200 and 200; one
#               account twice, 401 then 200).
plant() {
  . "$R/env"
  local what="$1" engine="$2" user pw dev="svrngs-lend-$1"
  TOKEN="$(cat "$SECRETS/admin_token")"
  id_of() { api GET /Users | python3 -c "import json,sys;print(next(u['Id'] for u in json.load(sys.stdin) if u['Name']=='$1'))"; }
  case "$what" in
    token) user=zed; pw=""
      # One left by a run stopped before its end goes first.
      if id="$(id_of zed 2>/dev/null)"; then api DELETE "/Users/$id" >/dev/null; fi
      api POST /Users/New '{"Name":"zed"}' >/dev/null
      A_EXTRA=",\"seed\":\"$(id_of zed)\",\"account\":\"zed\""; B_EXTRA="$A_EXTRA" ;;
    one-device) user=ana; pw="$(cat "$SECRETS/ana_pw")"
      A_EXTRA=",\"seed\":\"$(id_of ana)\""; B_EXTRA=',"account":"ana"' ;;
    *) echo "plant: token|one-device" >&2; exit 2 ;;
  esac
  local lend='MediaBrowser Client="svrngs lend", Device="Sam computer", DeviceId="'"$dev"'", Version="1"'
  local t; t="$(curl -sS -f -X POST "$O/Users/AuthenticateByName" -H "Content-Type: application/json" -H "Authorization: $lend" \
    -d "{\"Username\":\"$user\",\"Pw\":\"$pw\"}" | field '["AccessToken"]')"
  secret "plant_${what}_lend_token" "$t"
  local kept="$R/sam/secrets/lends/jellyfin"
  (umask 077; mkdir -p "$kept"; printf '%s, Token="%s"' "$lend" "$t" >"$kept/authorization")
  say "plant $what: the lend keeps $user's token, on device $dev"
  A_EXTRA="$A_EXTRA" B_EXTRA="$B_EXTRA" RUN="plant-$what" pair computer "$engine"
  rm -rf "$kept"
  [ "$what" = token ] && api DELETE "/Users/$(id_of zed)" >/dev/null
  say "plant $what: taken back; the lend keeps $(find "$R/sam/secrets/lends" -type f 2>/dev/null | wc -l | tr -d ' ') secrets"
}

# Demo A's three on this host (u5-9's row; demo/a.md): the host's load
# at the start and the end in steps.log beside the seeks, which carry
# their own (ralph/next/u5/PROMPT.addendum.md).
trio() {
  . "$R/env"
  local tag="trio${RUN:+-$RUN}"
  local people="
      {\"name\":\"Alex\",\"user\":\"alex\",\"way\":\"computer\",\"engine\":\"chromium\",\"at\":\"$ORIGIN\",\"film\":\"$ONE\",\"seeks\":10},
      {\"name\":\"Bo\",\"user\":\"bo\",\"way\":\"phone\",\"engine\":\"webkit\",\"at\":\"$LINK\",\"film\":\"$TWO\",\"seeks\":22},
      {\"name\":\"Cy\",\"user\":\"cy\",\"way\":\"phone\",\"engine\":\"chromium\",\"at\":\"$LINK\",\"film\":\"$THREE\",\"seeks\":22}" mw=""
  if [ -n "${MW:-}" ]; then
    # Demo A with nothing typed: the middleware's lend, started afresh, so
    # all three are new to it; each on a computer of their own.
    [ -n "${CY_AT:-}" ] || { say "trio: MW=1 needs \`check.sh mw\` run first with COMPUTERS=\"alex bo cy\""; exit 2; }
    mw_up "trio $tag"
    mw_forget
    tag="mw-$tag"; mw=',"mw":true'
    people="
      {\"name\":\"Alex\",\"user\":\"alex\",\"way\":\"computer\",\"engine\":\"chromium\",\"at\":\"$ALEX_AT\",\"film\":\"$ONE\",\"seeks\":10},
      {\"name\":\"Bo\",\"user\":\"bo\",\"way\":\"computer\",\"engine\":\"webkit\",\"at\":\"$BO_AT\",\"film\":\"$TWO\",\"seeks\":10},
      {\"name\":\"Cy\",\"user\":\"cy\",\"way\":\"computer\",\"engine\":\"chromium\",\"at\":\"$CY_AT\",\"film\":\"$THREE\",\"seeks\":10}"
  fi
  say "trio $tag: load at the start: $(uptime)"
  rm -rf "$R/dump/$tag"
  SECRETS="$SECRETS" JF="$O" ADMIN="$(cat "$SECRETS/admin_token")" DUMP="$R/dump/$tag" \
    node "$HERE/trio.mjs" "{\"server\":\"$SERVER\",\"window_s\":${WINDOW:-600}$mw,\"people\":[$people]}" \
    >"$OUT/$tag.json" 2>"$R/$tag.err" || true
  say "trio $tag: load at the end: $(uptime)"
  if [ -n "$mw" ]; then
    # What each account the middleware made may open, as Jellyfin lists it
    # to that account: a library kept back is in none.
    TOKEN="$(cat "$SECRETS/admin_token")"
    api GET /Users | python3 -c 'import json,re,sys;[print(u["Id"],u["Name"]) for u in json.load(sys.stdin) if re.search(r"-[0-9a-f]{16}$",u["Name"])]' |
      while read -r id name; do
        printf '{"user":"%s","views":%s}\n' "$name" "$(api GET "/UserViews?userId=$id" | python3 -c 'import json,sys;print(json.dumps([v["Name"] for v in json.load(sys.stdin)["Items"]]))')"
      done >"$OUT/views-$tag.json"
    say "trio $tag: each account opens: $(tr '\n' ' ' <"$OUT/views-$tag.json"); kept back: ${KEPT:-none}"
  fi
  python3 "$HERE/scan.py" "$R" "$OUT" | tee -a "$OUT/steps.log"
  python3 "$HERE/trio.py" "$OUT/$tag.json" "$R/sam/run.log" | tee -a "$OUT/steps.log"
}

# Jellyfin's middleware (mw-1; since mwp-3 the lend's sign-in): the lender
# mints it a key once, runs it beside Jellyfin and lends Jellyfin naming
# it, `share 8096 --sign-in <its port>`; Alex and Bo, two computers, ask
# through the lend (mw.py). Only users named as the
# middleware names them (<name>-<16 hex of a key>, viewer.rs user_name)
# are deleted between runs, so a person's first ask in each run is new.
# The middleware built, given its key (minted once), and started afresh
# on MW_PORT (else 8097), JELLYFIN_MW_PLANT passed on, the users it named
# before deleted.
mw_up() {
  local port="${MW_PORT:-8097}" mwb="$REPO/apps/jellyfin/mw/target/debug/jellyfin-mw"
  TOKEN="$(cat "$SECRETS/admin_token")"
  cargo build --quiet --manifest-path "$REPO/apps/jellyfin/mw/Cargo.toml"
  if [ ! -s "$SECRETS/mw_key" ]; then
    api POST "/Auth/Keys?app=svrngs-mw" >/dev/null
    secret mw_key "$(api GET /Auth/Keys | python3 -c 'import json,sys;print(next(k["AccessToken"] for k in json.load(sys.stdin)["Items"] if k["AppName"]=="svrngs-mw"))')"
  fi
  if [ -f "$R/mw.pid" ]; then kill "$(cat "$R/mw.pid")" 2>/dev/null || true; rm -f "$R/mw.pid"; sleep 1; fi
  api GET /Users | python3 -c 'import json,re,sys;[print(u["Id"]) for u in json.load(sys.stdin) if re.search(r"-[0-9a-f]{16}$",u["Name"])]' |
    while read -r id; do api DELETE "/Users/$id" >/dev/null; done
  NO_COLOR=1 RUST_LOG=info,jellyfin_mw=debug nohup "$mwb" --listen "127.0.0.1:$port" ${KEEP_BACK:+--keep-back "$KEEP_BACK"} \
    <"$SECRETS/mw_key" >"$R/mw.out" 2>"$R/mw.log" &
  echo $! >"$R/mw.pid"
  for _ in $(seq 1 50); do grep -q 'listening on port' "$R/mw.out" 2>/dev/null && break; sleep 0.1; done
  say "mw $1: $(tr '\n' ' ' <"$R/mw.out")${JELLYFIN_MW_PLANT:+; planted: $JELLYFIN_MW_PLANT}; users: $(users)"
}

# The door keeps each person's sign-in answer while Sam's svrngs runs, and
# mw_up just deleted the accounts they name: one request through each
# computer's address, refused 401 by Jellyfin, makes the door forget it
# (crates/svrngs-node/src/door/sign_in.rs, `Forget`), so Alex and Bo are
# new to it again, and no plant's answer outlives its run. Phones join as
# new people each pair, so only a computer pair needs it.
mw_forget() {
  local who at code
  for who in ALEX BO; do
    at="$(sed -n "s/^${who}_AT=//p" "$R/env" | tail -1)"
    code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 60 "${at}Users/Me")"
    say "mw: $who's kept answer, its account deleted, asked once more: $code"
    [ "$code" = 401 ] || { say "mw: $who's answer was not refused, so the door may keep it: stopping"; exit 3; }
  done
}

mw() {
  . "$R/env"
  local tag="$1" port="${MW_PORT:-8097}"
  mw_up "$tag"
  # A run after the first: the door keeps Alex's and Bo's answers from it,
  # whose accounts mw_up just deleted.
  [ -z "${MW_PORT_LENT:-}" ] || mw_forget
  if [ -z "${MW_PORT_LENT:-}" ]; then
    SVRNGS_DIR="$R/sam" NO_COLOR=1 RUST_LOG=debug "$B" share 8096 --name jellyfin-mw --sign-in "$port" ${MW_SHARE_FLAGS---limit 8} </dev/null >"$R/sam/share-mw.out" 2>>"$R/sam/share.log"
    say "sam: $(grep -m1 'You lend' "$R/sam/share-mw.out")"
    # What share tells a lender who names a sign-in: no accounts to make.
    say "sam is told: $(grep -v '^You lend' "$R/sam/share-mw.out" | tr '\n' ' ')"
    local client; client="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$R/sam/new.out")"
    # Each computer but Alex's joins once; a run stopped after it does not
    # join again.
    local who at computers="${COMPUTERS:-alex bo}"
    for who in $computers; do
      [ "$who" != alex ] && [ ! -d "$R/$who/rings" ] || continue
      mkdir -p "$R/$who"
      HOME="$R/$who" SVRNGS_DIR="$R/$who" RUST_LOG=info NO_COLOR=1 nohup "$B" join "$client" --as "$(printf '%s' "${who:0:1}" | tr a-z A-Z)${who:1}" \
        >"$R/$who/join.out" 2>"$R/$who/join.log" &
      echo $! >"$R/$who.pid"
      for _ in $(seq 1 60); do grep -q "You're in" "$R/$who/join.out" 2>/dev/null && break; sleep 1; done
      say "$who: $(head -1 "$R/$who/join.out")"
    done
    for who in $computers; do
      kill "$(cat "$R/$who.pid")" 2>/dev/null || true; sleep 2
      SVRNGS_DIR="$R/$who" RUST_LOG=info NO_COLOR=1 nohup "$B" >"$R/$who/run.out" 2>"$R/$who/run.log" &
      echo $! >"$R/$who.pid"
      for _ in $(seq 1 60); do grep -q 'jellyfin-mw.*in your browser' "$R/$who/run.out" 2>/dev/null && break; sleep 1; done
      at="$(grep 'jellyfin-mw.*in your browser' "$R/$who/run.out" | grep -o 'http://[a-z0-9-]*\.localhost:[0-9]*/')"
      [ -n "$at" ] || { say "$who's bare command lists no jellyfin-mw"; cat "$R/$who/run.out" >&2; exit 3; }
      say "$who's computer opens the middleware's lend at $at"
      printf '%s_AT=%s\n' "$(echo "$who" | tr a-z A-Z)" "$at" >>"$R/env"
    done
    printf 'MW_PORT=%s\nMW_PORT_LENT=1\n' "$port" >>"$R/env"
  fi
  python3 "$HERE/mw.py" "$R" "$OUT" "$tag" | tee -a "$OUT/steps.log"
}

# mw-2's runs (its run.sh), as one step, so a later run of them (mwp-3's,
# demo/runs/mwp/2026-10-08/run.sh) is the same sequence. Each step runs as
# its own check.sh, as the run.sh did; a plant's judge fails by design.
mw2() {
  local C="$HERE/check.sh" plant way engine
  "$C" up "$R" "$OUT"
  "$C" mw "$R" "$OUT" mw1-again
  for plant in no-sign-in lender-key one-device; do
    JELLYFIN_MW_PLANT=$plant MW=1 "$C" pair "$R" "$OUT" computer chromium
    "$C" judge "$R" "$OUT" "mw-computer-chromium-plant-$plant" || true
  done
  for way in computer phone; do for engine in chromium webkit; do
    MW=1 "$C" pair "$R" "$OUT" $way $engine
  done; done
  "$C" judge "$R" "$OUT" mw-computer-chromium mw-computer-webkit mw-phone-chromium mw-phone-webkit
  "$C" down "$R" "$OUT"
}

case "$STEP" in
  films) films "$R" "$OUT" ;;
  up) up ;;
  pair) pair "$4" "$5" ;;
  trio) trio ;;
  mw) mw "$4" ;;
  mw2) mw2 ;;
  plant) plant "$4" "$5" ;;
  judge) shift 3; python3 "$HERE/scan.py" "$R" "$OUT" >/dev/null; python3 "$HERE/judge.py" "$OUT" "$@" | tee -a "$OUT/steps.log" ;;
  down) TOKEN="$(cat "$SECRETS/admin_token")"; users >"$OUT/users-after.json" 2>/dev/null || true; down ;;
  *) echo "usage: check.sh films|up|pair|trio|plant|mw|mw2|judge|down <scratch> <out> [way engine | plant engine | run...]" >&2; exit 2 ;;
esac
