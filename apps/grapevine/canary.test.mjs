// The canary under `node --test`, so scripts/check.sh runs it (gv-page (d)):
// green on demo.json as the app is, and red on a copy of the app whose fold
// never says to correct a carry whose ask was withdrawn, the canary saying
// the step that broke.

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

const HERE = new URL('.', import.meta.url).pathname;
const run = (dir) => spawnSync(process.execPath, [join(dir, 'canary.mjs')], { encoding: 'utf8' });

test("the canary runs §10's scene on demo.json, each step timed", () => {
  const before = readFileSync(join(HERE, 'demo.json'), 'utf8');
  const out = run(HERE);
  assert.equal(out.status, 0, out.stderr);
  assert.match(out.stdout, /6\. +\d+\.\d{3} s  Sam takes it back; within one turn the carry in Waller is gone/);
  assert.match(out.stdout, /\d+ reads of a group's log, .* median \d+\.\d+ ms, slowest \d+\.\d+ ms/);
  assert.equal(readFileSync(join(HERE, 'demo.json'), 'utf8'), before, 'demo.json itself is never written');
});

test('the canary fails on a carry left uncorrected after withdrawal', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-plant-'));
  cpSync(HERE, dir, { recursive: true });
  const from = 'if (upstream?.voided && carried) relay.push(';
  const src = readFileSync(join(dir, 'grapevine.js'), 'utf8');
  assert.equal(src.split(from).length, 2, `the plant no longer applies to grapevine.js: ${from}`);
  writeFileSync(join(dir, 'grapevine.js'), src.replace(from, 'if (false) relay.push('));
  const out = run(dir);
  assert.equal(out.status, 1, out.stdout);
  assert.match(out.stderr, /step 6, Sam takes it back; within one turn the carry in Waller is gone, did not hold:\nKim still sees \["tile saw, Saturday"\]; Jo's carry corrected: false/);
});
