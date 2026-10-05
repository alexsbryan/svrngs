#!/usr/bin/env bash
# U4's dry run (DESIGN §8, U4; §11 risks 4 and 5), on one host: what a phone
# will pay to open a group's link and play a lent film, measured in
# Playwright's headless Chromium and its WebKit as an iPhone, cold each run.
# Not a run of U4's bar — no phone, no https origin; NEEDS_HUMAN.md is that.
#
#   1. the page built (scripts/build-web.sh): each file's bytes, raw, gzip
#      and brotli, as an origin may send them;
#   2. a film made by ffmpeg, lent by lend.mjs (byte ranges);
#   3. two passes, each Sam's group House with its links on the page served
#      at 127.0.0.1: `local`, Sam's computer reached through iroh's relay on
#      this computer (crates/svrngs/tests/phone_dry.rs, since the command
#      takes no relay); `public`, the `svrngs` command as a person runs it,
#      reached through iroh's public relays and address lookup;
#   4. in each, $RUNS cold runs per engine (drive.mjs): link opened → name
#      asked → list, the link again, the lend opened, its first frame and
#      each seek. Every run's line goes to runs-<pass>.jsonl, and every
#      answer the lent service gave during the pass to lend-<pass>.log;
#      summary.py reads every pass's runs in <out dir>.
#
# Each cold run is a new person, and a lend lets 3 people at once by
# default, counting each for 30 s after their last request
# (crates/svrngs-node/src/door/limit.rs:38): a fourth run inside that
# window is refused 503, which this run's first try met. So the lend here
# lets 16.
#
# usage: dry.sh <scratch dir> <out dir>
#   RUNS (3), ENGINES ("chromium webkit"), PASSES ("local public"): one pass
#   an invocation stays inside ten minutes on this host; a pass run again
#   replaces only its own files.
set -euo pipefail
R="$1"; OUT="$2"
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../../../.." && pwd)"
B="$REPO/target/debug/svrngs"
RUNS="${RUNS:-3}"; ENGINES="${ENGINES:-chromium webkit}"; PASSES="${PASSES:-local public}"
SEEKS='[60, 20, 100, 5, 110]'
NAMES=(Ana Ben Cleo Dara Eli Fay Gus Hana Ivo Jo Kai Lea Max Nia Oto Pia)
say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$OUT/steps.log" >&2; }
# A background process's first `port <n>` line.
port_of() { for _ in $(seq 1 100); do grep -m1 -o 'port [0-9]*' "$1" 2>/dev/null | cut -d' ' -f2 && return; sleep 0.1; done; return 1; }

stop() { for p in "$R"/*.pid; do [ -f "$p" ] && kill "$(cat "$p")" 2>/dev/null || true; done; touch "$R/local/stop" 2>/dev/null || true; }
trap stop EXIT
rm -rf "$R"; mkdir -p "$R"/{lend,local,public} "$OUT"
say "dry.sh: passes $PASSES, engines $ENGINES, $RUNS runs each"

# --- the page, and what it weighs --------------------------------------------
( cd "$REPO" && cargo build -q -p svrngs && cargo test -q -p svrngs --test phone_dry --no-run 2>/dev/null )
"$REPO/scripts/build-web.sh" --out "$R/site" >"$R/build.log"
node --input-type=module - "$R/site" "$OUT/bytes.json" <<'EOF'
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { brotliCompressSync, constants, gzipSync } from 'node:zlib';
const [root, out] = process.argv.slice(2);
const walk = (d) => readdirSync(d).flatMap((f) => (statSync(join(d, f)).isDirectory() ? walk(join(d, f)) : [join(d, f)]));
const files = walk(root).sort().map((f) => {
  const b = readFileSync(f);
  return { file: relative(root, f), raw: b.length, gzip: gzipSync(b, { level: 9 }).length,
    brotli: brotliCompressSync(b, { params: { [constants.BROTLI_PARAM_QUALITY]: 11 } }).length };
});
// The total is what the page fetches: wasm-bindgen's .d.ts files are built, and never fetched.
const fetched = files.filter((f) => !f.file.endsWith('.d.ts'));
const total = Object.fromEntries(['raw', 'gzip', 'brotli'].map((k) => [k, fetched.reduce((n, f) => n + f[k], 0)]));
writeFileSync(out, JSON.stringify({ files, total }, null, 1) + '\n');
EOF
say "the page: $(grep -o 'version [0-9a-f]*' "$R/build.log"); $(python3 -c "import json;t=json.load(open('$OUT/bytes.json'))['total'];print(f\"{t['raw']} bytes raw, {t['gzip']} gzip, {t['brotli']} brotli\")")"

# --- the film, lent; the page, served -----------------------------------------
# 120 s of 720p H.264 near 3 Mb/s, a keyframe each 2 s, its index first:
# noise keeps the encoder from shrinking it, so a seek needs bytes unfetched.
ffmpeg -v error -y -f lavfi -i "testsrc2=duration=120:size=1280x720:rate=24" -vf "noise=alls=12:allf=t" \
  -c:v libx264 -preset veryfast -b:v 3M -maxrate 3M -bufsize 6M -g 48 -pix_fmt yuv420p -movflags +faststart \
  "$R/lend/title.mp4"
say "the film: $(stat -f %z "$R/lend/title.mp4") bytes, 120 s, 1280x720"
node "$HERE/lend.mjs" "$R/lend" >"$R/lend.out" 2>&1 & echo $! >"$R/lend.pid"
LEND="$(port_of "$R/lend.out")"
node "$HERE/drive.mjs" serve "$R/site" >"$R/site.out" 2>&1 & echo $! >"$R/site.pid"
ORIGIN="http://127.0.0.1:$(port_of "$R/site.out")/"
say "lent on 127.0.0.1:$LEND; the page at $ORIGIN"

# --- the passes ---------------------------------------------------------------
n=0
runs() {  # runs <pass> <link>
  for engine in $ENGINES; do
    for i in $(seq 1 "$RUNS"); do
      name="${NAMES[$((n % ${#NAMES[@]}))]}"; n=$((n + 1))
      served="$(wc -l <"$R/site.out")"
      line="$(node "$HERE/drive.mjs" run "{\"engine\":\"$engine\",\"link\":\"$2\",\"name\":\"$name\",\"lend\":\"film\",\"seeks\":$SEEKS}" 2>>"$R/drive.err" | tail -1)"
      [ -n "$line" ] || line='{"stopped": "drive.mjs printed nothing"}'
      # What the page's origin sent during the run: its lines past `served`.
      python3 -c 'import json,sys
d = json.loads(sys.argv[1]); sent = [l.split() for l in open(sys.argv[4]).read().splitlines()[int(sys.argv[5]):]]
d.update({"pass": sys.argv[2], "run": int(sys.argv[3]), "origin_requests": len(sent), "origin_bytes": sum(int(l[2]) for l in sent)})
print(json.dumps(d))' "$line" "$1" "$i" "$R/site.out" "$served" >>"$OUT/runs-$1.jsonl"
      say "$1 $engine $i: $(tail -1 "$OUT/runs-$1.jsonl" | python3 "$HERE/summary.py" --line)"
    done
  done
}
for pass in $PASSES; do
  : >"$OUT/runs-$pass.jsonl"; lent="$(wc -l <"$R/lend.out")"
  case "$pass" in
    local)
      ( cd "$REPO" && DRY_ORIGIN="$ORIGIN" DRY_LEND="$LEND" DRY_LIMIT=16 DRY_DIR="$R/local" \
          cargo test -q -p svrngs --test phone_dry -- --ignored --nocapture ) >"$R/local/stage.log" 2>&1 &
      echo $! >"$R/local.pid"
      for _ in $(seq 1 120); do [ -s "$R/local/link" ] && break; sleep 0.5; done
      [ -s "$R/local/link" ] || { say "local: Sam's computer printed no link"; cat "$R/local/stage.log" >&2; exit 3; }
      say "local: Sam's computer up, through the relay on 127.0.0.1"
      runs local "$(cat "$R/local/link")"
      touch "$R/local/stop"; wait "$(cat "$R/local.pid")" || true ;;
    public)
      SVRNGS_DIR="$R/public/sam" NO_COLOR=1 "$B" new House --as Sam --origin "$ORIGIN" >"$R/public/new.out" 2>"$R/public/run.log" &
      echo $! >"$R/public.pid"
      for _ in $(seq 1 120); do grep -q "^ *$ORIGIN#" "$R/public/new.out" 2>/dev/null && break; sleep 0.5; done
      LINK="$(grep -o "$ORIGIN#[A-Za-z0-9_-]*" "$R/public/new.out" || true)"
      [ -n "$LINK" ] || { say "public: svrngs new printed no link on the page"; cat "$R/public/new.out" "$R/public/run.log" >&2; exit 3; }
      SVRNGS_DIR="$R/public/sam" NO_COLOR=1 "$B" share "$LEND" --kind film --limit 16 </dev/null >"$R/public/share.out" 2>&1
      say "public: $(head -1 "$R/public/share.out")"
      runs public "$LINK"
      kill "$(cat "$R/public.pid")" 2>/dev/null || true ;;
    *) say "no pass $pass: local or public"; exit 2 ;;
  esac
  tail -n +"$((lent + 1))" "$R/lend.out" >"$OUT/lend-$pass.log"
done

python3 "$HERE/summary.py" "$OUT/bytes.json" "$OUT"/runs-*.jsonl >"$OUT/summary.txt"
cat "$OUT/summary.txt"
{ echo "svrngs $(git -C "$REPO" rev-parse --short HEAD)$(git -C "$REPO" diff --quiet HEAD -- crates web || echo ' (+ uncommitted)')"
  sw_vers | tr '\n' ' '; echo; sysctl -n machdep.cpu.brand_string; node --version
  node -e "console.log('playwright', require('$REPO/web/node_modules/playwright/package.json').version)"
} >"$OUT/host.txt"
say "done: $OUT"
