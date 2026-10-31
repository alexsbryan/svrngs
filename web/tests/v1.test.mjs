// v1 of both pages, frozen until page-switchover (page-scaffold (g);
// ralph/next/page/CHARTER.md, "What is settled" 3): each file the live
// pages are built from, held to its sha256 at the queue's base, 3df32f1f.
// v2 is built beside these, never in them. A fix the gate needs on v1 is
// the one exception: its pin changes in the same commit, which says why,
// and the same fix lands in v2 in that row (the queue's fence 2). Any
// other change here is the operator's.

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { test } from 'node:test';

const ROOT = new URL('../..', import.meta.url).pathname;
const HOME = 'crates/svrngs-node/src/home';
const V1 = [
  ['web/index.html', '1abbd99d8108aa5e0cd3c0bf122c458a0d5c91850e4cae207b74a522da3f2225'],
  ['web/app.js', 'ab009db671f3e7622d85c3055f22eb4cb359c6ae84f532eca69caaf0bc053e67'],
  ['web/sw.js', '7f375c3b5759b8b283393b42ebfbe9f25082552ef3b62a5da0a7344552039364'],
  ['web/store.js', '21d0a9366aac181d0299c4e7d1b389fdeed06d7469292b7f776e25c70426d5c6'],
  ['web/open.html', 'a46d24099b1e3f231e115c2a76ce7aa953a2613fdfdb06186ce43e9dc1aaaeac'],
  ['web/open.js', '0d23fde638d1c476ce02c4c981527d2e4c75a04c01dcedc8ebd9ea62dd7df947'],
  ['web/lend.js', '9113523ec2589956b488f6a67b40188c926dba38882a313de6926448409a2d15'],
  ['web/shim.js', '5c1859df3c56d8efffae64cf7bcc62a630a49754a2f812e19a204ac3ac8b45a5'],
  [`${HOME}/index.html`, '1199619683241f1cb45f512c89f4d783b315ed1aa67626f0db6dc652cb59ac3b'],
  [`${HOME}/home.js`, '3e8513bc51d464b292f437d941ccfc010b4778cdcb030adab412405e2d762b76'],
  [`${HOME}/question.js`, '7ee9b4c76eeebc164679a17765a25348843002aa2aa15f45f2f4706e92271b59'],
  [`${HOME}/look.css`, '5d621798ab91d5cbfd5506fa0d876a0f2fe1fdf1208f66662622f6b24de98013'],
  [`${HOME}/away.html`, '6b53379435b2b6210b5921a37b975acb0f18da28deb022393540a25bcef0f330'],
];

/** Each v1 file under `root` whose bytes are not the ones pinned, said. */
function moved(root) {
  return V1.map(([f, want]) => [f, createHash('sha256').update(readFileSync(join(root, f))).digest('hex'), want])
    .filter(([, got, want]) => got !== want)
    .map(([f, got, want]) => `${f} is ${got}, pinned ${want}: v1 is frozen until page-switchover`);
}

test('every v1 page file is the one pinned', () => {
  assert.deepEqual(moved(ROOT), []);
});

test('one byte changed in web/app.js is caught, by name', () => {
  const dir = mkdtempSync(join(tmpdir(), 'svrngs-v1-'));
  for (const [f] of V1) {
    mkdirSync(join(dir, dirname(f)), { recursive: true });
    cpSync(join(ROOT, f), join(dir, f));
  }
  const bytes = readFileSync(join(dir, 'web/app.js'));
  bytes[100] ^= 1;
  writeFileSync(join(dir, 'web/app.js'), bytes);
  assert.deepEqual(moved(dir).map((s) => s.split(' ')[0]), ['web/app.js']);
});
