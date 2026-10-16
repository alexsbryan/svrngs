// Grapevine's ground and its answers (GRAPEVINE_DEMO §2-§4, §7), one rule a
// test. The laws of §8 are laws.test.mjs's.

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import { ACT_MOST, BODY_MOST, keyPair, open, seal } from './seal.mjs';

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

/** Every string `body` holds, at any depth. */
const words = (v) => (typeof v === 'string' ? [v] : v && typeof v === 'object' ? Object.values(v).flatMap(words) : []);

test('a sealed message opens at either end, and nowhere else', () => {
  const [sam, kim, jo] = [keyPair(), keyPair(), keyPair()];
  const body = { say: 'offer', fields: { when: 'Saturday morning', where: 'Waller and Pierce' } };
  const sealed = seal(body, sam.public, kim.secret);
  assert.deepEqual(open(sealed, kim.public, sam.secret), body);
  assert.deepEqual(open(sealed, sam.public, kim.secret), body, 'the sender reads back what they sealed');
  assert.equal(open(sealed, kim.public, jo.secret), null, 'a carrier holding every byte cannot open it');
  const box = Buffer.from(sealed.box, 'base64');
  box[3] ^= 1;
  assert.equal(open({ ...sealed, box: box.toString('base64') }, kim.public, sam.secret), null, 'one byte changed does not open');
});

test('a sealed message holds none of its words', () => {
  const [sam, kim] = [keyPair(), keyPair()];
  const body = { say: 'offer', fields: { when: 'Saturday morning', where: 'Waller and Pierce' } };
  const wire = JSON.stringify(seal(body, sam.public, kim.secret));
  for (const w of [...words(body), JSON.stringify(body), Buffer.from(JSON.stringify(body)).toString('base64')]) {
    assert.ok(!wire.includes(w), `the sealed message holds ${JSON.stringify(w)}`);
  }
});

test("the most an act may be is svrngs-core's, and the largest sealed message is under it", () => {
  const core = readFileSync(join(HERE, '../../crates/svrngs-core/src/payload.rs'), 'utf8');
  const [, a, b] = core.match(/pub const MAX_PAYLOAD_BYTES: usize = (\d+) \* (\d+);/) ?? [];
  assert.equal(ACT_MOST, Number(a) * Number(b), 'ACT_MOST is payload.rs\'s MAX_PAYLOAD_BYTES');
  const [sam, kim] = [keyPair(), keyPair()];
  const most = { say: 'text', text: 'x'.repeat(BODY_MOST - JSON.stringify({ say: 'text', text: '' }).length) };
  assert.equal(JSON.stringify(most).length, BODY_MOST);
  const msg = { app: 'grapevine', type: 'msg', handle: 'A'.repeat(22) + '==', dir: 'down', from: kim.public, to: sam.public, ...seal(most, sam.public, kim.secret) };
  assert.ok(Buffer.byteLength(JSON.stringify(msg)) <= ACT_MOST, `a msg sealing ${BODY_MOST} bytes is ${Buffer.byteLength(JSON.stringify(msg))}`);
  assert.throws(() => seal({ ...most, text: `${most.text}x` }, sam.public, kim.secret), new RegExp(`at most ${BODY_MOST} bytes sealed, and this one is ${BODY_MOST + 1}`));
});
