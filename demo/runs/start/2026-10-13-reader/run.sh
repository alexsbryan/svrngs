#!/bin/sh
# start-doc's reader (ralph/next/start/STATE.md): a Claude session with no
# context but demo/start.md follows it on this host, from a clean data dir
# to an answer from the friend's model. The page is served here first, as
# the document's "Before the page is published" says; the session plays
# both friends. Its transcript and notes land in data/ beside this file,
# the group's token redacted.
#
#   W=<scratch, its site/ built by ../2026-10-13-story/run.py build> sh run.sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../../.." && pwd)
W=${W:?W names the scratch dir}
me="$W/me"
if launchctl print "gui/$(id -u)/sh.svrnme.svrngs" >/dev/null 2>&1; then
  echo "a svrngs is kept running at login here already: this run would replace it" >&2
  exit 1
fi
[ ! -e /tmp/try ] || { echo "/tmp/try exists: the document's one-computer section uses it" >&2; exit 1; }
rm -rf "$me" "$W/stub" "$here/data"
mkdir -p "$me" "$W/stub" "$here/data"
printf '#!/bin/sh\nprintf "%%s\\n" "$1" >"$HOME/opened"\n' >"$W/stub/open"
chmod +x "$W/stub/open"

node "$repo/web/tests/serve.mjs" "$W/site" >"$W/serve.log" &
page_pid=$!
trap 'kill $page_pid 2>/dev/null; launchctl bootout "gui/$(id -u)/sh.svrnme.svrngs" 2>/dev/null || true' EXIT
until grep -q '^port ' "$W/serve.log" 2>/dev/null; do sleep 0.2; done
page="http://localhost:$(sed -n 's/^port //p' "$W/serve.log" | head -1)/"

sed -e "s#{{DOC}}#$repo/demo/start.md#g" -e "s#{{PAGE}}#$page#g" -e "s#{{ME}}#$me#g" \
  -e "s#{{NODE_MODULES}}#$repo/web/node_modules#g" -e "s#{{STUB}}#$W/stub#g" -e "s#{{PY}}#${PY:-/tmp/sm-py/bin/python}#g" \
  "$here/PROMPT.md" >"$me/prompt.txt"
printf '{"head":"%s","page":"%s","start":"%s"}\n' "$(git -C "$repo" rev-parse --short HEAD)" "$page" "$(date -u +%FT%TZ)" \
  >"$here/data/start.json"

cd "$me"
t0=$(date +%s)
claude -p "$(cat "$me/prompt.txt")" --output-format stream-json --verbose \
  --permission-mode dontAsk --allowedTools Bash Read Write Edit Glob Grep \
  --add-dir "$repo/demo" >"$W/transcript.jsonl" 2>"$W/claude.err" || true
echo "{\"seconds\":$(( $(date +%s) - t0 ))}" >"$here/data/end.json"

# The group's token is a secret of the scratch group: redacted wherever it shows.
for f in "$W/transcript.jsonl" "$me/NOTES.md"; do
  [ -f "$f" ] && perl -pe 's/#[A-Za-z0-9_-]{40,}/#<the token>/g' "$f" >"$here/data/$(basename "$f")"
done
gzip -f "$here/data/transcript.jsonl"
launchctl bootout "gui/$(id -u)/sh.svrnme.svrngs" 2>/dev/null || true
pkill -f "llama-server -hf Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M --host 127.0.0.1 --port 18089" || true
pkill -f "/tmp/try/sam/.local/bin/svrngs" || true
tail -1 "$here/data/NOTES.md" 2>/dev/null || echo "no notes"
