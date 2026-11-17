// Isolated cases extracted from the Redis suite. The real handler/signature code
// runs; unavailable providers and rejected ingress require no persistence process.
import assert from 'node:assert/strict';
import { sign } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { createResolver, phrase, read, SPACE, words } from '../../resolver/handler.mjs';
import { GENERATION, PREFIX, SCRIPT } from '../../resolver/store.mjs';
import { ownerKey, owner, hash, fixed, old, body, mutation, refused } from './resolver-fixture.mjs';

// Levenshtein distance by the textbook table, an instrument apart from
// handler.mjs's `near`, which the pruned list is held against.
function distance(a, b) {
  let row = Array.from({ length: b.length + 1 }, (_, j) => j);
  for (let i = 1; i <= a.length; i++) {
    const next = [i];
    for (let j = 1; j <= b.length; j++)
      next[j] = Math.min(row[j] + 1, next[j - 1] + 1, row[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
    row = next;
  }
  return row[b.length];
}

test('phrase source, pruned list, space and Rust signing-key vector are pinned', () => {
  const eff = readFileSync(new URL('../../resolver/words.txt', import.meta.url), 'utf8')
    .trim()
    .split(/\s+/);
  assert.equal(eff.length, 1295);
  assert.equal(new Set(eff).size, 1295);
  assert.equal(
    hash(eff.join('\n')),
    'd2ee5adec14a5e9a63ab145e169bbb052091c4d8b5fae9d36a55099f76cf3072',
  );
  // Shortest first, then alphabetically: each word kept is two edits or more
  // from every word kept, and each word dropped is one edit from a word kept
  // before it.
  const order = eff.toSorted((a, b) => a.length - b.length || (a < b ? -1 : 1));
  assert.deepEqual(
    words,
    order.filter((w) => words.includes(w)),
  );
  assert.equal(words.length, 818);
  for (const [i, a] of words.entries())
    for (const b of words.slice(i + 1)) assert.ok(distance(a, b) >= 2, `${a} ${b}`);
  for (const w of order.filter((w) => !words.includes(w))) {
    const before = words.filter((k) => order.indexOf(k) < order.indexOf(w));
    assert.ok(
      before.some((k) => distance(k, w) === 1),
      `${w} was dropped but no earlier word is near it`,
    );
  }
  assert.equal(SPACE, 818 ** 3 * 10 ** 4);
  assert.equal(Math.log2(SPACE).toFixed(1), '42.3');
  assert.equal(owner, '8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c');
  assert.equal(
    sign(null, Buffer.from('svrngs-app:notes:line 812'), ownerKey).toString('hex'),
    '6296480dc80a73a3f3872dac92fb777835e837829b20c7f818ab7ea922abdf401452e404d04f7a39cf613161bdb80a89ba55bfc43700b0a678f63b5c4110440f',
  );
  for (let i = 0; i < 1000; i++) {
    const drawn = phrase();
    const [, ...parts] = drawn.match(/^([a-z]+)-([a-z]+)-([a-z]+)-([0-9]{4})$/);
    assert.ok(
      parts.slice(0, 3).every((word) => words.includes(word)),
      drawn,
    );
    assert.equal(read(drawn).phrase, drawn);
  }
});

// A store that counts what reached it and answers one result.
function store(result = { state: 'ended', phrase: '', secret: false }) {
  const asked = [];
  return {
    handle: createResolver({
      token: 'fixture',
      fetch: async (_url, options) => {
        asked.push(JSON.parse(JSON.parse(options.body).at(-1)).phrase);
        return Response.json({ result: JSON.stringify(result) });
      },
    }),
    asked,
  };
}
const html = { accept: 'text/html' };

test('an address is read forgivingly, before any store call, as one canonical phrase', async () => {
  for (const [typed, meant] of [
    [fixed, fixed],
    ['Acorn-BASIL-Robin-0042', fixed],
    ['acorn%20basil%20robin%200042', fixed],
    ['acorn.basil_robin+0042/', fixed],
    ['acorn basil robin0042', fixed],
    ['acorn-basil-robin-00-42', fixed],
    ['acorm-basil-robin-0042', fixed],
    ['acorns-basil-robin-0042', fixed],
    ['acorn-basil-robbin-0042', fixed],
    ['aocrn-basil-rboin-0042', fixed],
    [`${fixed}?ref=chat`, fixed],
    [old, old],
    ['ACID acorn acre acts afar/', old],
  ]) {
    const { handle, asked } = store();
    const answer = await handle(new Request(`https://join.example/${typed}`), '192.0.2.1');
    assert.equal(answer.status, 200, typed);
    assert.deepEqual(asked, [meant], `${typed}: one lookup of the canonical phrase`);
  }
});

test('an address that cannot be read names what is wrong, and asks the store nothing', async () => {
  const shape = /An invitation address is three words and a four-digit number/;
  for (const [typed, said] of [
    ['', shape],
    ['list', shape],
    ['favicon.ico', shape],
    ['acorn-basil-robin', shape],
    ['acorn-basil-robin-042', shape],
    ['acorn-basil-robin-eagle-0042', shape],
    [`${fixed}-${fixed}-${fixed}`, shape],
    // apron and acorn are each one slip from it, acre and acorn too.
    ['acron-basil-robin-0042', /“acron” isn't one of the words/],
    ['acrn-basil-robin-0042', /“acrn” isn't one of the words/],
    ['acorn-basil-zzzzz-0042', /“zzzzz” isn't one of the words/],
  ]) {
    const { handle, asked } = store();
    const json = await handle(new Request(`https://join.example/${typed}`), '192.0.2.1');
    assert.equal(json.status, 404, typed);
    const answer = await json.json();
    assert.equal(answer.state, 'wrong');
    assert.match(answer.message, said);
    assert.match(answer.message, /with whoever gave it to you/);
    const page = await handle(
      new Request(`https://join.example/${typed}`, { headers: html }),
      '192.0.2.1',
    );
    assert.equal(page.status, 404);
    assert.match(page.headers.get('content-security-policy'), /^default-src 'none'/);
    assert.match(await page.text(), said);
    assert.deepEqual(asked, [], `${typed}: the store is never asked`);
  }
});

test('each answer says what happened: never issued, ended, or unavailable', async () => {
  for (const [result, status, said] of [
    [
      { state: 'missing', phrase: '', secret: false },
      404,
      /No invitation has this address\. Check each word and the number/,
    ],
    [{ state: 'ended', phrase: '', secret: false }, 200, /This invitation has ended/],
    [{ state: 'unavailable', reason: 'pending' }, 503, /Joining is temporarily unavailable/],
    [
      { state: 'unavailable', reason: 'lookup_limit' },
      503,
      /send the link in a chat or show its QR code/,
    ],
  ])
    for (const headers of [{}, html]) {
      const { handle, asked } = store(result);
      const answer = await handle(
        new Request(`https://join.example/${fixed}`, { headers }),
        '192.0.2.1',
      );
      assert.equal(answer.status, status, result.state);
      const text = await answer.text();
      assert.match(text, said);
      assert.doesNotMatch(text, /<script|ordinary-invitation/);
      if (status !== 503) assert.doesNotMatch(text, /temporarily unavailable/);
      assert.deepEqual(asked, [fixed]);
    }
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
    new Request('https://join.example/', { method: 'POST', body: JSON.stringify(body()) }),
    new Request(`https://join.example/${fixed}`, { method: 'PUT' }),
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
  await refused(answer);
});

test('debug events name codes and store reasons, never foreign text', async () => {
  const seen = async (reply, request = new Request(`https://join.example/${fixed}`)) => {
    const events = [];
    const handle = createResolver({
      token: 'fixture',
      fetch: async () => reply(),
      debug: (e) => events.push(e),
    });
    await handle(request, '127.0.0.1');
    return events;
  };
  const refusing = (reason) => () =>
    Response.json({ result: JSON.stringify({ state: 'unavailable', reason }) });
  assert.deepEqual(await seen(() => new Response('', { status: 500 })), [
    'read:phrase',
    'unavailable:provider_status',
  ]);
  assert.deepEqual(await seen(() => new Response(`{"result": ${fixed}`)), [
    'read:phrase',
    'unavailable:transport_or_decode',
  ]);
  assert.deepEqual(
    await seen(() => {
      throw new Error(`TLS ${fixed}`);
    }),
    ['read:phrase', 'unavailable:transport_or_decode'],
  );
  for (const reason of ['generation', 'lookup_limit', 'pending', 'missing_secret'])
    assert.deepEqual(await seen(refusing(reason)), [
      'read:phrase',
      `store:${reason}`,
      'lookup:unavailable',
      'unavailable:store_refusal',
    ]);
  assert.deepEqual(await seen(refusing(fixed)), [
    'read:phrase',
    'store:refused',
    'lookup:unavailable',
    'unavailable:store_refusal',
  ]);
  assert.deepEqual(await seen(refusing(), mutation('x'.repeat(8193))), ['unavailable:body_limit']);
  assert.deepEqual(await seen(refusing(), new Request('https://join.example/acron-x')), [
    'read:wrong',
  ]);
  // 2,000 quotes fit the 8 KiB body, but escaped twice in the store's reply they do not.
  assert.deepEqual(await seen(refusing(), mutation(body({ token: '"'.repeat(2000) }))), [
    'unavailable:secret_response_limit',
  ]);
  assert.deepEqual(await seen(refusing('lookup_limit'), mutation(body())), [
    'store:lookup_limit',
    'publish:unavailable',
    'unavailable:store_refusal',
  ]);
});

test('a lookup is one EVAL of five keys under one hash tag', async () => {
  const sent = [];
  const handle = createResolver({
    token: 'fixture',
    fetch: async (_url, options) => {
      sent.push(JSON.parse(options.body));
      return Response.json({
        result: JSON.stringify({ state: 'ended', phrase: '', secret: false }),
      });
    },
  });
  await handle(new Request(`https://join.example/${fixed}`), '127.0.0.1');
  assert.equal(sent.length, 1);
  const [command] = sent;
  assert.deepEqual(command.slice(0, 2), ['EVAL', SCRIPT]);
  assert.deepEqual(command.slice(2, 9), [
    '5',
    `${PREFIX}:generation`,
    `${PREFIX}:reservations`,
    `${PREFIX}:active`,
    `${PREFIX}:quotas`,
    `${PREFIX}:quota:ip:${hash('127.0.0.1')}`,
    GENERATION,
  ]);
  assert.equal(JSON.parse(command[9]).phrase, fixed);
  assert.equal(command.length, 10);
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
