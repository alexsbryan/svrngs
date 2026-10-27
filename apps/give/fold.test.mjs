// Give's fold, the page's build (give-app-keep (a)): every case in
// fold/cases.json, the file fold/tests/cases.rs runs against the keeper
// program's build, answered by the JavaScript the page runs, so the two
// builds are held to one set of answers (AGENTS.md 8). And the vendored
// build is the one built from the fold's source now: page/vendor/README.md
// pins the build and the source it came from, so a change to the fold that
// is not built again fails here, not on a page.
//
// `node --test` runs it against page/vendor/fold.js; apps/give/build.sh runs
// it as `node fold.test.mjs --wasm <file> --js <file>`, against a fresh
// build, before vendoring it.

import assert from 'node:assert/strict';
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { pathToFileURL } from 'node:url';

import { pins, unpinned } from '../pinned.mjs';
import { bind } from './page/vendor/call.mjs';

const HERE = new URL('.', import.meta.url).pathname;
const VENDOR = join(HERE, 'page/vendor');
const CASES = JSON.parse(readFileSync(join(HERE, 'fold/cases.json'), 'utf8'));
/** As fold/tests/cases.rs counts them: a case either build skipped would show. */
const COUNT = 41;

const flag = (name) => {
  const at = process.argv.indexOf(name);
  return at < 0 ? null : process.argv[at + 1];
};

/** Each build to prove, by name: the vendored JavaScript, or those build.sh names. */
async function builds() {
  const out = [];
  const js = flag('--js');
  out.push(['JavaScript', bind(await import(pathToFileURL(js ?? join(VENDOR, 'fold.js')).href))]);
  const wasm = flag('--wasm');
  if (wasm) {
    const { instance } = await WebAssembly.instantiate(readFileSync(wasm), {});
    out.push(['wasm', bind(instance.exports)]);
  }
  return out;
}

for (const [name, fold] of await builds()) {
  test(`every case folds as written, in the ${name} build`, () => {
    assert.equal(CASES.length, COUNT, `fold/cases.json holds ${COUNT} cases`);
    const failed = CASES.filter((c) => JSON.stringify(sorted(fold(c.ops, c.now))) !== JSON.stringify(sorted(c.kept)));
    assert.deepEqual(
      failed.map((c) => `${c.name}\n  wanted ${JSON.stringify(c.kept)}\n  folded ${JSON.stringify(fold(c.ops, c.now))}`),
      [],
    );
  });

  test(`a request that is not ops is said so, in the ${name} build`, () => {
    assert.throws(() => fold(undefined), /give's fold: a request is \{"ops": \[\.\.\.\]\}/);
  });
}

if (!flag('--js')) {
  test('the vendored build is the one built from the fold as it is now', () => {
    const pinned = pins(VENDOR).map(([f]) => f);
    assert.deepEqual(pinned, ['fold.js', 'call.mjs', '../../fold/src/lib.rs', '../../fold/Cargo.toml']);
    assert.deepEqual(unpinned(VENDOR), [], 'run apps/give/build.sh --vendor, and pin what it prints');
  });

  test('one byte changed in the fold since its build is caught', () => {
    const dir = mkdtempSync(join(tmpdir(), 'give-fold-'));
    cpSync(join(HERE, 'page'), join(dir, 'page'), { recursive: true });
    cpSync(join(HERE, 'fold'), join(dir, 'fold'), { recursive: true });
    const f = join(dir, 'fold/src/lib.rs');
    writeFileSync(f, readFileSync(f, 'utf8').replace('"stale"', '"stalE"'));
    assert.deepEqual(unpinned(VENDOR, join(dir, 'page/vendor')).map((s) => s.split(' ')[0]), ['../../fold/src/lib.rs']);
  });
}

/** `v` with every object's keys in order, so two answers compare by what they say. */
function sorted(v) {
  if (Array.isArray(v)) return v.map(sorted);
  if (v && typeof v === 'object') return Object.fromEntries(Object.keys(v).sort().map((k) => [k, sorted(v[k])]));
  return v;
}
