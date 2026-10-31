// One URL per module (page-scaffold (g); ralph/next/page/CHARTER.md, "What
// is settled" 2): every import into vendor/, ui/, v2/ or question.js
// carries `?v=__VERSION__`, since two URLs for one file load it twice, and
// Preact loaded twice crashed the spike's hooks (`reading '__H'`). And only
// ui/view.js imports vendor/, so the pages hold one Preact behind one door.
// v1's files are frozen (v1.test.mjs) and not read here.

import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { posix } from 'node:path';
import { test } from 'node:test';

const ROOT = new URL('../..', import.meta.url).pathname;
const HOME = 'crates/svrngs-node/src/home';
const VIEW = `${HOME}/ui/view.js`;
const DIRS = ['web/v2', 'web/vendor', `${HOME}/ui`, `${HOME}/v2`, 'web/tests'];

/** v2's modules, the vendored ones and the tests that import them. */
function modules() {
  return DIRS.flatMap((d) => readdirSync(`${ROOT}${d}`, { recursive: true }).map((f) => `${d}/${f}`))
    .filter((f) => f.startsWith('web/tests/') ? /\.test\.mjs$|^web\/tests\/v2\/.*\.mjs$/.test(f) : /\.m?js$/.test(f));
}

/** What `file`'s `text` imports against the two rules, each said. */
function strays(file, text) {
  const said = [];
  for (const [, , spec] of text.matchAll(/(?:\bfrom|\bimport)\s*\(?\s*(['"`])([^'"`]+)\1/g)) {
    if (!spec.startsWith('.')) continue;
    const to = posix.join(posix.dirname(file), spec.split('?')[0]);
    const into = /(^|\/)(vendor|ui|v2)\//.test(to.replace(/^web\/tests\//, '')) || to.endsWith('/question.js');
    if (into && !spec.endsWith('?v=__VERSION__')) said.push(`${file} imports ${spec} without ?v=__VERSION__`);
    const own = file === VIEW || file.startsWith('web/vendor/') || file.startsWith('web/tests/');
    if (/(^|\/)vendor\//.test(to) && !own) said.push(`${file} imports ${spec}: only ${VIEW} imports vendor/`);
  }
  return said;
}

test('every import into vendor/, ui/, v2/ or question.js is versioned, and only ui/view.js imports vendor/', () => {
  const files = modules();
  assert.ok(files.includes(VIEW) && files.includes('web/vendor/hooks.js'), files.join(' '));
  assert.deepEqual(files.flatMap((f) => strays(f, readFileSync(`${ROOT}${f}`, 'utf8'))), []);
});

test('a bare ../vendor/preact.js is caught, twice, by name', () => {
  // Written in two pieces, so this file's own scan does not read it.
  const bare = ['import { h } from ', "'../vendor/preact.js';"].join('');
  assert.deepEqual(strays('web/v2/page.js', bare), [
    'web/v2/page.js imports ../vendor/preact.js without ?v=__VERSION__',
    `web/v2/page.js imports ../vendor/preact.js: only ${VIEW} imports vendor/`,
  ]);
  assert.deepEqual(strays(VIEW, "import { h } from '../vendor/preact.js?v=__VERSION__';"), []);
});
