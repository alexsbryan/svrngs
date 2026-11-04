#!/usr/bin/env bash
# The dev page, end to end, as a person follows what `node web/tests/dev.mjs`
# prints: ada's computer (target/debug/svrngs, a data dir and $HOME of the
# run's own) makes house and carries Money, the test's app
# (crates/svrngs-node/tests/browser/carried.rs
# `an_app_s_entry_is_approved_by_the_group_s_signer_and_carried`); `svrngs
# link house --origin` points it at the dev page, refused first at the https
# address and taken at http://localhost; then drive.mjs, in Chrome as the
# printed line opens it, joins as bo, opens Money, saves, and asks ada's
# computer whether it counts the entry. `plain` opens the link as svrngs
# printed it, on http://localhost: the plant.
#
#   lockf -k /tmp/svrngs-gate.lock demo/runs/dev-page/2026-11-02/run.sh [plain]
set -u
cd "$(dirname "$0")/../../../.."
here=demo/runs/dev-page/2026-11-02
t=$(mktemp -d)
pids=()
trap 'kill "${pids[@]}" 2>/dev/null; rm -rf "$t"' EXIT
waitfor() { for _ in $(seq 1 120); do grep -q "$1" "$2" 2>/dev/null && return 0; sleep 0.5; done; echo "never: $1 in $2" >&2; cat "$2" >&2; exit 1; }

node web/tests/dev.mjs >"$t/dev.txt" 2>&1 & pids+=($!)
waitfor 'Ctrl-C stops this' "$t/dev.txt"
echo "== node web/tests/dev.mjs"; cat "$t/dev.txt"
line=$(grep 'open -na' "$t/dev.txt" | sed 's/^ *//')
port=$(grep -o 'group.svrnme.test:[0-9]*' "$t/dev.txt" | head -1 | cut -d: -f2)

home=$HOME
export HOME="$t/home" SVRNGS_DIR="$t/data"
mkdir -p "$HOME"
target/debug/svrngs new house --as ada >"$t/new.txt" 2>&1 & pids+=($!)
waitfor 'Leave this running' "$t/new.txt"
kill -INT "${pids[1]}"; wait "${pids[1]}" 2>/dev/null
money="$HOME/svrngs/house/apps/Money"
mkdir -p "$money"
echo '<!doctype html><title>…</title><script type=module src=app.js></script>' >"$money/index.html"
cat >"$money/app.js" <<'EOF'
window.save = async (what) => {
  const r = await fetch('svrngs/acts', { method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ op: { kind: 'expense', amount_cents: 1, description: what } }) });
  return { status: r.status, said: await r.text() };
};
document.title = 'ready';
EOF

echo "== svrngs link house --origin https://group.svrnme.test:$port/"
target/debug/svrngs link house --origin "https://group.svrnme.test:$port/" 2>&1; echo "exit $?"
echo "== svrngs link house --origin http://localhost:$port/"
target/debug/svrngs link house --origin "http://localhost:$port/" >"$t/link.txt" 2>&1 & pids+=($!)
waitfor 'Leave this running' "$t/link.txt"
cat "$t/link.txt"
link=$(grep -o "http://localhost:$port/#[A-Za-z0-9_-]*" "$t/link.txt")
group=$(ls "$SVRNGS_DIR/rings")

echo "== drive.mjs${1:+ $1}"
HOME=$home node "$here/drive.mjs" "$(node -e 'console.log(JSON.stringify({ line: process.argv[1], link: process.argv[2], data: process.argv[3], group: process.argv[4], plain: process.argv[5] === "plain" }))' "$line" "$link" "$SVRNGS_DIR" "$group" "${1:-}")"
echo "exit $?"
