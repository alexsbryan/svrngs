// canon's build, vendored (row canon-spike (c)): the files are the ones
// apps/canon/build.sh proved against canon-cli, byte for byte, and the build
// loads and folds under node with no WebAssembly.

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

const HERE = new URL('.', import.meta.url).pathname;

/** The vendored files' sha256s, as vendor/README.md pins them: [[file, sha256]]. */
function pins() {
  const readme = readFileSync(join(HERE, 'vendor/README.md'), 'utf8');
  return [...readme.matchAll(/^\| `([^`]+)` \| `([0-9a-f]{64})` \|$/gm)].map((m) => [m[1], m[2]]);
}

/** Each vendored file under `dir` whose bytes are not the ones pinned, said. */
function unpinned(dir) {
  return pins()
    .map(([f, want]) => [f, createHash('sha256').update(readFileSync(join(dir, f))).digest('hex'), want])
    .filter(([, got, want]) => got !== want)
    .map(([f, got, want]) => `${f} is ${got}, pinned ${want}`);
}

test('the vendored build is the one proven, every file as pinned', () => {
  assert.equal(pins().length, 3, 'vendor/README.md pins canon.js, call.mjs and LICENSE');
  assert.deepEqual(unpinned(join(HERE, 'vendor')), []);
});

test('one byte changed in the vendored canon.js is caught', () => {
  const dir = mkdtempSync(join(tmpdir(), 'canon-vendor-'));
  cpSync(join(HERE, 'vendor'), dir, { recursive: true });
  const f = join(dir, 'canon.js');
  const bytes = readFileSync(f);
  bytes[1000] ^= 1;
  writeFileSync(f, bytes);
  assert.deepEqual(unpinned(dir).map((s) => s.split(' ')[0]), ['canon.js']);
});

test('the vendored build mints a rule and folds it, as canon', async () => {
  const { bind } = await import('./vendor/call.mjs');
  const call = bind(await import('./vendor/canon.js'));
  const line = call({
    call: 'act',
    kind: { op: 'assert', text: 'Quiet hours are 10pm to 7am.' },
    ts_unix: 1_792_000_000,
    actor: 'human:kim',
  });
  const act = JSON.parse(line);
  assert.match(act.id, /^can-[0-9a-f]{12}$/);
  const canon = call({ call: 'derive', acts: line });
  assert.deepEqual(
    canon.commitments.map((c) => [c.id, c.text, c.actor, c.status.status]),
    [[act.id, 'Quiet hours are 10pm to 7am.', 'human:kim', 'active']],
  );
  assert.throws(() => call({ call: 'derive', acts: '{"v":99}' }), /canon: /, 'a refusal is said, never defaulted');
});
