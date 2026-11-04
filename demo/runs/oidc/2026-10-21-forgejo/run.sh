#!/usr/bin/env bash
# oidc-forgejo's run (ralph/next/oidc/STATE.md;
# SIGN_IN_WITH_SVRNGS §5a, §7 1): a real Forgejo 17.0.0 on this host, lent by
# Kim with `svrngs share 18097 --sign-in-with-svrngs`; Alex on another
# computer, Bo on a WebKit phone, Cy a member Kim removes.
#
# The row said under Podman. On a Mac Podman's containers run in its own
# Linux machine, so their `127.0.0.1` is that machine's, not this
# computer's (SIGN_IN_WITH_SVRNGS §8 8), and the provider's one issuer
# string is `http://127.0.0.1:<api port>/oidc/<lend>`. The design's path for
# a Mac is the app outside a container; this take uses the pinned Homebrew
# bottle and no Docker. `data/podman-probe.txt` keeps the container's side.
# The predictions are PREDICTIONS.md, committed before this and before any
# data.
#
#   run.sh [W]        the whole run, its data under data/; W is the
#                     scratch (else /tmp/oidc-forgejo)
#
# One host, one run of each: not a rate (AGENTS.md 7). A step that did not
# happen is null with what the page showed, never a number guessed.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../../.." && pwd)"
W="${1:-${W:-/tmp/oidc-forgejo}}"; OUT="$HERE/data"
B="$REPO/target/debug/svrngs"
FJ=/opt/homebrew/opt/forgejo/bin/forgejo
FJ_PORT=18097; O="http://127.0.0.1:$FJ_PORT"
FJ_SHA=fb508a2fcf4411e7792284224f7be8ae9c41fa6f550926912b22b0aff33404f1
BOTTLE_SHA=d997331b207cf62e274ab7d15cf668c242152d1146292491360f8053f48238f6
export NO_COLOR=1
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
pw() { head -c 18 /dev/urandom | base64 | tr -d '/+='; }
port_of() { for _ in $(seq 1 100); do grep -m1 -o 'port [0-9]*' "$1" 2>/dev/null | cut -d' ' -f2 && return; sleep 0.1; done; return 1; }
starts() { # starts <pidfile> <log> <command...>
  local pid="$1" log="$2"; shift 2
  "$@" >"$log" 2>&1 & echo $! >"$pid"
}
down() {
  for p in "$W"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done
  rm -f "$W"/*.pid
}
trap down EXIT

rm -rf "$OUT" "$W"/kim "$W"/alex "$W"/bo "$W"/cy "$W"/forgejo "$W"/site "$W"/env "$W"/env.json "$W"/probe-www
mkdir -p "$OUT" "$W"/{kim,alex,bo,cy} "$W"/forgejo/{custom/conf,data,log,tmp} "$W"/site "$W"/secrets "$W"/probe-www
: >"$OUT/steps.log"
say "host: $(uname -m) $(sw_vers -productVersion), load $(uptime | sed 's/.*load/load/')"

# --- the instrument, pinned ---------------------------------------------------
[ -x "$B" ] || { say "svrngs is not built: cargo build -p svrngs"; exit 3; }
[ -x "$FJ" ] || {
  say "Forgejo is not installed. On this Mac the design's path is the app outside a container: brew install forgejo"
  echo could-not-judge >"$OUT/verdicts.json"
  exit 3
}
GOT_SHA="$(shasum -a 256 "$FJ" | cut -d' ' -f1)"
[ "$GOT_SHA" = "$FJ_SHA" ] || { say "Forgejo is $GOT_SHA, not the pinned $FJ_SHA"; exit 3; }
FJV="$("$FJ" --version 2>&1 | head -1)"
say "pinned: $FJV, sha256 $GOT_SHA (bottle $BOTTLE_SHA)"
say "svrngs $(git -C "$REPO" rev-parse --short HEAD), $(stat -f '%Sm' -t '%Y-%m-%d %H:%M:%S' "$B")"

# --- the row's Podman premise, kept as the reason ----------------------------
mkdir -p "$W/probe-www"; echo probe >"$W/probe-www/index.html"
python3 -m http.server 18779 --bind 127.0.0.1 --directory "$W/probe-www" >"$W/probe-www.log" 2>&1 &
PROBE=$!
echo "$PROBE" >"$W/probe.pid"
sleep 1
{
  echo "# The row's Podman premise on this Mac, 2026-10-21"
  echo "# A probe answer on this Mac's 127.0.0.1, asked three ways."
  echo
  echo "## this Mac: curl http://127.0.0.1:18779/"
  curl -sS --max-time 5 http://127.0.0.1:18779/ || echo "refused"
  echo
  echo "## podman --network=host: its 127.0.0.1 is the Podman machine's, not this Mac's"
  podman run --rm --network=host docker.io/library/alpine:3.20 \
    sh -c 'wget -qO- -T 5 http://127.0.0.1:18779/ || echo "refused"' 2>&1 || echo "podman refused to run"
  echo
  echo "## podman, bridged: host.containers.internal reaches this Mac's loopback"
  podman run --rm docker.io/library/alpine:3.20 \
    sh -c 'wget -qO- -T 5 http://host.containers.internal:18779/ || echo "refused"' 2>&1 || echo "podman refused to run"
  echo
  echo "# The provider's one issuer string is http://127.0.0.1:<api port>/oidc/<lend>"
  echo "# (crates/svrngs-node/src/oidc/provider.rs:62-65), which a container's"
  echo "# 127.0.0.1 cannot name, so Forgejo runs outside a container, as"
  echo "# SIGN_IN_WITH_SVRNGS §8 8 says a Mac's app does."
} >>"$OUT/podman-probe.txt" 2>&1 || true
kill "$PROBE" 2>/dev/null || true

# --- Forgejo, its own work path, install locked ------------------------------
(umask 077; [ -f "$W/secrets/kim_pw" ] || pw >"$W/secrets/kim_pw")
KIMPW="$(cat "$W/secrets/kim_pw")"
SK="$("$FJ" generate secret SECRET_KEY)"
IT="$("$FJ" generate secret INTERNAL_TOKEN)"
cat >"$W/forgejo/custom/conf/app.ini" <<INI
APP_NAME = Forgejo
RUN_USER = $(whoami)
RUN_MODE = prod
WORK_PATH = $W/forgejo

[server]
PROTOCOL = http
DOMAIN = localhost
HTTP_ADDR = 127.0.0.1
HTTP_PORT = $FJ_PORT
ROOT_URL = http://localhost:$FJ_PORT/
DISABLE_SSH = true
START_SSH_SERVER = false
OFFLINE_MODE = true
LFS_START_SERVER = false
APP_DATA_PATH = $W/forgejo/data

[database]
DB_TYPE = sqlite3
PATH = $W/forgejo/data/forgejo.db

[repository]
ROOT = $W/forgejo/data/repositories

[security]
INSTALL_LOCK = true
SECRET_KEY = $SK
INTERNAL_TOKEN = $IT

[service]
DISABLE_REGISTRATION = false
REGISTER_EMAIL_CONFIRM = false
REGISTER_MANUAL_CONFIRM = false
ENABLE_NOTIFY_MAIL = false

[oauth2]
ENABLED = true

[oauth2_client]
ENABLE_AUTO_REGISTRATION = true
USERNAME = preferred_username
REGISTER_EMAIL_CONFIRM = false

[log]
MODE = console
LEVEL = Info

[session]
PROVIDER = file
PROVIDER_CONFIG = $W/forgejo/data/sessions
INI
sed -E 's/^(SECRET_KEY|INTERNAL_TOKEN) = .*/\1 = <redacted>/' \
  "$W/forgejo/custom/conf/app.ini" >"$OUT/forgejo.ini"
"$FJ" migrate --work-path "$W/forgejo" >"$W/forgejo/migrate.log" 2>&1 || {
  say "Forgejo did not migrate: $(tail -3 "$W/forgejo/migrate.log")"; exit 3; }
"$FJ" admin user create --work-path "$W/forgejo" --admin --username kim \
  --email kim@example.invalid --password "$KIMPW" --must-change-password=false >"$W/forgejo/admin.log" 2>&1 || {
  say "Forgejo did not make Kim: $(tail -3 "$W/forgejo/admin.log")"; exit 3; }
starts "$W/fj.pid" "$W/forgejo/web.log" "$FJ" web --work-path "$W/forgejo"
ok=0
for _ in $(seq 1 90); do
  if curl -sf --max-time 2 "$O/api/v1/version" >/dev/null 2>&1; then ok=$((ok + 1)); [ $ok -ge 2 ] && break; else ok=0; fi
  sleep 1
done
[ $ok -ge 2 ] || { say "Forgejo did not come up: $(tail -8 "$W/forgejo/web.log")"; exit 3; }
say "forgejo up: $(curl -sS "$O/api/v1/version")"

# --- the page and the group --------------------------------------------------
"$REPO/scripts/build-web.sh" --out "$W/site" >"$W/build.log"
starts "$W/site.pid" "$W/site.out" node "$REPO/web/tests/serve.mjs" "$W/site"
PAGE="http://localhost:$(port_of "$W/site.out")/"
say "the page: $PAGE"
starts "$W/kim.pid" "$W/kim/new.out" env SVRNGS_DIR="$W/kim" RUST_LOG=info,door=debug,oidc=debug NO_COLOR=1 "$B" new House --as Kim --origin "$PAGE"
for _ in $(seq 1 90); do grep -q "$PAGE#" "$W/kim/new.out" 2>/dev/null && break; sleep 1; done
LINK="$(grep -o "$PAGE#[A-Za-z0-9_-]*" "$W/kim/new.out" | head -1)"
CLIENT_LINK="$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' "$W/kim/new.out" | head -1)"
[ -n "$CLIENT_LINK" ] || { say "Kim's link was not made: $(cat "$W/kim/new.out")"; exit 3; }
say "kim: $(grep -i 'phone' "$W/kim/new.out" || grep -v '^$' "$W/kim/new.out" | head -1)"
for who in Alex Bo Cy; do
  lwho="$(printf '%s' "$who" | tr 'A-Z' 'a-z')"
  starts "$W/$lwho.pid" "$W/$lwho/join.out" env HOME="$W/$lwho" SVRNGS_DIR="$W/$lwho" \
    RUST_LOG=info NO_COLOR=1 "$B" join "$CLIENT_LINK" --as "$who"
  for _ in $(seq 1 120); do grep -q "You're in" "$W/$lwho/join.out" 2>/dev/null && break; sleep 1; done
  grep -q "You're in" "$W/$lwho/join.out" || { say "$who did not join: $(cat "$W/$lwho/join.out")"; exit 3; }
  say "$who: $(head -1 "$W/$lwho/join.out")"
done

# --- Kim lends it, and Forgejo is told --------------------------------------
SVRNGS_DIR="$W/kim" NO_COLOR=1 RUST_LOG=debug "$B" share "$FJ_PORT" --name forgejo --kind forgejo \
  --sign-in-with-svrngs --limit 8 </dev/null >"$W/kim/share.out" 2>"$W/kim/share.log"
say "kim: $(grep -m1 'You lend' "$W/kim/share.out")"
ISSUER="$(grep -o 'issuer http://127.0.0.1:[0-9]*/oidc/[A-Za-z0-9-]*' "$W/kim/share.out" | head -1 | cut -d' ' -f2)"
CID="$(grep -o 'client id [A-Za-z0-9._~-]*' "$W/kim/share.out" | head -1 | cut -d' ' -f3)"
CSECRET="$(grep -o 'client secret [A-Za-z0-9._~-]*' "$W/kim/share.out" | head -1 | cut -d' ' -f3)"; CSECRET="${CSECRET%.}"
[ -n "$ISSUER" ] && [ -n "$CID" ] && [ -n "$CSECRET" ] || { say "share printed no issuer/client: $(cat "$W/kim/share.out")"; exit 3; }
say "the app's settings: issuer $ISSUER, client id $CID, a secret of ${#CSECRET} characters"
sed -E "s/$CSECRET/<redacted>/g" "$W/kim/share.out" >"$OUT/share.out"

# The API itself, from the container the row asked for: the host-network
# container's 127.0.0.1 cannot name it, and the bridged one reaches it by
# host.containers.internal (the caller read as loopback). Kept as evidence.
{
  echo
  echo "## the API itself, from podman --network=host at 127.0.0.1:<api>"
  podman run --rm --network=host docker.io/library/alpine:3.20 \
    sh -c "wget -qO- -T 5 '$ISSUER/.well-known/openid-configuration' || echo 'refused'" 2>&1 || echo "podman refused to run"
  echo
  echo "## the API itself, bridged, by host.containers.internal:<api>"
  REPLACED="$(printf '%s' "$ISSUER" | sed -E 's#http://127\.0\.0\.1:#http://host.containers.internal:#')"
  podman run --rm docker.io/library/alpine:3.20 \
    sh -c "wget -qO- -T 5 '$REPLACED/.well-known/openid-configuration' || echo 'refused'" 2>&1 || echo "podman refused to run"
} >>"$OUT/podman-probe.txt" 2>&1 || true

"$FJ" admin auth add-oauth --work-path "$W/forgejo" --name svrngs --provider openidConnect \
  --key "$CID" --secret "$CSECRET" \
  --auto-discover-url "$ISSUER/.well-known/openid-configuration" \
  --scopes openid --scopes profile --scopes groups >"$W/forgejo/oauth.log" 2>&1 || {
  say "Forgejo did not take the source: $(tail -5 "$W/forgejo/oauth.log")"; exit 3; }
"$FJ" admin auth list --work-path "$W/forgejo" >"$OUT/forgejo-auth.txt" 2>&1 || true
say "forgejo's source: $(grep -i svrngs "$OUT/forgejo-auth.txt" | head -1)"
ADMIN_TOKEN="$("$FJ" admin user generate-access-token --work-path "$W/forgejo" \
  --username kim --token-name oidc-run --scopes all --raw 2>/dev/null)"
[ -n "$ADMIN_TOKEN" ] || { say "Forgejo made Kim no admin token"; exit 3; }
curl -sS "$O/user/login" >"$OUT/forgejo-login.html"
if grep -q 'user/oauth2/svrngs' "$OUT/forgejo-login.html"; then
  say "forgejo's login page offers: Sign in with svrngs"
else
  say "forgejo's login page offers no svrngs source: $(grep -o 'Sign in[^<]*' "$OUT/forgejo-login.html" | head -3)"
  exit 3
fi

# Each computer's bare command, after the lend reaches it, prints the
# lend's own address on that computer; a phone reaches it through the page.
for who in Alex Cy; do
  lwho="$(printf '%s' "$who" | tr 'A-Z' 'a-z')"
  kill "$(cat "$W/$lwho.pid")" 2>/dev/null || true
  while kill -0 "$(cat "$W/$lwho.pid")" 2>/dev/null; do sleep 0.2; done
  starts "$W/$lwho.pid" "$W/$lwho/run.out" env SVRNGS_DIR="$W/$lwho" RUST_LOG=info NO_COLOR=1 "$B"
  for _ in $(seq 1 120); do grep -q 'forgejo.*in your browser' "$W/$lwho/run.out" 2>/dev/null && break; sleep 1; done
  at="$(grep 'forgejo.*in your browser' "$W/$lwho/run.out" | grep -o 'http://[a-z0-9-]*\.localhost:[0-9]*/' | head -1)"
  [ -n "$at" ] || { say "$who's bare command lists no forgejo"; cat "$W/$lwho/run.out" >&2; exit 3; }
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

python3 - "$W" "$PAGE" "$LINK" "$ISSUER" "$CID" "$CSECRET" "$O" "$FJ_PORT" \
  "$(sed -n 's/^ALEX_AT=//p' "$W/env" | tail -1)" "$BO_AT" "$BO_LINK" \
  "$(sed -n 's/^CY_AT=//p' "$W/env" | tail -1)" "$B" "$KIMPW" "$ADMIN_TOKEN" "$FJ" <<'PY'
import json, sys
w, page, link, issuer, cid, secret, fj, port, alex_at, bo_at, bo_link, cy_at, bin, kim_pw, admin, fjb = sys.argv[1:]
json.dump({
    "page": page, "link": link, "issuer": issuer, "client_id": cid, "client_secret": secret,
    "fj": fj, "fj_port": int(port), "alex_at": alex_at, "bo_at": bo_at, "bo_link": bo_link,
    "cy_at": cy_at, "bin": bin, "kim_dir": f"{w}/kim", "kim_log": f"{w}/kim/new.out",
    "kim_pw": kim_pw, "admin_token": admin, "fj_bin": fjb, "fj_work": f"{w}/forgejo",
}, open(f"{w}/env.json", "w"))
PY

say "load at the drive's start: $(uptime | sed 's/.*load/load/')"
node "$HERE/people.mjs" "$W/env.json" "$OUT/raw.json" >"$OUT/driver.out" 2>"$W/driver.err" || true
say "load at the drive's end: $(uptime | sed 's/.*load/load/')"
mkdir -p "$OUT/forgejo-log"; cp "$W/forgejo/log/"*.log "$OUT/forgejo-log/" 2>/dev/null || true
cp "$W/forgejo/web.log" "$OUT/forgejo-web.log" 2>/dev/null || true
python3 "$HERE/judge.py" "$OUT" | tee -a "$OUT/steps.log" || true
python3 "$REPO/scripts/lean_check.py" >"$OUT/lean.txt" 2>&1 || true
if git -C "$REPO" diff --quiet -- scripts/trust_floor.txt; then FLOOR=unchanged; else FLOOR=changed; fi
python3 "$HERE/setup.py" "$OUT" "$FLOOR" "$FJV" "$GOT_SHA" "$BOTTLE_SHA"
say "setup.json: pinned versions, the trust floor $FLOOR, lean in lean.txt"
