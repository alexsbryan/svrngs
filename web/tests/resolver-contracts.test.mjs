// Isolated cases extracted from the Redis suite. The real handler/signature code
// runs; unavailable providers and rejected ingress require no persistence process.
import assert from 'node:assert/strict';
import { sign } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { createResolver, phrase } from '../../resolver/handler.mjs';
import { ownerKey, owner, hash, fixed, body, mutation, refused } from './resolver-fixture.mjs';

test('phrase source, space and Rust signing-key vector are pinned', () => {
  const words = readFileSync(new URL('../../resolver/words.txt', import.meta.url), 'utf8')
    .trim()
    .split(/\s+/);
  assert.equal(words.length, 1295);
  assert.equal(new Set(words).size, 1295);
  assert.equal(
    hash(words.join('\n')),
    'd2ee5adec14a5e9a63ab145e169bbb052091c4d8b5fae9d36a55099f76cf3072',
  );
  assert.equal(owner, '8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c');
  assert.equal(
    sign(null, Buffer.from('svrngs-app:notes:line 812'), ownerKey).toString('hex'),
    '6296480dc80a73a3f3872dac92fb777835e837829b20c7f818ab7ea922abdf401452e404d04f7a39cf613161bdb80a89ba55bfc43700b0a678f63b5c4110440f',
  );
  for (let i = 0; i < 100; i++)
    assert.ok(
      phrase()
        .split('-')
        .every((word) => words.includes(word)),
    );
});

test('provider failures are unavailable with one attempt and no false ended', async () => {
  for (const response of [
    () => new Response('', { status: 429 }),
    () => new Response('bandwidth', { status: 509 }),
    () => new Response('', { status: 500 }),
    () => new Response('', { status: 401 }),
    () => Response.json({ error: 'ERR max requests limit exceeded' }),
    () => Response.json({ error: 'daily command quota exhausted' }),
    () => Response.json({ error: 'bandwidth quota exhausted' }),
    () => Response.json({ error: 'OOM storage exhausted' }),
    () => Response.json({ result: null }),
    () => new Response('{'),
    () => new Response('x'.repeat(8193)),
    () => {
      throw new Error('TLS/DNS/timeout');
    },
  ]) {
    let attempts = 0;
    const failed = createResolver({
      token: 'fixture',
      fetch: async () => {
        attempts++;
        return response();
      },
    });
    for (const request of [
      new Request(`https://join.example/${fixed}`),
      mutation(body()),
      mutation(body({ action: 'close', token: null })),
    ])
      await refused(await failed(request, '127.0.0.1'));
    assert.equal(attempts, 3, 'one store attempt per explicit request');
  }
});

function untouched() {
  let calls = 0;
  return {
    handle: createResolver({
      token: 'fixture',
      fetch: async () => {
        calls++;
        throw new Error('rejected ingress reached the store');
      },
    }),
    calls: () => calls,
  };
}

test('ingress bounds, authentication, trusted IP and no public directory', async () => {
  const store = untouched();
  for (const request of [
    mutation('x'.repeat(8193)),
    mutation(JSON.stringify(body()) + ' '.repeat(8193)),
    mutation(body(), ownerKey, 'wrong-domain'),
    new Request('https://join.example/'),
    new Request('https://join.example/list'),
    new Request('https://join.example/', { method: 'POST', body: JSON.stringify(body()) }),
    mutation({ ...body(), unexpected: 'field' }),
    new Request(mutation(body()), {
      body: JSON.stringify(body({ token: 'tampered-after-signing' })),
    }),
  ])
    await refused(await store.handle(request, '127.0.0.1'));
  await refused(await store.handle(mutation(body()), undefined));
  assert.equal(store.calls(), 0);
});

test('a UTF-8 BOM cannot change the bytes whose signature is checked', async () => {
  const store = untouched();
  const q = body();
  const signed = mutation(q);
  await refused(
    await store.handle(new Request(signed, { body: `\ufeff${JSON.stringify(q)}` }), '127.0.0.1'),
  );
  assert.equal(store.calls(), 0);
});

test('resolver refuses provider failure with retry guidance, never ended', async () => {
  const handle = createResolver({
    token: 'fixture',
    fetch: async () => new Response('', { status: 429 }),
  });
  const answer = await handle(
    new Request('https://join.example/acid-acid-acid-acid-acid'),
    '127.0.0.1',
  );
  assert.equal(answer.status, 503);
  assert.equal(answer.headers.get('retry-after'), '60');
  assert.match(await answer.text(), /joining is temporarily unavailable/);
});

test('whole-response timeout cancels stalled streams and cannot later publish', async (t) => {
  // node:test's mocked setTimeout holds the handler's 8 s bound, so the
  // bound is an order (not before 8000 ms, at it), never a wait.
  t.mock.timers.enable({ apis: ['setTimeout'] });
  let canceled = false;
  let attempts = 0;
  let asked;
  const reached = new Promise((resolve) => (asked = resolve));
  const stalled = createResolver({
    token: 'fixture',
    fetch: async () => {
      attempts++;
      asked();
      return new Response(
        new ReadableStream({
          cancel() {
            canceled = true;
          },
        }),
      );
    },
  });
  let settled = false;
  const answer = stalled(new Request(`https://join.example/${fixed}`), '127.0.0.1').finally(
    () => (settled = true),
  );
  await reached;
  await new Promise((resolve) => setImmediate(resolve));
  t.mock.timers.tick(7999);
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(settled, false, 'the whole-response bound is 8000 ms, not less');
  t.mock.timers.tick(1);
  await refused(await answer);
  assert.equal(attempts, 1);
  assert.equal(canceled, true);
});
