// Grapevine's ground and its answers (GRAPEVINE_DEMO §2-§4, §7), one rule a
// test. The laws of §8 are laws.test.mjs's.

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

test('tweetnacl is the copy npm published, every file as pinned', () => {
  assert.equal(pins().length, 3, 'vendor/README.md pins nacl-fast.js, package.json and LICENSE');
  assert.deepEqual(unpinned(join(HERE, 'vendor')), []);
});

test('one byte changed in the vendored nacl-fast.js is caught', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-vendor-'));
  cpSync(join(HERE, 'vendor'), dir, { recursive: true });
  const f = join(dir, 'tweetnacl/nacl-fast.js');
  const bytes = readFileSync(f);
  bytes[1000] ^= 1;
  writeFileSync(f, bytes);
  assert.deepEqual(unpinned(dir).map((s) => s.split(' ')[0]), ['tweetnacl/nacl-fast.js']);
});
