// What Spot's server does with a chat, against a stand-in provider and a
// stand-in svrngs: the request each person's door would carry is sent as
// the door sends it (x-svrngs-key, x-svrngs-person, the lender's key as
// authorization). That the door adds the key in place of a friend's and
// never lets it back is svrngs's: crates/svrngs-node/tests/home.rs
// `a_header_the_door_adds_is_given_on_the_home_page`.
//
// Plants, each watched failing in server.mjs, then put back: named in the
// commit that brought these laws.

import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { endOf, start as listen } from './server.mjs';

// The lend on a free port, closed when its test ends, passed or failed.
async function start(t, options) {
  const lend = await listen({ port: 0, ...options });
  t.after(() => lend.server.close() && lend.server.closeAllConnections());
  return lend;
}

// A provider: what it was sent, and a streamed or whole answer with usage.
async function provider(t) {
  const seen = [];
  const server = http.createServer(async (req, res) => {
    let body = '';
    for await (const c of req) body += c;
    seen.push({ url: req.url, headers: req.headers, body: JSON.parse(body) });
    if (JSON.parse(body).stream) {
      res.writeHead(200, { 'content-type': 'text/event-stream' });
      res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'hi' } }] })}\n\n`);
      res.write(`data: ${JSON.stringify({ choices: [], usage: { total_tokens: 7 } })}\n\n`);
      return res.end('data: [DONE]\n\n');
    }
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(
      JSON.stringify({ choices: [{ message: { content: 'ok' } }], usage: { total_tokens: 5 } }),
    );
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  t.after(() => server.close() && server.closeAllConnections());
  return { seen, url: `http://127.0.0.1:${server.address().port}/v1/openai`, server };
}

// svrngs's record API, as `--svrngs` reads it: its api file, the token beside it.
async function svrngs(t, person) {
  const dir = mkdtempSync(join(tmpdir(), 'spot-'));
  const server = http.createServer((req, res) => {
    const ok = req.headers.authorization === 'Bearer tok';
    res.writeHead(ok ? 200 : 401, { 'content-type': 'application/json' });
    res.end(JSON.stringify(ok ? { groups: [{ id: 'g', person }] } : { error: 'no_token' }));
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  t.after(() => server.close() && server.closeAllConnections());
  writeFileSync(join(dir, 'api'), `http://127.0.0.1:${server.address().port}\n`);
  writeFileSync(join(dir, 'token'), 'tok\n');
  return { api: join(dir, 'api'), server };
}

// A request as svrngs's door carries it to the lend: as `who`, with the key.
const as = (who, key = `key-of-${who}`) => ({
  'x-svrngs-key': key,
  'x-svrngs-person': who,
  authorization: 'Bearer lenders-key',
  cookie: 'a=1',
  'x-svrngs-device': 'phone',
});
async function ask(lend, method, path, headers = {}, body) {
  const r = await fetch(`http://127.0.0.1:${lend.port}${path}`, {
    method,
    headers: { 'content-type': 'application/json', ...headers },
    body: body && JSON.stringify(body),
  });
  return { status: r.status, text: await r.text() };
}
const chat = (model, more = {}) => ({
  model,
  messages: [{ role: 'user', content: 'hi' }],
  ...more,
});
const settled = () => new Promise((done) => setTimeout(done, 50));

test('a chat goes to the provider as its model, with the key and nothing of the person', async (t) => {
  const p = await provider(t);
  const lend = await start(t, { provider: p.url, models: new Map([['flash', 'org/Flash-1']]) });
  const r = await ask(lend, 'POST', '/v1/chat/completions', as('bo'), chat('flash'));
  assert.equal(r.status, 200, r.text);
  assert.equal(JSON.parse(r.text).choices[0].message.content, 'ok');
  const [sent] = p.seen;
  assert.equal(sent.url, '/v1/openai/chat/completions');
  assert.equal(sent.body.model, 'org/Flash-1');
  assert.equal(sent.headers.authorization, 'Bearer lenders-key');
  const named = Object.keys(sent.headers).filter((h) => /svrngs|cookie/.test(h));
  assert.deepEqual(named, [], 'no x-svrngs-* and no cookie reach the provider');
});

test('tokens are counted per person by the door’s key, two of one name apart, a stream’s too', async (t) => {
  const p = await provider(t);
  const lend = await start(t, { provider: p.url, models: new Map([['flash', 'org/Flash-1']]) });
  await ask(lend, 'POST', '/v1/chat/completions', as('sam', 'k1'), chat('flash'));
  await ask(lend, 'POST', '/v1/chat/completions', as('sam', 'k2'), chat('flash', { stream: true }));
  await settled();
  assert.equal(
    p.seen[1].body.stream_options.include_usage,
    true,
    'a stream is asked for its usage',
  );
  assert.deepEqual(Object.fromEntries(lend.used), {
    k1: { name: 'sam', tokens: 5 },
    k2: { name: 'sam', tokens: 7 },
  });
  const mine = JSON.parse((await ask(lend, 'GET', '/spot.json', as('sam', 'k2'))).text);
  assert.equal(mine.yours, 7);
  assert.equal(mine.spent, 12);
});

test('each refusal is said, and the provider is never asked', async (t) => {
  const p = await provider(t);
  const lend = await start(t, { provider: p.url, models: new Map([['flash', 'org/Flash-1']]) });
  const { authorization, ...noKey } = as('bo');
  const { 'x-svrngs-key': _, ...notThrough } = as('bo');
  for (const [headers, method, path, body, status] of [
    [noKey, 'POST', '/v1/chat/completions', chat('flash'), 401],
    [notThrough, 'POST', '/v1/chat/completions', chat('flash'), 401],
    [as('bo'), 'POST', '/v1/chat/completions', chat('org/Flash-1'), 400],
    [as('bo'), 'POST', '/v1/embeddings', chat('flash'), 404],
    [as('bo'), 'GET', '/slots', undefined, 404],
  ]) {
    const r = await ask(lend, method, path, headers, body);
    assert.equal(r.status, status, `${method} ${path}: ${r.text}`);
    assert.equal(JSON.parse(r.text).error.type, 'spot_refused');
  }
  assert.equal(p.seen.length, 0);
});

test('past its end, or its budget spent, a chat is refused and the page says why', async (t) => {
  const p = await provider(t);
  const ended = await start(t, {
    provider: p.url,
    models: new Map([['flash', 'org/Flash-1']]),
    until: new Date(Date.now() - 1000),
  });
  assert.equal(
    (await ask(ended, 'POST', '/v1/chat/completions', as('bo'), chat('flash'))).status,
    403,
  );
  const lend = await start(t, {
    provider: p.url,
    models: new Map([['flash', 'org/Flash-1']]),
    budget: 5,
  });
  assert.equal(
    (await ask(lend, 'POST', '/v1/chat/completions', as('bo'), chat('flash'))).status,
    200,
  );
  await settled();
  const r = await ask(lend, 'POST', '/v1/chat/completions', as('cy'), chat('flash'));
  assert.equal(r.status, 403, r.text);
  assert.match(
    JSON.parse((await ask(lend, 'GET', '/spot.json', as('cy'))).text).ended,
    /5 tokens are spent/,
  );
  assert.equal(p.seen.length, 1);
});

test('the page names its lender as svrngs does, and lists what is lent', async (t) => {
  const p = await provider(t);
  const s = await svrngs(t, 'ana');
  const models = new Map([
    ['flash', 'org/Flash-1'],
    ['pro', 'org/Pro-2'],
  ]);
  const lend = await start(t, { provider: p.url, models, api: s.api });
  const said = JSON.parse((await ask(lend, 'GET', '/spot.json', as('bo'))).text);
  assert.equal(said.lender, 'ana');
  assert.deepEqual(said.models, ['flash', 'pro']);
  assert.equal(said.provider, '127.0.0.1');
  const listed = JSON.parse((await ask(lend, 'GET', '/v1/models', as('bo'))).text);
  assert.deepEqual(
    listed.data.map((m) => [m.id, m.owned_by]),
    [
      ['flash', 'ana'],
      ['pro', 'ana'],
    ],
  );
  const page = await ask(lend, 'GET', '/', as('bo'));
  assert.equal(page.status, 200);
  assert.match(page.text, /<script type="module" src="page.js">/);
});

test('--until HH:MM is its next time; a time gone is refused', () => {
  const now = new Date(2026, 10, 17, 23, 0);
  assert.equal(endOf('02:00', now).getTime(), new Date(2026, 10, 18, 2, 0).getTime());
  assert.equal(endOf('23:30', now).getTime(), new Date(2026, 10, 17, 23, 30).getTime());
  assert.equal(endOf(null, now), null);
  assert.throws(() => endOf('2026-11-01T02:00', now), /a time to come/);
  assert.throws(() => endOf('2am', now), /a time to come/);
});
