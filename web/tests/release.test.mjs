// No release carries v2 before the switchover (page-scaffold (g);
// ralph/next/page/CHARTER.md, "What is settled" 5): the home's v2 page is
// served only by a test build, under the `test-support` feature, so no
// crate turns that on outside its dev-dependencies, and the release and the
// page's deploy name no feature and no `--ui`. And scripts/build-release.sh
// refuses a built svrngs that carries v2's trace, `svrngs-ui-v2`
// (crates/svrngs-node/src/home/v2/serve.rs), whatever turned it on.

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

const ROOT = new URL('../..', import.meta.url).pathname;
const read = (f) => readFileSync(join(ROOT, f), 'utf8');

/** Each line of a Cargo.toml's `text` that gives `test-support` to a build that is not a test's. */
function leaks(file, text) {
  let section = '', inDefault = false;
  return text.split('\n').flatMap((line, i) => {
    const head = line.match(/^\s*\[([^\]]+)\]/);
    if (head) [section, inDefault] = [head[1].trim(), false];
    if (section === 'features' && /^\s*default\s*=/.test(line)) inDefault = true;
    const built = /(^|\.)(build-)?dependencies(\.|$)/.test(section) && !/dev-dependencies/.test(section);
    const said = (built || inDefault) && line.includes('test-support') ? [`${file}:${i + 1} [${section}] ${line.trim()}`] : [];
    if (inDefault && line.includes(']')) inDefault = false;
    return said;
  });
}

test('no crate gives test-support to anything but a test', () => {
  const tomls = spawnSync('git', ['ls-files', '*Cargo.toml'], { cwd: ROOT, encoding: 'utf8' }).stdout.split('\n').filter(Boolean);
  assert.ok(tomls.includes('crates/svrngs/Cargo.toml') && tomls.includes('crates/svrngs-node/Cargo.toml'), tomls.join(' '));
  assert.deepEqual(tomls.flatMap((f) => leaks(f, read(f))), []);
});

test("test-support under svrngs's [dependencies] is caught, by line", () => {
  const planted = read('crates/svrngs/Cargo.toml').replace(/^\[dependencies\]$/m, '[dependencies]\nsvrngs-node = { workspace = true, features = ["test-support"] }');
  assert.deepEqual(leaks('crates/svrngs/Cargo.toml', planted).map((l) => l.split(' ')[1]), ['[dependencies]']);
});

test('the release and the deploy name no feature and no --ui', () => {
  for (const f of ['scripts/build-release.sh', 'scripts/deploy-page.sh']) {
    assert.deepEqual(read(f).match(/--(all-)?features|--ui\b/g), null, f);
  }
});

/** build-release.sh's refusal, its one line, run on a built `svrngs` of `bytes`: [what it said, failed]. */
function refusal(bytes) {
  const lines = read('scripts/build-release.sh').split('\n').filter((l) => l.includes('svrngs-ui-v2'));
  assert.equal(lines.length, 1, 'build-release.sh refuses a binary carrying svrngs-ui-v2, in one line');
  const dir = mkdtempSync(join(tmpdir(), 'svrngs-release-'));
  mkdirSync(join(dir, 'target/x-triple/release'), { recursive: true });
  writeFileSync(join(dir, 'target/x-triple/release/svrngs'), bytes);
  const run = spawnSync('bash', ['-c', `set -euo pipefail; failed=()
    for t in "x x-triple"; do read -r platform triple <<<"$t"
${lines[0]}
      echo built; done; echo "failed: \${failed[*]}"`], { cwd: dir, encoding: 'utf8' });
  return [run.stderr.trim(), run.stdout.trim()];
}

test('build-release.sh refuses a built svrngs carrying svrngs-ui-v2, by name', () => {
  const [said, out] = refusal(Buffer.concat([Buffer.from('\x7fELF\0\0'), Buffer.from('home: chosen once (svrngs-ui-v2)'), Buffer.from([0, 1])]));
  assert.match(said, /^build-release: x's svrngs carries svrngs-ui-v2/);
  assert.equal(out, 'failed: x');
  assert.deepEqual(refusal(Buffer.from('\x7fELF\0\0home: chosen once\0')), ['', 'built\nfailed:']);
});
