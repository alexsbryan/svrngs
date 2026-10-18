#!/usr/bin/env bash
# Scoped use's run (su-1's, ralph/next/compose/STATE.md; the rule, bars and
# predictions in demo/runs/su/2026-10-09/README.md, su-0), run again on
# what svrngs carries since sup-1 to sup-3, the computer first (sup-4,
# ralph/next/sup/STATE.md; demo/runs/sup/2026-10-10/README.md): the group
# as mn-1's run made it (apps/movie-night/check/check.sh `up`: Sam's and
# Dee's Jellyfins through their middlewares, Cy's asleep, Movie Night,
# Alex's computer), and on it:
#
# - Ana's computer, joined as Ana, running her taste (apps/taste/taste.mjs)
#   on loopback beside a planted history, lent `share <its port> --name
#   taste --kind taste --refuse /diary --ask "$ASK"`. Its owners: the key
#   of her computer and of each of her phones, each read from Sam's
#   computer's log as it admits it (the only place a person's key across
#   devices is told; PLAN §4, "Then").
# - Tonight's Picks (apps/movie-night/picks), served by a plain file
#   server, lent by Sam `--name tonights-picks --kind movie-night --reads
#   jellyfin --reads taste`.
# - The other app (check/other), lent by Sam `--name other --kind other
#   --reads taste`: a plant's page, counted nowhere.
#
#   check.sh up       <scratch> <out>          all of the above
#   check.sh computer <scratch> <out> <engine> Ana's computer and Alex's, from each one's home page, in
#                                              <engine> as a desktop: every step of su.mjs but `join` and
#                                              `asleep`; writes <out>/computer-<engine>.json
#   check.sh phone    <scratch> <out> <engine> Ana's two phones, <engine> first and the other engine
#                                              second, and Bo in Chromium: every step of su.mjs but
#                                              `home-stop` and `asleep`; <out>/phone-<engine>.json
#   check.sh asleep   <scratch> <out>          Ana's phone in Chromium, her yes, then her computer put to
#                                              sleep after the list (su.mjs `asleep`); <out>/asleep.json
#   check.sh plant    <scratch> <out> <plant>  one plant on, on the computer in Chromium — bo, history,
#                                              other-app, after-stop, empty — a journey that meets it,
#                                              then the plant taken back
#   check.sh judge    <scratch> <out>          scan.py over every byte each browser received
#                                              for the history's markers, then judge.py
#   check.sh down     <scratch> <out>          stops everything
#
# Each step stays inside ten minutes on this host; each journey starts on
# a taste lent afresh, so no choice is kept. svrngs built first: cargo
# build -p svrngs; and what mn's check needs: Jellyfin.app, Playwright's
# browsers (web/tests), the middleware. scripts/check.sh runs none of
# this: it needs Jellyfin.
set -euo pipefail
STEP="$1"; R="$2"; OUT="$3"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../.." && pwd)"
MN="$REPO/apps/movie-night/check/check.sh"
B="$REPO/target/debug/svrngs"
TASTE_PORT=8696; PICKS_PORT=8697; OTHER_PORT=8698
# Sam's computer admits each joiner: its key, at debug, is how Ana's
# owners are read.
export LOG='info,door=debug,origin=debug,redeem=debug,home=debug,surface=debug'
# The lender's words for what an app receives (svrn-docs
# rings/current/SCOPED_USE_MADE_PRODUCTION.md §4, its example).
ASK='the app receives ranked film suggestions; your viewing history stays where it is stored'
mkdir -p "$OUT"
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
admitted() { grep 'redeem: admitted' "$R/sam/run.log" | tail -1 | grep -o 'key=[0-9a-f]*' | cut -d= -f2; }
ana() { SVRNGS_DIR="$R/ana" NO_COLOR=1 RUST_LOG=debug "$B" "$@" </dev/null >>"$R/ana/share.out" 2>>"$R/ana/share.log"; }
sam() { SVRNGS_DIR="$R/sam" NO_COLOR=1 RUST_LOG=debug "$B" "$@" </dev/null >>"$R/sam/share.out" 2>>"$R/sam/share.log"; }

# Ana's taste, its switch set to PLANT (else none).
taste() {
  [ -f "$R/taste.pid" ] && { kill "$(cat "$R/taste.pid")" 2>/dev/null || true; sleep 1; }
  TASTE_PLANT="${PLANT:-}" nohup node "$REPO/apps/taste/taste.mjs" $TASTE_PORT "$R/taste" >>"$R/taste.log" 2>&1 &
  echo $! >"$R/taste.pid"
  for _ in $(seq 1 50); do curl -s -o /dev/null "http://127.0.0.1:$TASTE_PORT/" && break; sleep 0.1; done
  say "taste: up${PLANT:+ with plant $PLANT}"
}

# The taste lent afresh, so its door keeps no one's choice (`unshare` takes
# them): as kept, `--refuse /diary --ask`, or as a plant names it.
lent() {
  local how="${1:-kept}"
  [ -f "$R/ana/lent-taste" ] && ana unshare taste
  case "$how" in
    kept) ana share $TASTE_PORT --name taste --kind taste --refuse /diary --ask "$ASK" --limit 8 ;;
    no-refuse) ana share $TASTE_PORT --name taste --kind taste --ask "$ASK" --limit 8 ;;
    no-ask) ana share $TASTE_PORT --name taste --kind taste --refuse /diary --limit 8 ;;
  esac
  touch "$R/ana/lent-taste"
  say "taste lent ($how), no choice kept: $(grep 'You lend' "$R/ana/share.out" | tail -1)"
}

# Tonight's Picks as the check serves it: its links copied as files, and
# plant.js naming PLANT (else none).
picks_site() {
  rm -rf "$R/picks-site"; mkdir -p "$R/picks-site"
  rsync -aL "$REPO/apps/movie-night/picks/" "$R/picks-site/"
  [ -n "${PLANT:-}" ] && printf "export const PLANT = '%s';\nexport const TOKEN = null;\n" "$PLANT" >"$R/picks-site/plant.js"
  return 0
}

# The history: rows of films Ana watched, each with a marker of its own,
# and the order they give Tonight's Picks' films, written before the run.
# Each marker is kept in <scratch>/secrets too, where scan.py searches.
plant_history() {
  mkdir -p "$R/taste"
  python3 - "$R/taste/diary.json" "$R/secrets" <<'EOF'
import json, secrets, sys
from pathlib import Path
rows = [("Arrival", 2016, {"tmdb": "329865", "imdb": "tt2543164"}, 5, "2026-09-12T21:04:00Z"),
        ("Arrival", 2016, {"tmdb": "329865", "imdb": "tt2543164"}, 5, "2026-10-01T20:40:00Z"),
        ("Brazil", 1985, {"tmdb": "68", "imdb": "tt0088846"}, 4, "2026-08-30T22:15:00Z"),
        ("Blade Runner", 1982, {"tmdb": "78", "imdb": "tt0083658"}, 2, "2026-07-19T21:30:00Z"),
        ("Solaris", 1972, {"tmdb": "593"}, 5, "2026-06-02T23:10:00Z")]
out = [{"title": t, "year": y, "ids": i, "rating": r, "at": at, "marker": f"su-diary-{n}-{secrets.token_hex(8)}"}
       for n, (t, y, i, r, at) in enumerate(rows, 1)]
# The group's films ranked by these rows: Arrival 5, Brazil 4, Blade
# Runner 2; Alien, never watched, after them.
doc = {"rows": out, "expected": ["Arrival", "Brazil", "Blade Runner", "Alien"]}
Path(sys.argv[1]).write_text(json.dumps(doc, indent=1) + "\n")
for row in out:
    (Path(sys.argv[2]) / f"diary_row_{row['marker'].split('-')[2]}").write_text(row["marker"])
EOF
  cp "$R/taste/diary.json" "$OUT/diary-planted.json"
}

up() {
  "$MN" up "$R" "$OUT"
  plant_history
  local client; client="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$R/sam/new.out")"
  mkdir -p "$R/ana"
  SVRNGS_DIR="$R/ana" RUST_LOG="$LOG" NO_COLOR=1 nohup "$B" join "$client" --as Ana >"$R/ana/join.out" 2>"$R/ana/run.log" &
  echo $! >"$R/ana.pid"
  for _ in $(seq 1 60); do grep -q "You're in" "$R/ana/join.out" 2>/dev/null && break; sleep 1; done
  say "ana: $(head -1 "$R/ana/join.out")"
  admitted >"$R/taste/owners"
  say "ana's taste: owned by her computer's key $(cat "$R/taste/owners")"
  PLANT='' taste
  lent kept
  PLANT='' picks_site
  nohup python3 -m http.server $PICKS_PORT --bind 127.0.0.1 --directory "$R/picks-site" >"$R/picks-server.log" 2>&1 &
  echo $! >"$R/picks-server.pid"
  nohup python3 -m http.server $OTHER_PORT --bind 127.0.0.1 --directory "$HERE/other" >"$R/other-server.log" 2>&1 &
  echo $! >"$R/other-server.pid"
  sleep 1
  sam share $PICKS_PORT --name tonights-picks --kind movie-night --reads jellyfin --reads taste --limit 8
  sam share $OTHER_PORT --name other --kind other --reads taste --limit 8
  say "sam: $(grep 'You lend' "$R/sam/share.out" | tail -2 | tr '\n' ' ')"
  cp "$R/ana/share.out" "$OUT/ana-share.out"; cp "$R/sam/share.out" "$OUT/sam-share.out"
  # Ana's computer learns of those who joined before it by its ring round.
  for _ in $(seq 1 150); do
    grep 'round membership exchanged_with=' "$R/ana/run.log" | grep 'Dee/' | grep -q 'Sam/' && break; sleep 1
  done
  say "ana: $(grep -o 'round membership exchanged_with=.*' "$R/ana/run.log" | tail -1)"
  # Alex's computer learns of Ana's by its own round, so it can dial her door.
  for _ in $(seq 1 150); do
    grep 'round membership exchanged_with=' "$R/alex/run.log" | grep -q 'Ana/' && break; sleep 1
  done
  say "alex: $(grep -o 'round membership exchanged_with=.*' "$R/alex/run.log" | tail -1)"
  homes
}

# Each computer's home page, from the address its node wrote.
homes() {
  local who
  for who in ana alex; do
    echo "$(echo $who | tr a-z A-Z)_HOME=http://svrngs.localhost:$(sed 's/.*://' "$R/$who/api")/" >>"$R/env"
  done
  say "home pages: $(grep _HOME "$R/env" | tr '\n' ' ')"
}

drive() {
  local tag="$1" people="$2" steps="$3"
  # shellcheck disable=SC1090
  . "$R/env"
  # Tonight's Picks served as it is now, with plant.js naming `empty` only
  # for that plant.
  PLANT="$([ "${PLANT:-}" = empty ] && echo empty || true)" picks_site
  rm -rf "$R/dump/$tag"
  local t0; t0="$(date -u +%Y-%m-%dT%H:%M:%S)"
  DUMP="$R/dump/$tag" \
    OWN="k=\$(grep 'redeem: admitted' '$R/sam/run.log' | tail -1 | grep -o 'key=[0-9a-f]*' | cut -d= -f2); echo \$k >>'$R/taste/owners'; echo \$k" \
    SLEEP="kill \$(cat '$R/ana.pid') && rm -f '$R/ana.pid'" AFTER_STOP="${AFTER_STOP:-}" \
    node "$HERE/su.mjs" "{\"tag\":\"$tag\",\"link\":\"$LINK\",\"steps\":$steps,\"people\":$people}" >"$OUT/$tag.json" 2>"$R/$tag.err" \
    || say "$tag: the driver exited $?: $(tail -3 "$R/$tag.err")"
  # What the taste and Ana's door said during it, and the choices it kept.
  awk -v t0="$t0" '$1 >= t0' "$R/taste.log" >"$OUT/$tag-taste.log"
  grep -h 'taste\|lend=taste\|home:' "$R/ana/run.log" 2>/dev/null | awk -v t0="$t0" '$1 >= t0' >"$OUT/$tag-ana-door.log" || true
  grep -h 'home:\|origin:' "$R/alex/run.log" 2>/dev/null | awk -v t0="$t0" '$1 >= t0' >"$OUT/$tag-alex.log" || true
  cp "$R/ana/choices/taste.json" "$OUT/$tag-choices.json" 2>/dev/null || echo '{}' >"$OUT/$tag-choices.json"
  say "$tag: $(python3 "$HERE/judge.py" --line "$OUT/$tag.json")"
}

ana_phone() { echo "{\"name\":\"Ana\",\"user\":\"ana-$1\",\"engine\":\"$1\"}"; }
BO='{"name":"Bo","user":"bo","engine":"chromium"}'
# Ana's and Alex's computers, in <engine> as a desktop, from their home pages.
computers() {
  # shellcheck disable=SC1090
  . "$R/env"
  echo "{\"name\":\"Ana\",\"user\":\"ana-computer-$1\",\"engine\":\"$1\",\"home\":\"$ANA_HOME\"},{\"name\":\"Alex\",\"user\":\"alex-computer-$1\",\"engine\":\"$1\",\"home\":\"$ALEX_HOME\"}"
}

case "$STEP" in
  up) up ;;
  computer)
    PLANT='' taste; lent kept
    drive "computer-$4" "[$(computers "$4")]" '["yes","again","other-person","diary","other","stop","home-stop"]' ;;
  phone)
    first="$4"; second=webkit; [ "$first" = webkit ] && second=chromium
    PLANT='' taste; lent kept
    drive "phone-$first" "[$(ana_phone "$first"),$(ana_phone $second),$BO]" '["join","yes","again","other-person","diary","other","stop"]' ;;
  asleep)
    PLANT='' taste; lent kept
    drive asleep "[$(ana_phone chromium)]" '["join","yes","asleep"]'
    if [ -f "$R/ana.pid" ]; then kill "$(cat "$R/ana.pid")"; rm -f "$R/ana.pid"; say "ana: put to sleep, the driver having not"; fi ;;
  plant)
    PLANT="$4"; export PLANT
    case "$PLANT" in
      bo) taste; lent kept; steps='["other-person"]' ;;
      history) PLANT='' taste; lent no-refuse; steps='["yes","diary"]' ;;
      other-app) PLANT='' taste; lent no-ask; steps='["other"]' ;;
      after-stop)
        # A door that kept no stop: the stop's record put back to the yes it replaced.
        PLANT='' taste; lent kept; steps='["yes","stop"]'
        export AFTER_STOP="perl -pi -e 's/\"said\": \"stop\"/\"said\": \"yes\"/' '$R/ana/choices/taste.json' && cat '$R/ana/choices/taste.json'" ;;
      empty) PLANT='' taste; lent kept; steps='["other-person"]' ;;
      *) echo "plant: bo|history|other-app|after-stop|empty" >&2; exit 2 ;;
    esac
    say "plant $PLANT: on"
    drive "plant-$PLANT" "[$(computers chromium)]" "$steps"
    unset AFTER_STOP
    [ "$PLANT" = empty ] && PLANT='' picks_site
    PLANT='' taste; lent kept; say "plant $4 taken back" ;;
  judge)
    python3 "$REPO/apps/jellyfin/check/scan.py" "$R" "$OUT" | tee -a "$OUT/steps.log"
    python3 "$HERE/judge.py" "$OUT" | tee -a "$OUT/steps.log" ;;
  down) "$MN" down "$R" "$OUT" ;;
  *) echo "usage: check.sh up|computer|phone|asleep|plant|judge|down <scratch> <out> [engine | plant]" >&2; exit 2 ;;
esac
