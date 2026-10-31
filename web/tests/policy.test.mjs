// The phone's v2 page keeps the one page policy (page-scaffold (g);
// ralph/next/page/CHARTER.md, "What is settled" 4): its index.html is
// web/index.html with web/v2/body.html in its body, derived by
// web/v2/build.sh, so its <head>, the policy line (web/index.html:51)
// among it, is web/index.html's byte for byte, and scripts/trust_floor.txt
// holds that one. The build runs here as scripts/build-web.sh sources it,
// over a stand-in for the wasm it does not need, so no wasm is compiled.

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

const ROOT = new URL('../..', import.meta.url).pathname;
const POLICY = 51;

/** web/v2/build.sh, or `script`, run over a copy of web/ that `plant` (bash) changes first. */
function build({ plant = ':', script = 'web/v2/build.sh' } = {}) {
  const dir = mkdtempSync(join(tmpdir(), 'svrngs-v2-build-'));
  const env = { ...process.env, APP: join(dir, 'web'), OUT: join(dir, 'site'), PLANT: plant, BUILD: script };
  const run = spawnSync('bash', ['-c', `set -euo pipefail
    eval "$(grep -E '^(SHARED|PAGE)=\\(|^if command -v sha256sum' scripts/build-web.sh)"
    mkdir -p "$APP" "$OUT/wasm" && : >"$OUT/wasm/svrngs_web_bg.wasm" && : >"$OUT/wasm/svrngs_web.js"
    cp "\${PAGE[@]/#/web/}" "$APP/" && cp -R web/v2 web/vendor "$APP/" && cp "\${SHARED[@]}" "$OUT/"
    eval "$PLANT"
    . "$BUILD"`], { cwd: ROOT, env, encoding: 'utf8' });
  return { ...run, out: env.OUT };
}

const heads = (html) => html.match(/<head>[\s\S]*?<\/head>/g);

test("v2's index.html keeps web/index.html's <head>, its policy line included", () => {
  const index = readFileSync(join(ROOT, 'web/index.html'), 'utf8');
  const policy = index.split('\n')[POLICY - 1];
  assert.match(policy, /^<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self'/);
  const { status, stderr, out } = build();
  assert.equal(status, 0, stderr);
  // Stamped after the build's own cmp: its version read back as the literal.
  const derived = readFileSync(join(out, 'index.html'), 'utf8').replace(/\?v=[0-9a-f]{12}"/g, '?v=__VERSION__"');
  assert.deepEqual(heads(derived), heads(index));
  assert.ok(heads(derived)[0].split('\n').includes(policy), 'web/index.html:51, the policy, in the derived <head>');
  assert.ok(derived.includes(readFileSync(join(ROOT, 'web/v2/body.html'), 'utf8')), 'web/v2/body.html in its body');
});

test('a <head> of its own in body.html, unsafe-inline in it, fails the build in words', () => {
  const { status, stderr } = build({
    plant: `printf '%s' "<head><meta http-equiv=\\"Content-Security-Policy\\" content=\\"script-src 'unsafe-inline'\\"></head>" >>"$APP/v2/body.html"`,
  });
  assert.equal(status, 1, stderr);
  assert.match(stderr, /v2's index.html must keep web\/index.html's <head> byte for byte/);
});

test('ui/ left out of the stamping fails the build in words, naming each file', () => {
  const script = join(mkdtempSync(join(tmpdir(), 'svrngs-v2-build-')), 'build.sh');
  const source = readFileSync(join(ROOT, 'web/v2/build.sh'), 'utf8');
  const planted = source.replace(/^(STAMPED=\(.*)"\$\{MODULES\[@\]\}"\)$/m, `$1$(printf '%s\\n' "\${MODULES[@]}" | grep -v '^ui/'))`);
  assert.notEqual(planted, source, 'the plant took');
  writeFileSync(script, planted);
  const { status, stderr } = build({ script });
  assert.equal(status, 1, stderr);
  assert.match(stderr, /v2's site still says __VERSION__ in:.*\/ui\/view\.js/);
});

test('the trust floor holds every page policy', () => {
  const run = spawnSync('python3', ['scripts/trust_check.py'], { cwd: ROOT, encoding: 'utf8' });
  assert.equal(run.status, 0, run.stdout + run.stderr);
});
