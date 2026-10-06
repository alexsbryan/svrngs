#!/usr/bin/env bash
# The catalogue dry run (u5-catalogue-dry, ralph/next/u5/STATE.md): five of
# DESIGN §1's apps in podman on this host, each lent by Sam with
# `svrngs share <port> --kind <name>`, and Bo, on a phone through the page
# by the link — WebKit as an iPhone, and Chromium — doing each one's first
# real act on the floor and with the wildcard (DESIGN §7). Each outcome is
# judged against its prediction in CENSUS.md, committed before this ran.
#
#   run.sh apps  <scratch>                        the five in podman, each set up by its lender (lender.mjs)
#   run.sh up    <scratch> <out>                  the page on localhost, Sam's group with the wildcard, the five lent
#   run.sh floor <scratch> <out>                  a new link with every app under one address (the floor)
#   run.sh act   <scratch> <out> <app> <way> <engine>
#                                                 Bo does <app>'s act (act.mjs); way wildcard|floor, engine
#                                                 chromium|webkit; writes <out>/<app>-<way>-<engine>.json
#   run.sh down  <scratch>                        svrngs and the page stopped; the five keep running
#   run.sh apps-down <scratch>                    the five removed
#
# <scratch> must be under $HOME: podman's machine shares only that with
# its containers. Each step stays inside ten minutes on this host. svrngs
# built first: cargo build -p svrngs. The page is served on 127.0.0.1 and
# answers every name under localhost, so `--origin http://localhost:<p>/`
# gives each app an address of its own (u5-2), and `--origin
# http://127.0.0.1:<p>/ --lends-origin http://localhost:<p>/` puts them all
# under one, the floor. The run's load average is kept beside it (`uptime`
# at each act's start and end, ralph/next/u5/PROMPT.addendum.md).
set -euo pipefail
STEP="$1"; R="$2"; OUT="${3:-}"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../../.." && pwd)"
B="$REPO/target/debug/svrngs"
SECRETS="$R/secrets"
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
port_of() { for _ in $(seq 1 100); do grep -m1 -o 'port [0-9]*' "$1" 2>/dev/null | cut -d' ' -f2 && return; sleep 0.1; done; return 1; }
pw() { head -c 18 /dev/urandom | base64 | tr -d '/+='; }
waiting() { for _ in $(seq 1 "$2"); do curl -s -o /dev/null --max-time 3 "$1" && return; sleep 2; done; echo "$1 did not answer" >&2; exit 3; }

# The five, as their own docs run them, each on its own port on 127.0.0.1.
APPS="home-assistant:8123 immich:2283 nextcloud:8080 actual:5006 paperless:8000"

apps() {
  mkdir -p "$R"/ha "$R"/actual "$R"/immich/upload
  (umask 077; mkdir -p "$SECRETS")
  [ -f "$SECRETS/sam_pw" ] || pw >"$SECRETS/sam_pw"
  [ -f "$SECRETS/bo_pw" ] || pw >"$SECRETS/bo_pw"
  local sam bo; sam="$(cat "$SECRETS/sam_pw")"; bo="$(cat "$SECRETS/bo_pw")"
  printf 'default_config:\ninput_boolean:\n  lamp:\n    name: Lamp\n' >"$R/ha/configuration.yaml"
  podman machine start >/dev/null 2>&1 || true
  podman run -d --name cat-ha -p 127.0.0.1:8123:8123 -v "$R/ha:/config" ghcr.io/home-assistant/home-assistant:stable
  podman run -d --name cat-nc -p 127.0.0.1:8080:80 -e SQLITE_DATABASE=nextcloud -e NEXTCLOUD_ADMIN_USER=sam \
    -e NEXTCLOUD_ADMIN_PASSWORD="$sam" -e NEXTCLOUD_TRUSTED_DOMAINS=localhost docker.io/library/nextcloud:apache
  podman run -d --name cat-actual -p 127.0.0.1:5006:5006 -v "$R/actual:/data" docker.io/actualbudget/actual-server:latest
  podman pod create --name cat-immich -p 127.0.0.1:2283:2283
  podman run -d --pod cat-immich --name cat-immich-db -e POSTGRES_PASSWORD=immich -e POSTGRES_USER=immich \
    -e POSTGRES_DB=immich -e POSTGRES_INITDB_ARGS=--data-checksums ghcr.io/immich-app/postgres:14-vectorchord0.4.3-pgvectors0.2.0
  podman run -d --pod cat-immich --name cat-immich-redis docker.io/valkey/valkey:8-bookworm
  podman pod create --name cat-paperless -p 127.0.0.1:8000:8000
  podman run -d --pod cat-paperless --name cat-paperless-redis docker.io/library/redis:7
  podman run -d --pod cat-paperless --name cat-paperless-web -e PAPERLESS_SECRET_KEY="$(pw)$(pw)" \
    -e PAPERLESS_REDIS=redis://127.0.0.1:6379 -e PAPERLESS_ADMIN_USER=sam -e PAPERLESS_ADMIN_PASSWORD="$sam" \
    ghcr.io/paperless-ngx/paperless-ngx:latest
  sleep 8
  podman run -d --pod cat-immich --name cat-immich-server -e DB_HOSTNAME=127.0.0.1 -e DB_USERNAME=immich \
    -e DB_PASSWORD=immich -e DB_DATABASE_NAME=immich -e REDIS_HOSTNAME=127.0.0.1 \
    -e IMMICH_MACHINE_LEARNING_ENABLED=false -v "$R/immich/upload:/data" ghcr.io/immich-app/immich-server:release
  waiting http://127.0.0.1:8123/api/onboarding 90
  waiting http://127.0.0.1:2283/api/server/ping 90
  waiting http://127.0.0.1:8080/status.php 90
  waiting http://127.0.0.1:5006/ 90
  waiting http://127.0.0.1:8000/ 120
  for a in home-assistant immich actual paperless; do
    (umask 077; SAM_PW="$sam" node "$HERE/lender.mjs" "$a" "$bo" >"$SECRETS/$a.json")
  done
  for _ in $(seq 1 60); do podman exec -u www-data cat-nc php occ status 2>/dev/null | grep -q 'installed: true' && break; sleep 2; done
  podman exec -u www-data -e OC_PASS="$bo" cat-nc php occ user:add --password-from-env --display-name Bo bo
}

up() {
  rm -rf "$R/sam" "$R/site"; mkdir -p "$R/sam" "$OUT"
  : >"$OUT/steps.log"
  "$REPO/scripts/build-web.sh" --out "$R/site" >"$R/build.log"
  node "$REPO/demo/runs/u4/2026-10-05/drive.mjs" serve "$R/site" >"$R/site.out" 2>&1 & echo $! >"$R/site.pid"
  local p; p="$(port_of "$R/site.out")"
  local page="http://localhost:$p/"
  SVRNGS_DIR="$R/sam" RUST_LOG=info NO_COLOR=1 nohup "$B" new House --as Sam --origin "$page" >"$R/sam/new.out" 2>"$R/sam/run.log" &
  echo $! >"$R/sam.pid"
  for _ in $(seq 1 60); do grep -q "$page#" "$R/sam/new.out" 2>/dev/null && break; sleep 1; done
  grep -o "$page#[A-Za-z0-9_-]*" "$R/sam/new.out" >"$R/link-wildcard"
  say "sam: $(grep -i 'phone' "$R/sam/new.out" || head -1 "$R/sam/new.out")"
  for ap in $APPS; do
    local app="${ap%%:*}" port="${ap##*:}"
    SVRNGS_DIR="$R/sam" NO_COLOR=1 RUST_LOG=debug "$B" share "$port" --kind "$app" --name "$app" --limit 8 \
      </dev/null >"$R/sam/share-$app.out" 2>"$R/sam/share-$app.log"
    say "sam: $(grep -m1 Lent "$R/sam/share-$app.out")"
    cp "$R/sam/share-$app.out" "$OUT/sam-share-$app.out"
  done
  echo "$p" >"$R/site.port"
  sed -E 's#(svrngs://join/|'"$page"'\#)[A-Za-z0-9_-]+#\1<the link>#g' "$R/sam/new.out" >"$OUT/sam-new.out"
}

floor() {
  local p; p="$(cat "$R/site.port")"
  SVRNGS_DIR="$R/sam" NO_COLOR=1 "$B" link House --origin "http://127.0.0.1:$p/" --lends-origin "http://localhost:$p/" \
    >"$R/sam/link-floor.out" 2>"$R/sam/link-floor.log"
  grep -o "http://127.0.0.1:$p/#[A-Za-z0-9_~.:-]*" "$R/sam/link-floor.out" | head -1 >"$R/link-floor"
  say "sam, the floor: $(sed -E 's/#[^ ]+/#<the link>/' "$R/sam/link-floor.out" | tr '\n' ' ')"
  sed -E 's/#[^ ]+/#<the link>/' "$R/sam/link-floor.out" >"$OUT/sam-link-floor.out"
}

act() {
  local app="$1" way="$2" engine="$3"
  local tag="$app-$way-$engine"
  say "act $tag: load at start $(uptime | sed 's/.*load/load/')"
  SECRETS="$SECRETS" node "$HERE/act.mjs" "{\"app\":\"$app\",\"way\":\"$way\",\"engine\":\"$engine\",
    \"link\":\"$(cat "$R/link-$way")\",\"name\":\"Bo $engine $way $app\",\"shots\":\"$OUT/shots\"}" \
    >"$OUT/$tag.json" 2>"$R/act-$tag.err" || true
  say "act $tag: $(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print("did" if d.get("did") else "did not", "—", d.get("why") or d.get("saw") or "")' "$OUT/$tag.json" 2>/dev/null || tail -2 "$R/act-$tag.err")"
  say "act $tag: load at end $(uptime | sed 's/.*load/load/')"
}

down() {
  for p in "$R"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done
  rm -f "$R"/*.pid
}

apps_down() {
  podman rm -f cat-ha cat-nc cat-actual >/dev/null 2>&1 || true
  podman pod rm -f cat-immich cat-paperless >/dev/null 2>&1 || true
}

case "$STEP" in
  apps) apps ;;
  up) up ;;
  floor) floor ;;
  act) act "$4" "$5" "$6" ;;
  down) down ;;
  apps-down) apps_down ;;
  *) echo "usage: run.sh apps|up|floor|act|down <scratch> [<out>] [app way engine]" >&2; exit 2 ;;
esac
