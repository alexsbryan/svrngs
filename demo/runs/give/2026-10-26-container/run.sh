#!/bin/sh
# give-session (c): a container on this Mac, through podman's gvproxy, asks
# svrngs on the Mac's loopback at host.containers.internal:<api port>, as
# GIVE.md §5 measured it could before give-token and give-session; then the
# person's own `svrngs open` opens the home page. svrngs is this build's
# target/debug/svrngs over a scratch data dir under target/; the container
# is python:3.12-slim, named svrngs-test-container and removed at the end.
#
#   sh demo/runs/give/2026-10-26-container/run.sh > demo/runs/give/2026-10-26-container/run.out 2>&1
set -eu
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../../.." && pwd)
scratch="$repo/target/give-container"
bin="$repo/target/debug/svrngs"
rm -rf "$scratch"
mkdir -p "$scratch/home" "$scratch/stubs"
say() { printf '%s\n' "$*"; }

say "== svrngs $(git -C "$repo" rev-parse --short HEAD), podman $(podman --version | cut -d' ' -f3), $(sw_vers -productName) $(sw_vers -productVersion)"
NO_COLOR=1 HOME="$scratch/home" SVRNGS_DIR="$scratch/data" RUST_LOG=session=debug,home=debug,rails=debug \
  "$bin" >"$scratch/svrngs.out" 2>"$scratch/svrngs.log" &
pid=$!
trap 'kill $pid 2>/dev/null || true; podman rm -f svrngs-test-container >/dev/null 2>&1 || true' EXIT
tries=0
until [ -s "$scratch/data/api" ] && [ -s "$scratch/data/token" ] && curl -s -o /dev/null "$(cat "$scratch/data/api")/"; do
  tries=$((tries + 1)); [ "$tries" -lt 150 ] || { say "svrngs did not answer"; exit 1; }
  sleep 0.2
done
api=$(cat "$scratch/data/api")
port=${api##*:}
say "svrngs answers on the Mac at $api"

say "" "== 1. The container asks, as any program in it can"
podman run --rm --name svrngs-test-container docker.io/library/python:3.12-slim python3 -c '
import sys, urllib.request, urllib.error
port = sys.argv[1]
asks = [
    ("the record, its groups", "127.0.0.1:" + port, "GET", "/v1/groups", {}),
    ("an open, minted for it", "127.0.0.1:" + port, "POST", "/v1/open", {"content-type": "application/json"}),
    ("the home page, typed", "svrngs.localhost:" + port, "GET", "/", {"Sec-Fetch-Site": "none", "Sec-Fetch-Mode": "navigate"}),
    ("the home page, its yes", "svrngs.localhost:" + port, "POST", "/choices",
     {"Sec-Fetch-Site": "same-origin", "Origin": "http://svrngs.localhost:" + port, "content-type": "application/json"}),
]
for what, host, method, path, headers in asks:
    req = urllib.request.Request("http://host.containers.internal:" + port + path, method=method,
                                 data=b"{}" if method == "POST" else None, headers={"Host": host, **headers})
    try:
        with urllib.request.urlopen(req, timeout=10) as r:
            status, body = r.status, r.read()
    except urllib.error.HTTPError as e:
        status, body = e.code, e.read()
    except Exception as e:
        status, body = "no answer", str(e).encode()
    text = body.decode("utf-8", "replace").replace("\n", " ")
    print(f"{what}: {method} {host}{path} -> {status} {text[:160]}")
' "$port"

say "" "== 2. The person opens the home page from their computer: svrngs open"
printf '#!/bin/sh\necho "$1" >"%s/opened"\n' "$scratch" >"$scratch/stubs/open"
chmod +x "$scratch/stubs/open"
PATH="$scratch/stubs:$PATH" HOME="$scratch/home" SVRNGS_DIR="$scratch/data" "$bin" open
opened=$(cat "$scratch/opened")
say "the browser was handed: $(printf '%s' "$opened" | sed 's/code=.*/code=…/')"
path="/${opened#http://*/}"
redeemed=$(curl -s -D - -o /dev/null -H "Host: svrngs.localhost:$port" "http://127.0.0.1:$port$path")
say "redeemed: $(printf '%s' "$redeemed" | head -1 | tr -d '\r'); $(printf '%s' "$redeemed" | grep -i '^location:' | tr -d '\r'); set-cookie: $(printf '%s' "$redeemed" | grep -ci '^set-cookie: svrngs=')"
cookie=$(printf '%s' "$redeemed" | sed -n 's/^[Ss]et-[Cc]ookie: \(svrngs=[^;]*\).*/\1/p')
page=$(curl -s -w '\n%{http_code}' -H "Host: svrngs.localhost:$port" -H 'Sec-Fetch-Site: none' -H 'Sec-Fetch-Mode: navigate' \
  -H "Cookie: $cookie" "http://127.0.0.1:$port/")
say "the home page, with its session: $(printf '%s' "$page" | tail -1), $(printf '%s' "$page" | grep -o '<title>[^<]*</title>')"
again=$(curl -s -o /dev/null -w '%{http_code}' -H "Host: svrngs.localhost:$port" "http://127.0.0.1:$port$path")
say "the same code again: $again"

say "" "== svrngs's traces of the container's asks and the open (target/give-container/svrngs.log)"
grep -E 'session: |home: answered|api: whether' "$scratch/svrngs.log" | sed 's/^[^ ]* //' | cut -c1-260
