// SPDX-License-Identifier: AGPL-3.0-or-later
// One bounded adapter smoke test on isolated fixture keys; never seeds the
// namespace or prints credentials, invitations, phrases or provider bodies.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash, generateKeyPairSync, randomUUID, sign } from 'node:crypto';
import { createResolver } from '../../../../resolver/handler.mjs';
import { PREFIX } from '../../../../resolver/store.mjs';

const token = execFileSync('security', ['find-generic-password', '-s', 'svrngs-upstash', '-a', 'alexsbryan', '-w'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
const fixture = `${PREFIX}:resolver-proof:${randomUUID()}`;
const { privateKey, publicKey } = generateKeyPairSync('ed25519');
const owner = publicKey.export({ format: 'der', type: 'spki' }).subarray(-32).toString('hex');
const q = { action: 'publish', group: 'synthetic-adapter-proof', id: 'ab'.repeat(32), token: 'synthetic-not-an-invitation', until: Math.floor(Date.now() / 1000) + 300, page: 'https://svrnme.sh' };
const idkey = createHash('sha256').update(`${owner}:${q.id}`).digest('hex');
const endpoint = 'https://true-pheasant-225442.upstash.io';
let guard = 'matching';
let requests = 0;
const keys = new Set();
async function command(parts) {
  const response = await fetch(endpoint, { method: 'POST', headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' }, body: JSON.stringify(parts), signal: AbortSignal.timeout(10000) });
  assert.ok(response.ok, 'provider HTTP refusal');
  const envelope = await response.json();
  assert.ok(!envelope.error, 'provider Redis refusal');
  return envelope.result;
}
const handle = createResolver({ token, fetch: async (url, options) => {
  requests++;
  const parts = JSON.parse(options.body);
  for (let i = 4; i < 8; i++) { parts[i] = parts[i].replace(PREFIX, fixture); keys.add(parts[i]); }
  keys.add(`${fixture}:reservations:secret:${idkey}`);
  if (guard === 'missing') parts[3] = `${fixture}:missing-generation`;
  if (guard === 'mismatch') parts[8] = 'wrong-generation';
  return fetch(url, { ...options, body: JSON.stringify(parts) });
} });
function request(action) {
  const raw = JSON.stringify({ ...q, action, token: action === 'publish' ? q.token : null });
  return new Request('https://join.example/', { method: 'POST', headers: { 'x-svrngs-key': owner, 'x-svrngs-signature': sign(null, Buffer.from(`svrngs-app:onboarding:${raw}`), privateKey).toString('hex') }, body: raw });
}
async function mutate(action) { return handle(request(action), '127.0.0.1'); }
try {
  const first = await mutate('publish');
  assert.equal(first.status, 200, 'live publication');
  const { phrase } = await first.json();
  const lookup = () => handle(new Request(`https://join.example/${phrase}`), '127.0.0.1');
  assert.equal((await (await lookup()).json()).token, q.token);
  assert.equal((await (await mutate('publish')).json()).phrase, phrase);
  assert.equal(await command(['TTL', `${fixture}:reservations`]), -1);
  assert.ok(await command(['TTL', `${fixture}:reservations:secret:${idkey}`]) > 0);
  console.log('live adapter: publish, lookup, idempotent retry, permanent reservation and expiring secret passed');
  for (guard of ['missing', 'mismatch']) {
    for (const action of ['publish', 'status', 'close']) assert.equal((await mutate(action)).status, 503);
    assert.equal((await lookup()).status, 503);
  }
  guard = 'matching';
  console.log('live adapter: missing/mismatched generation refused all four operations');
  assert.equal((await (await mutate('close')).json()).closed, true);
  assert.equal((await (await lookup()).json()).state, 'ended');
  assert.equal((await mutate('publish')).status, 503);
  assert.equal(await command(['EXISTS', `${fixture}:reservations:secret:${idkey}`]), 0);
  assert.equal(await command(['TTL', `${fixture}:reservations`]), -1);
  console.log('live adapter: close, ended guidance, replay refusal and secret deletion passed');
  console.log(`live adapter requests: ${requests}; no deployment or quota exhaustion`);
} catch {
  console.error('live adapter proof failed; no provider body or credential logged');
  process.exitCode = 1;
} finally {
  await command(['DEL', ...keys]);
  console.log('isolated fixture keys cleaned; real generation retained');
}
