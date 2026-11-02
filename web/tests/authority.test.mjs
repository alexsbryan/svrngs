import assert from 'node:assert/strict';
import { test } from 'node:test';
import { maySign, signerFor, sealOpen, openSeal, hostCookie, handoffCookie, holderMatches } from '../authority.js';
import { webcrypto } from 'node:crypto';

test('a write is caused by a page of this origin, or a navigation one of them began', async () => {
  const origin = 'https://app.example';
  const event = (id, mode = 'cors', originHeader, referrer = '') => ({ clientId: id, request: { mode, referrer, headers: new Headers(originHeader ? { origin: originHeader } : {}) } });
  const clients = { get: async (id) => ({ ours: { url: `${origin}/page` }, elsewhere: { url: 'https://elsewhere.example/' } })[id] };
  assert.equal(await maySign(event('ours'), clients, origin), true);
  assert.equal(await maySign(event('elsewhere'), clients, origin), false);
  assert.equal(await maySign(event('missing'), clients, origin), false);
  assert.equal(await maySign(event(''), clients, origin), false);
  assert.equal(await maySign(event('ours', 'cors', 'https://elsewhere.example'), clients, origin), false);
  assert.equal(await maySign(event('', 'navigate', undefined, `${origin}/g/e`), clients, origin), true, 'a form this origin posted');
  assert.equal(await maySign(event('', 'navigate', undefined, 'https://elsewhere.example/'), clients, origin), false, 'another site’s form');
  assert.equal(await maySign(event('', 'navigate'), clients, origin), false, 'no referrer, no write');
});

test('the only extra frame remains at the platform parent origin', () => {
  assert.equal(signerFor('https://app.group.example/'), 'https://group.example/sign.html');
  assert.equal(signerFor('http://app.localhost:4321/base/', '/base/'), 'http://localhost:4321/base/sign.html');
  assert.equal(new URL(signerFor('https://app.group.example/', '//elsewhere.example/')).origin, 'https://group.example');
  assert.throws(() => signerFor('https://localhost/'));
});

test('an open’s approver opens only with its own W, for the open it was sealed to', async () => {
  globalThis.crypto ??= webcrypto;
  const seed = crypto.getRandomValues(new Uint8Array(32));
  const h = { label: 'app', holder: 'holder', group: 'house', writes: 'write-grant', lends: [{ label: 'app', key: 'real-peer' }], pass: 'signed-pass', page: 'https://group.example/', raw: false };
  const { proof, w } = await sealOpen(h, seed);
  h.proof = proof;
  assert.deepEqual(await openSeal(h, w), seed);
  const other = (await sealOpen(h, seed)).w;
  assert.equal(await openSeal(h, other), null, 'another open’s W opens nothing');
  for (const changed of [{ lends: [{ label: 'app', key: 'wrong-peer' }] }, { page: 'https://elsewhere.example/' }, { pass: 'another-pass' }, { writes: 'another-grant' }, { group: 'another-group' }, { holder: 'another-holder' }, { label: 'another-app' }]) {
    assert.equal(await openSeal({ ...h, ...changed }, w), null, `a changed ${Object.keys(changed)[0]} opens nothing`);
  }
  assert.deepEqual(await openSeal({ ...h, key: { opaque: true }, line: { seq: 1 }, seed: 'not-authenticated-as-metadata' }, w), seed);
  assert.equal(await openSeal(h, null), null, 'no cookie, nothing');
  assert.equal(await openSeal({ ...h, proof: 'zz' }, w), null);
});

test('the signer reads only the one __Host- cookie named for its app', () => {
  const name = handoffCookie('app');
  assert.equal(name, '__Host-svrngs-app');
  assert.equal(hostCookie(`a=1; ${name}=w1; b=2`, name), 'w1');
  assert.equal(hostCookie('svrngs-app=planted; tossed=evil', name), null, 'a cookie an app could plant is not read');
  assert.equal(hostCookie(`${name}-other=w2`, name), null);
  assert.equal(hostCookie(`${name}=w1; ${name}=w2`, name), null, 'two of one name are none');
  assert.equal(hostCookie('', name), null);
});

test('a private seed must prove the holder named by the trusted handoff', async () => {
  globalThis.crypto ??= webcrypto;
  const pair = await crypto.subtle.generateKey('Ed25519', false, ['sign', 'verify']);
  const other = await crypto.subtle.generateKey('Ed25519', false, ['sign', 'verify']);
  const holder = Buffer.from(await crypto.subtle.exportKey('raw', pair.publicKey)).toString('hex');
  assert.equal(await holderMatches(pair.privateKey, holder), true);
  assert.equal(await holderMatches(other.privateKey, holder), false);
});
