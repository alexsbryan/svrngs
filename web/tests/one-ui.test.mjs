// One UI (page-switchover; ralph/next/page/STATE.md): the pages are the
// ones in web/ and crates/svrngs-node/src/home/, and no switch to another
// is left anywhere in the tree. So no file names the variable that chose
// a page, the mark of the second one, its directories, the flag that
// asked for it, or the test target that drove it alone. The records,
// demo/ and ralph/, say what was, and are not read.

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

const ROOT = new URL('../..', import.meta.url).pathname;
const SELF = 'web/tests/one-ui.test.mjs';
const LEFT = /SVRNGS_UI|svrngs-ui-v2|web\/v2\/|home\/v2\/|--ui\b|page_v2/;

test('no file names a second UI, or the switch to it', () => {
  const tracked = spawnSync('git', ['ls-files', '-co', '--exclude-standard'], {
    cwd: ROOT,
    encoding: 'utf8',
  }).stdout.split('\n');
  const read = tracked.filter(
    (f) => f && f !== SELF && !/^(demo|ralph)\//.test(f) && !/\.(png|wasm|ico)$/.test(f),
  );
  assert.ok(read.includes('crates/svrngs-node/tests/browser/harness.rs'), 'the tree is read');
  const left = read.flatMap((f) => {
    let text;
    try {
      text = readFileSync(join(ROOT, f), 'utf8');
    } catch {
      return [];
    }
    return text
      .split('\n')
      .flatMap((l, i) => (LEFT.test(l) ? [`${f}:${i + 1}: ${l.trim().slice(0, 120)}`] : []));
  });
  assert.deepEqual(left, []);
});
