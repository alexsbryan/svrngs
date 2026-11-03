import assert from 'node:assert/strict';
import { createHmac, webcrypto } from 'node:crypto';
import { test } from 'node:test';
import { proof, Browser, browserAuthority } from '../../crates/svrngs-node/src/home/session.js';

test('browser proof covers port, method, raw path, body digest, intent and boot challenge', async () => {
  globalThis.crypto ??= webcrypto;
  const bytes = new Uint8Array(32).fill(7);
  const key = await crypto.subtle.importKey(
    'raw',
    bytes,
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  );
  const request = new Request('http://svrngs.localhost:4321/group?x=1', {
    method: 'POST',
    headers: { 'x-svrngs-body': 'digest', 'x-svrngs-intent': 'yes' },
    body: '{}',
  });
  const signed = await proof(key, request, 'boot-challenge');
  const [stamp, nonce, tag] = signed.split('.');
  const message = `svrngs.localhost:4321\nPOST\n/group?x=1\n${stamp}.${nonce}\ndigest\nyes\nboot-challenge`;
  assert.equal(
    Buffer.from(tag, 'base64url').toString('hex'),
    createHmac('sha256', bytes).update(message).digest('hex'),
  );
  assert.equal(await request.text(), '{}', 'making the proof never consumes the request body');
});

test('the authentication worker never lends its key to a foreign frame or form', async () => {
  const origin = 'http://svrngs.localhost:4321';
  const clients = {
    get: async (id) => ({ url: id === 'own' ? `${origin}/` : 'https://elsewhere.example/' }),
  };
  const event = (id, mode, destination, headers = {}, method = 'GET') => ({
    clientId: id,
    request: { mode, destination, method, headers: new Headers(headers) },
  });
  assert.equal(await browserAuthority(event('own', 'cors', ''), clients, origin), Browser.Own);
  assert.equal(
    await browserAuthority(
      event('', 'navigate', 'document', { 'sec-fetch-site': 'none' }),
      clients,
      origin,
    ),
    Browser.Tab,
  );
  // What a worker is shown of a typed or bookmarked address: no Sec-Fetch-* at all.
  assert.equal(
    await browserAuthority(event('', 'navigate', 'document'), clients, origin),
    Browser.Tab,
  );
  assert.equal(
    await browserAuthority(event('foreign', 'navigate', 'iframe'), clients, origin),
    null,
  );
  assert.equal(
    await browserAuthority(event('foreign', 'navigate', 'document', {}, 'POST'), clients, origin),
    null,
  );
  assert.equal(await browserAuthority(event('foreign', 'cors', ''), clients, origin), null);
  const anotherPort = { get: async () => ({ url: 'http://svrngs.localhost:4322/' }) };
  assert.equal(await browserAuthority(event('foreign', 'cors', ''), anotherPort, origin), null);
  assert.equal(
    await browserAuthority(event('own', 'cors', '', { origin: 'null' }), clients, origin),
    null,
  );
});

// WebKit shows a service worker neither a navigation's `Origin` nor the
// client that started it; its `referrer`, which the browser sets to the
// page that started it and no page sets to another origin, is all it has
// (measured 2026-11-02, scratchpad probe2/navseen.txt: WebKit `own-post`
// and `cross-post`). Red before: the own-origin form post was `null`,
// refused "Open this page as a tab, or from its own origin".
test('a navigation the worker is shown no Origin of is its own page’s only by its referrer', async () => {
  const origin = 'http://app.localhost:4321';
  const clients = { get: async () => undefined };
  const navigation = (method, referrer, destination = 'document') => ({
    clientId: '',
    request: { mode: 'navigate', destination, method, referrer, headers: new Headers() },
  });
  assert.equal(
    await browserAuthority(navigation('POST', `${origin}/form`), clients, origin),
    Browser.Own,
  );
  assert.equal(
    await browserAuthority(navigation('GET', `${origin}/`, 'iframe'), clients, origin),
    Browser.Own,
  );
  assert.equal(
    await browserAuthority(navigation('POST', 'http://other.localhost:4321/'), clients, origin),
    null,
  );
  assert.equal(
    await browserAuthority(navigation('POST', 'http://app.localhost:4322/'), clients, origin),
    null,
    'another port is another origin',
  );
  assert.equal(
    await browserAuthority(navigation('POST', ''), clients, origin),
    null,
    'a withheld referrer proves nothing',
  );
  assert.equal(await browserAuthority(navigation('POST', 'about:client'), clients, origin), null);
  assert.equal(
    await browserAuthority(navigation('GET', 'http://other.localhost:4321/'), clients, origin),
    Browser.Tab,
  );
  // A fetch, not a navigation: its client decides, never its referrer.
  const fetched = {
    clientId: '',
    request: {
      mode: 'cors',
      destination: '',
      method: 'POST',
      referrer: `${origin}/`,
      headers: new Headers(),
    },
  };
  assert.equal(await browserAuthority(fetched, clients, origin), null);
});
