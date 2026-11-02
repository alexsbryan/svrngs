// No release carries what only a test asks of a node (page-scaffold (g);
// ralph/next/page/CHARTER.md, "What is settled" 5): its `test-support`
// feature, which no crate turns on outside its dev-dependencies, and which
// the release and the page's deploy never name, nor any feature.

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
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

test('the release and the deploy name no feature', () => {
  for (const f of ['scripts/build-release.sh', 'scripts/deploy-page.sh']) {
    assert.deepEqual(read(f).match(/--(all-)?features/g), null, f);
  }
});
