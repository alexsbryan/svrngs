// scripts/deploy-page.sh publishes a page only with its release's signature
// (DESIGN §7, "A computer updates only from a signed release"): `--site
// <dir>` checks get/SHA256SUMS.sig against svrngs-release-key.asc in a
// keyring of its own and refuses to deploy without a valid one, and with no
// argument it refuses, naming the three steps. `vercel` is a stand-in on
// PATH that keeps what it was asked, so nothing is deployed. The signed site
// is v1.11.2's own sums and signature (crates/svrngs-node/tests/fixtures/).
//
// Planted, watched failing: the signature's check removed from
// deploy-page.sh (the unsigned site and the changed one deployed).

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

const ROOT = new URL('../..', import.meta.url).pathname;
const SIGNED = join(ROOT, 'crates/svrngs-node/tests/fixtures/v1.11.2');
const SUMS = readFileSync(join(SIGNED, 'SHA256SUMS'));
const SIG = readFileSync(join(SIGNED, 'SHA256SUMS.sig'));

// A site of `sums` and, if given, `sig`, and deploy-page.sh run over it
// with `vercel` a stand-in: its answer, and what vercel was asked.
function deploy(args, sums, sig) {
  const at = mkdtempSync(join(tmpdir(), 'deploy-page-'));
  mkdirSync(join(at, 'bin'));
  mkdirSync(join(at, 'site/get'), { recursive: true });
  const asked = join(at, 'asked');
  writeFileSync(join(at, 'bin/vercel'), `#!/bin/sh\necho "$*" >>"${asked}"\n`);
  chmodSync(join(at, 'bin/vercel'), 0o755);
  if (sums) writeFileSync(join(at, 'site/get/SHA256SUMS'), sums);
  if (sig) writeFileSync(join(at, 'site/get/SHA256SUMS.sig'), sig);
  const site = join(at, 'site');
  const said = spawnSync(join(ROOT, 'scripts/deploy-page.sh'), args(site), {
    env: { ...process.env, PATH: `${join(at, 'bin')}:${process.env.PATH}` },
    encoding: 'utf8',
  });
  const vercel = existsSync(asked) ? readFileSync(asked, 'utf8').trim().split('\n') : [];
  rmSync(at, { recursive: true, force: true });
  return { status: said.status, stderr: said.stderr, vercel, site };
}

test('an unsigned site is refused, and nothing deployed', () => {
  const said = deploy((site) => ['--site', site], SUMS);
  assert.equal(said.status, 1, said.stderr);
  assert.match(said.stderr, /has no valid signature by the release key/);
  assert.match(said.stderr, /scripts\/sign-release\.sh/);
  assert.deepEqual(said.vercel, []);
});

test('sums changed after they were signed are refused', () => {
  const said = deploy((site) => ['--site', site], Buffer.concat([SUMS, Buffer.from('\n')]), SIG);
  assert.equal(said.status, 1, said.stderr);
  assert.deepEqual(said.vercel, []);
});

test('a site the release key signed is deployed as before', () => {
  const said = deploy((site) => ['--site', site, '--project', 'p', '--scope', 's'], SUMS, SIG);
  assert.equal(said.status, 0, said.stderr);
  assert.deepEqual(said.vercel, [
    `link --yes --project p --scope s --cwd ${said.site}`,
    `deploy --prod --yes --scope s --cwd ${said.site}`,
  ]);
});

test('no argument refuses, naming the three steps', () => {
  const said = deploy(() => []);
  assert.equal(said.status, 2, said.stderr);
  for (const step of [
    'scripts/deploy-page.sh --build',
    'scripts/sign-release.sh',
    'scripts/deploy-page.sh --site',
  ]) {
    assert.ok(said.stderr.includes(step), step);
  }
  assert.deepEqual(said.vercel, []);
});
