// The pantry's minimum (CHARTER.md, "The minimum"; bars C1, C2, C3, C6):
// stand-in people in a group lend stand-in servers, and the pantry finds
// them by asking their protocol, never by a kind it keeps a list of.

import assert from 'node:assert/strict';
import http from 'node:http';
import net from 'node:net';
import { createHash } from 'node:crypto';
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { FOUND_FOR, LENDS_FOR, formatList, list, serve } from './pantry.mjs';

const HERE = new URL('.', import.meta.url).pathname;

function body(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => resolve(Buffer.concat(chunks)));
    req.on('error', reject);
  });
}

function json(res, status, value) {
  res.writeHead(status, { 'content-type': 'application/json' });
  res.end(JSON.stringify(value));
}

async function standin(handler) {
  const requests = [];
  const server = http.createServer(async (req, res) => {
    const raw = await body(req);
    const seen = { method: req.method, path: req.url, headers: req.headers, body: raw, at: Date.now() };
    requests.push(seen);
    try {
      await handler(req, res, seen);
    } catch (e) {
      json(res, 500, { error: String(e) });
    }
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}/`;
  return {
    origin,
    requests,
    close: () => new Promise((resolve) => {
      server.close(resolve);
      server.closeAllConnections?.();
    }),
  };
}

async function group(lends, unlisted = {}, { delay = 0 } = {}) {
  return standin(async (req, res) => {
    if (req.url === '/v1/groups') {
      return json(res, 200, { groups: [...new Set(lends.map((l) => l.group))].map((id) => ({ id, name: id })) });
    }
    const at = new URL(req.url, 'http://record');
    if (at.pathname === '/v1/lends') {
      if (delay) await new Promise((resolve) => setTimeout(resolve, delay));
      const namespace = at.searchParams.get('namespace');
      fsLends(res, lends.filter((l) => l.group === namespace), unlisted);
      return;
    }
    json(res, 404, { error: 'not a record route' });
  });
}

function fsLends(res, lends, unlisted) {
  json(res, 200, {
    namespace: lends[0]?.group ?? null,
    lent: lends.map(({ group, ...l }) => l),
    unanswered: unlisted.unanswered ?? [],
    unheard: unlisted.unheard ?? [],
    refused: unlisted.refused ?? [],
    older: unlisted.older ?? [],
  });
}

function modelLend({ id, reply = 'ok', extra = [] }) {
  return standin(async (req, res, seen) => {
    if (req.url === '/v1/models') {
      return json(res, 200, { object: 'list', data: [{ id, object: 'model' }] });
    }
    if (req.url === '/v1/chat/completions') {
      const asked = JSON.parse(seen.body);
      if (asked.stream) {
        res.writeHead(200, { 'content-type': 'text/event-stream' });
        res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'first' } }] })}\n\n`);
        await new Promise((r) => setTimeout(r, 150));
        seen.secondSent = true;
        res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'second' } }] })}\n\n`);
        res.end('data: [DONE]\n\n');
        return;
      }
      return json(res, 200, {
        object: 'chat.completion',
        model: asked.model,
        choices: [{ index: 0, message: { role: 'assistant', content: reply } }],
        seen: asked,
      });
    }
    if (req.url === '/v1/audio/transcriptions') {
      const form = await new Response(seen.body, { headers: seen.headers }).formData();
      return json(res, 200, {
        text: await form.get('file').text(),
        model: form.get('model'),
        language: form.get('language'),
      });
    }
    if (req.url === '/v1/audio/speech') {
      const asked = JSON.parse(seen.body);
      return json(res, 200, { model: asked.model, seen: asked });
    }
    if (extra.includes(req.url)) {
      const asked = JSON.parse(seen.body || '{}');
      return json(res, 200, { path: req.url, model: asked.model, seen: asked });
    }
    json(res, 404, { error: 'no such route' });
  });
}

function toolLend(tools) {
  return standin(async (req, res, seen) => {
    if (req.url !== '/mcp') return json(res, 404, { error: 'no such route' });
    const rpc = JSON.parse(seen.body);
    const session = 'standin-session';
    if (rpc.method === 'initialize') {
      res.setHeader('mcp-session-id', session);
      return json(res, 200, {
        jsonrpc: '2.0', id: rpc.id,
        result: {
          protocolVersion: '2025-06-18',
          capabilities: { tools: {} },
          serverInfo: { name: 'standin', version: '0' },
        },
      });
    }
    if (rpc.method === 'notifications/initialized') {
      res.writeHead(202);
      return res.end();
    }
    if (rpc.method === 'tools/list') {
      assert.equal(seen.headers['mcp-session-id'], session);
      return json(res, 200, { jsonrpc: '2.0', id: rpc.id, result: { tools } });
    }
    json(res, 200, { jsonrpc: '2.0', id: rpc.id, error: { code: -32601, message: 'no' } });
  });
}

function silent() {
  return standin((req, res) => json(res, 404, { error: 'not me' }));
}

async function pantry(opts) {
  return serve({ host: '127.0.0.1', port: 0, ...opts });
}

// C1: discovery by protocol, no registration.
test('list names each lend by asking it, with whose computer, its address and its tools', async (t) => {
  const ana = await modelLend({ id: 'coder' });
  const sam = await toolLend([
    { name: 'whoami', annotations: { readOnlyHint: true } },
    { name: 'erase', annotations: { destructiveHint: true } },
    { name: 'plain' },
  ]);
  const cy = await silent();
  const record = await group([
    { group: 'g1', name: 'ana-coder', person: 'ana', origin: ana.origin, kind: 'anything-at-all', limit: null, using: 0 },
    { group: 'g1', name: 'cy-odd', person: 'cy', origin: cy.origin, kind: 'unheard-of', limit: null, using: 0 },
    { group: 'g2', name: 'sam-tools', person: 'sam', origin: sam.origin, kind: 'mcp', limit: null, using: 0 },
  ]);
  t.after(() => Promise.all([ana.close(), sam.close(), cy.close(), record.close()]));

  const found = await list({ record: record.origin });
  const by = Object.fromEntries(found.lends.map((l) => [l.person, l]));

  assert.equal(by.ana.protocol, 'model');
  assert.equal(by.ana.group, 'g1');
  assert.ok(by.ana.origin.startsWith('http://127.0.0.1:'));
  assert.deepEqual(by.ana.models.map((m) => m.id), ['coder']);
  assert.equal(by.ana.models[0].as, 'ana/coder');

  assert.equal(by.sam.protocol, 'tool');
  assert.equal(by.sam.group, 'g2');
  assert.deepEqual(by.sam.tools.map((x) => [x.name, x.effect]),
    [['whoami', 'read-only'], ['erase', 'destructive'], ['plain', 'not declared']]);

  assert.equal(by.cy.protocol, 'neither');
  assert.match(by.cy.why, /\/v1\/models/);
  assert.match(by.cy.why, /MCP/);
});

// C2: one address, routed by model, every endpoint, everything else untouched.
test('serve routes chat, multipart transcription and speech by the prefixed model', async (t) => {
  const ana = await modelLend({ id: 'coder' });
  const stt = await modelLend({ id: 'stt' });
  const voice = await modelLend({ id: 'voice' });
  const record = await group([
    { group: 'g', name: 'ana-coder', person: 'ana', origin: ana.origin, kind: 'x', limit: null, using: 0 },
    { group: 'g', name: 'sam-stt', person: 'sam', origin: stt.origin, kind: 'x', limit: null, using: 0 },
    { group: 'g', name: 'sam-voice', person: 'sam', origin: voice.origin, kind: 'x', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([ana.close(), stt.close(), voice.close(), record.close(), at.close()]));

  const models = await (await fetch(`${at.url}/v1/models`)).json();
  assert.deepEqual(models.data.map((m) => m.id).sort(), ['ana/coder', 'sam/stt', 'sam/voice']);

  const tools = [{ type: 'function', function: { name: 'weather', parameters: { type: 'object' } } }];
  const chat = {
    model: 'ana/coder', temperature: 0.3, tools, tool_choice: 'auto',
    messages: [{ role: 'user', content: 'hi' }],
  };
  const answer = await (await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(chat),
  })).json();
  assert.equal(ana.requests.filter((r) => r.path === '/v1/chat/completions').length, 1);
  assert.equal(stt.requests.filter((r) => r.path === '/v1/chat/completions').length, 0);
  assert.equal(answer.seen.model, 'coder', 'the lend is asked for its own id');
  assert.deepEqual(answer.seen.tools, tools, 'tools pass through untouched');
  assert.equal(answer.seen.temperature, 0.3);
  assert.equal(answer.seen.tool_choice, 'auto');

  const form = new FormData();
  form.set('model', 'sam/stt');
  form.set('language', 'en');
  form.set('file', new Blob([Buffer.from('the quick brown fox')], { type: 'audio/wav' }), 'say.wav');
  const heard = await (await fetch(`${at.url}/v1/audio/transcriptions`, { method: 'POST', body: form })).json();
  assert.equal(heard.model, 'stt', 'the multipart model is read from the form and written back');
  assert.equal(heard.language, 'en');
  assert.equal(heard.text, 'the quick brown fox', 'the file bytes pass through untouched');

  const said = await (await fetch(`${at.url}/v1/audio/speech`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'sam/voice', input: 'hello', voice: 'af_heart' }),
  })).json();
  assert.equal(said.seen.model, 'voice');
  assert.equal(said.seen.voice, 'af_heart');
});

test('a streamed answer reaches the client chunk by chunk, not buffered', async (t) => {
  const ana = await modelLend({ id: 'coder' });
  const record = await group([
    { group: 'g', name: 'ana-coder', person: 'ana', origin: ana.origin, kind: 'x', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([ana.close(), record.close(), at.close()]));

  const res = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'ana/coder', stream: true, messages: [{ role: 'user', content: 'hi' }] }),
  });
  const reader = res.body.getReader();
  const first = await reader.read();
  const seen = ana.requests.find((r) => r.path === '/v1/chat/completions');
  assert.match(Buffer.from(first.value).toString(), /first/);
  assert.equal(seen.secondSent, undefined, 'the first chunk arrived before the lend sent the second');
  await reader.cancel();
});

// C3: a server the pantry has never seen works as it is.
test('an unregistered server, path and body are listed and answered with the pantry unchanged', async (t) => {
  const cy = await modelLend({ id: 'never-seen-id', extra: ['/v7/odd/path'] });
  const record = await group([
    { group: 'g', name: 'cy-thing', person: 'cy', origin: cy.origin, kind: 'a-kind-no-one-registered', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([cy.close(), record.close(), at.close()]));

  const before = hashFiles(HERE);
  const found = await list({ record: record.origin });
  assert.equal(found.lends[0].protocol, 'model');
  assert.equal(found.lends[0].models[0].as, 'cy/never-seen-id');

  const odd = { what: 'ever', nested: { a: [1, 2, 3] }, model: 'cy/never-seen-id' };
  const answer = await (await fetch(`${at.url}/v7/odd/path`, {
    method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(odd),
  })).json();
  assert.equal(answer.path, '/v7/odd/path');
  assert.equal(answer.seen.model, 'never-seen-id');
  assert.equal(answer.seen.what, 'ever');
  assert.deepEqual(answer.seen.nested, { a: [1, 2, 3] });

  assert.deepEqual(hashFiles(HERE), before, 'the pantry learned nothing');
});

function hashFiles(dir) {
  const out = {};
  for (const name of readdirSync(dir)) {
    if (!name.endsWith('.mjs') || name.endsWith('.test.mjs')) continue;
    const p = join(dir, name);
    if (statSync(p).isFile()) out[name] = createHash('sha256').update(readFileSync(p)).digest('hex');
  }
  return out;
}

// C6: the refusal is the payload, and no other model answers.
test('an asleep, full or unknown lender is named and never substituted', async (t) => {
  const sam = await modelLend({ id: 'voice' });
  const full = await standin((req, res) => {
    if (req.url === '/v1/models') return json(res, 200, { object: 'list', data: [{ id: 'full', object: 'model' }] });
    if (req.url === '/v1/chat/completions') {
      res.writeHead(429, { 'content-type': 'application/json', 'x-svrngs-refused': 'full' });
      return res.end(JSON.stringify({ error: 'sam-full is full (1 of 1)', kind: 'full' }));
    }
    return json(res, 404, { error: 'not me' });
  });
  const record = await group([
    { group: 'g', name: 'sam-voice', person: 'sam', origin: sam.origin, kind: 'x', limit: null, using: 0 },
    { group: 'g', name: 'sam-full', person: 'sam', origin: full.origin, kind: 'x', limit: 1, using: 1 },
  ], { unanswered: ['ana'] });
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([sam.close(), full.close(), record.close(), at.close()]));

  const models = await (await fetch(`${at.url}/v1/models`)).json();
  assert.deepEqual(models.data.map((m) => m.id).sort(), ['sam/full', 'sam/voice']);

  const asleep = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'ana/coder', messages: [{ role: 'user', content: 'hi' }] }),
  });
  assert.equal(asleep.status, 503);
  const asleepBody = await asleep.text();
  assert.match(asleepBody, /ana/);
  assert.match(asleepBody, /did not answer|asleep/);
  assert.equal(sam.requests.filter((r) => r.path === '/v1/chat/completions').length, 0);

  const limited = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'sam/full', messages: [{ role: 'user', content: 'hi' }] }),
  });
  assert.equal(limited.status, 429);
  const limitedBody = await limited.text();
  assert.match(limitedBody, /sam/);
  assert.match(limitedBody, /full/);
  assert.match(limitedBody, /full/);
  assert.equal(limited.headers.get('x-svrngs-refused'), 'full',
    'the door refused it, and the pantry passed that through');
  assert.equal(full.requests.filter((r) => r.path === '/v1/chat/completions').length, 1,
    'the door decided, not the pantry');
  assert.equal(sam.requests.filter((r) => r.path === '/v1/chat/completions').length, 0);

  const unknown = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'nobody/none', messages: [{ role: 'user', content: 'hi' }] }),
  });
  assert.equal(unknown.status, 404);
  assert.match(await unknown.text(), /nobody\/none/);
  assert.equal(sam.requests.filter((r) => r.path === '/v1/chat/completions').length, 0);
});

function stateDir() {
  return mkdtempSync(join(process.cwd(), 'target', 'pantry-test-'));
}

// (0)(i): a lend whose protocol did not answer in time is could-not-judge,
// absent from /v1/models with its reason in list, and a request naming it is
// a 503 naming the lend and why, never 404 "no lend answers" (AGENTS.md 6).
test('a lend that did not answer in time is could-not-judge, absent, and refused by name', async (t) => {
  let clock = 1000;
  const now = () => clock;
  const mute = await standin(async () => {
    await new Promise((resolve) => {
      const held = setTimeout(resolve, 4500);
      held.unref?.();
    });
  });
  const sam = await modelLend({ id: 'voice' });
  const record = await group([
    { group: 'g', key: 'ana-key', name: 'ana-coder', person: 'ana', origin: mute.origin, kind: 'x', limit: null, using: 0 },
    { group: 'g', key: 'sam-key', name: 'sam-voice', person: 'sam', origin: sam.origin, kind: 'x', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin, clock: now });
  t.after(() => Promise.all([mute.close(), sam.close(), record.close(), at.close()]));

  const models = await (await fetch(`${at.url}/v1/models`)).json();
  assert.deepEqual(models.data.map((m) => m.id), ['sam/voice'], 'the silent lend is absent');

  const seen = await list({ record: record.origin, clock: now });
  const ana = seen.lends.find((l) => l.person === 'ana');
  assert.equal(ana.protocol, 'neither');
  assert.equal(ana.couldNotJudge, true);
  assert.match(ana.why, /did not answer/);
  assert.match(await formatList(seen), /could-not-judge/);

  clock += 1000;
  const asleep = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'ana/coder', messages: [{ role: 'user', content: 'hi' }] }),
  });
  assert.equal(asleep.status, 503, 'never 404 "no lend answers"');
  const body = await asleep.text();
  assert.match(body, /ana/);
  assert.match(body, /did not answer|could-not-judge/);
  assert.match(body, /looked 1s ago/, 'the error says how old the look was');
  assert.equal(sam.requests.filter((r) => r.path === '/v1/chat/completions').length, 0,
    'another lend never answers it');
});

// (0)(ii): ids stay readable but are unique by identity, from the lend's key
// in /v1/lends, and never one dropped in silence (AGENTS.md 8).
test('two people of one name are told apart by their key and both routed', async (t) => {
  const first = await modelLend({ id: 'coder', reply: 'first' });
  const second = await modelLend({ id: 'coder', reply: 'second' });
  const record = await group([
    { group: 'g', key: 'aa'.repeat(32), name: 'sam-one', person: 'sam', origin: first.origin, kind: 'x', limit: null, using: 0 },
    { group: 'g', key: 'bb'.repeat(32), name: 'sam-two', person: 'sam', origin: second.origin, kind: 'x', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([first.close(), second.close(), record.close(), at.close()]));

  const models = await (await fetch(`${at.url}/v1/models`)).json();
  const ids = models.data.map((m) => m.id).sort();
  assert.equal(ids.length, 2, 'neither one is dropped');
  assert.notEqual(ids[0], ids[1]);
  for (const id of ids) assert.match(id, /^sam[~.\-]/);
  for (const id of ids) assert.match(id, /\/coder$/);

  const chats = (lend) => lend.requests.filter((r) => r.path === '/v1/chat/completions').length;
  for (const [id, target, spare, said] of [[ids[0], first, second, 'first'], [ids[1], second, first, 'second']]) {
    const before = chats(spare);
    const answer = await (await fetch(`${at.url}/v1/chat/completions`, {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ model: id, messages: [{ role: 'user', content: 'hi' }] }),
    })).json();
    assert.equal(answer.choices[0].message.content, said, `${id} reaches its own computer`);
    assert.equal(chats(target), 1);
    assert.equal(chats(spare), before, `${id} reaches no other computer`);
  }
});

test('one person\'s two lends of one model id are both listed as serve routes them', async (t) => {
  const first = await modelLend({ id: 'coder', reply: 'first' });
  const second = await modelLend({ id: 'coder', reply: 'second' });
  const record = await group([
    { group: 'g', key: 'aa'.repeat(32), name: 'ana-one', person: 'ana', origin: first.origin, kind: 'x', limit: null, using: 0 },
    { group: 'g', key: 'aa'.repeat(32), name: 'ana-two', person: 'ana', origin: second.origin, kind: 'x', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([first.close(), second.close(), record.close(), at.close()]));

  const listed = (await list({ record: record.origin })).lends.flatMap((l) => l.models.map((m) => m.as));
  const served = (await (await fetch(`${at.url}/v1/models`)).json()).data.map((m) => m.id);
  assert.deepEqual(listed, ['ana/coder', 'ana/ana-two/coder']);
  assert.deepEqual(served, listed, 'list shows the ids serve routes by');

  for (const [id, said] of [['ana/coder', 'first'], ['ana/ana-two/coder', 'second']]) {
    const answer = await (await fetch(`${at.url}/v1/chat/completions`, {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ model: id, messages: [{ role: 'user', content: 'hi' }] }),
    })).json();
    assert.equal(answer.choices[0].message.content, said, `${id} reaches its own lend`);
  }
});

// (0)(iii): discovery is kept for a short time, named once, its age said in
// list and in each error, so an asleep friend does not cost every message a
// dial (crates/svrngs-node/src/dial.rs:30).
test('discovery is kept for a short time and its age is said', async (t) => {
  let clock = 1000;
  const now = () => clock;
  const ana = await modelLend({ id: 'coder' });
  const record = await group([
    { group: 'g', key: 'k', name: 'ana-coder', person: 'ana', origin: ana.origin, kind: 'x', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin, clock: now });
  t.after(() => Promise.all([ana.close(), record.close(), at.close()]));

  const probes = () => ana.requests.filter((r) => r.path === '/v1/models').length;
  await fetch(`${at.url}/v1/models`);
  assert.equal(probes(), 1);
  await fetch(`${at.url}/v1/models`);
  assert.equal(probes(), 1, 'the second request does not dial again');

  clock += 1000;
  await fetch(`${at.url}/v1/models`);
  assert.equal(probes(), 1, 'still inside the time it is kept');
  const seen = await list({ record: record.origin, clock: now });
  assert.match(await formatList(seen), /looked 1s ago/);

  clock += FOUND_FOR;
  await fetch(`${at.url}/v1/models`);
  assert.equal(probes(), 2, 'older than FOUND_FOR, it is asked again');
});

// models-fresh (a): one friend asleep costs one ask of the record, not one
// per message. The record's `/v1/lends` is where an asleep friend's dial is
// waited on (`crates/svrngs-node/src/node.rs:109`), so the stand-in record
// takes a delay; the requests between refreshes must not pay it.
test('a lender asleep costs one ask of the record, not every message', async (t) => {
  let clock = 1000;
  const now = () => clock;
  const ana = await standin(async () => {});
  const sam = await modelLend({ id: 'voice' });
  const record = await group([
    { group: 'g', key: 'sam-key', name: 'sam-voice', person: 'sam', origin: sam.origin, kind: 'x', limit: null, using: 0 },
  ], { unanswered: ['ana'] }, { delay: 600 });
  const at = await pantry({ record: record.origin, clock: now });
  t.after(() => Promise.all([ana.close(), sam.close(), record.close(), at.close()]));

  const lendsAsked = () => record.requests.filter((r) => r.path.startsWith('/v1/lends')).length;
  const chat = async () => {
    const began = Date.now();
    const answer = await fetch(`${at.url}/v1/chat/completions`, {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ model: 'sam/voice', messages: [{ role: 'user', content: 'hi' }] }),
    });
    return { status: answer.status, body: await answer.text(), elapsed: Date.now() - began };
  };

  const first = await chat();
  assert.equal(first.status, 200);
  assert.equal(lendsAsked(), 1, 'the first request asks the record');

  for (let i = 0; i < 3; i += 1) {
    clock += FOUND_FOR + 1000;
    const again = await chat();
    assert.equal(again.status, 200);
    assert.ok(again.elapsed < 500,
      `request ${i + 2} waited ${again.elapsed} ms on the asleep friend`);
  }
  assert.equal(lendsAsked(), 1, 'the record is asked once while its answer is kept');

  clock += 1000;
  const began = Date.now();
  const asleep = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'ana/whisper-1', messages: [{ role: 'user', content: 'hi' }] }),
  });
  const asleepBody = await asleep.text();
  const asleepElapsed = Date.now() - began;
  assert.equal(asleep.status, 503, 'the asleep lender is named, never answered by another');
  assert.match(asleepBody, /ana/);
  assert.match(asleepBody, /asleep|did not answer/);
  assert.match(asleepBody, /looked \d+s ago/, 'the refusal says the age of what it used');
  assert.ok(asleepElapsed < 500, `the refusal waited ${asleepElapsed} ms on the asleep friend`);
  assert.equal(lendsAsked(), 1, 'the asleep lender is kept too, so asking for it asks nothing again');

  clock = 1000 + LENDS_FOR;
  const older = await chat();
  assert.equal(older.status, 200);
  assert.equal(lendsAsked(), 2, 'the record is asked again once its answer outlives its time');
});

// (0)(iv): the door decides limits. The pantry keeps no `using` check and
// passes the door's JSON refusal and its refusal header back unchanged
// (crates/svrngs-node/src/door.rs:254-268; AGENTS.md 8, 12).
test('the door decides limits: its refusal and header pass through unchanged', async (t) => {
  const door = await standin((req, res) => {
    if (req.url === '/v1/models') return json(res, 200, { object: 'list', data: [{ id: 'strong', object: 'model' }] });
    if (req.url === '/v1/chat/completions') {
      const refusal = { error: 'sam-strong is full (1 of 1)', kind: 'full' };
      res.writeHead(429, { 'content-type': 'application/json', 'x-svrngs-refused': 'full' });
      return res.end(JSON.stringify(refusal));
    }
    return json(res, 404, { error: 'not me' });
  });
  const record = await group([
    { group: 'g', key: 'k', name: 'sam-strong', person: 'sam', origin: door.origin, kind: 'x', limit: 1, using: 1 },
  ]);
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([door.close(), record.close(), at.close()]));

  const refused = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'sam/strong', messages: [{ role: 'user', content: 'hi' }] }),
  });
  assert.equal(refused.status, 429);
  assert.equal(refused.headers.get('x-svrngs-refused'), 'full');
  assert.deepEqual(await refused.json(), { error: 'sam-strong is full (1 of 1)', kind: 'full' });
  assert.equal(door.requests.filter((r) => r.path === '/v1/chat/completions').length, 1,
    'the pantry did not preempt the door');
});

// (0)(v): glassbox. With PANTRY_LOG=debug every probe and its answer, every
// route, and every skip is written; a routed answer names whose computer and
// which lend (AGENTS.md 1).
test('PANTRY_LOG=debug writes each probe, route and skip, and the answer names both', async (t) => {
  const written = [];
  const was = process.stderr.write;
  const before = process.env.PANTRY_LOG;
  process.stderr.write = (chunk) => {
    written.push(String(chunk));
    return true;
  };
  process.env.PANTRY_LOG = 'debug';
  t.after(() => {
    process.stderr.write = was;
    if (before === undefined) delete process.env.PANTRY_LOG;
    else process.env.PANTRY_LOG = before;
  });

  const ana = await modelLend({ id: 'coder' });
  const cy = await silent();
  const record = await group([
    { group: 'g', key: 'k1', name: 'ana-coder', person: 'ana', origin: ana.origin, kind: 'x', limit: null, using: 0 },
    { group: 'g', key: 'k2', name: 'cy-odd', person: 'cy', origin: cy.origin, kind: 'x', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([ana.close(), cy.close(), record.close(), at.close()]));

  await list({ record: record.origin });
  const answer = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'ana/coder', messages: [{ role: 'user', content: 'hi' }] }),
  });
  assert.equal(answer.headers.get('x-pantry-person'), 'ana');
  assert.equal(answer.headers.get('x-pantry-lend'), 'ana-coder');

  const said = written.join('');
  assert.match(said, /probed .*\/v1\/models/);
  assert.match(said, /ana-coder/);
  assert.match(said, /routed ana\/coder/);
  assert.match(said, /ana-coder/);
  assert.match(said, /skipped cy-odd/);
});

// (0)(vi): Authorization and Cookie are never forwarded to a friend's lend.
test('a client\'s Authorization and Cookie never reach a friend\'s lend', async (t) => {
  const ana = await modelLend({ id: 'coder' });
  const record = await group([
    { group: 'g', key: 'k', name: 'ana-coder', person: 'ana', origin: ana.origin, kind: 'x', limit: null, using: 0 },
  ]);
  const at = await pantry({ record: record.origin });
  t.after(() => Promise.all([ana.close(), record.close(), at.close()]));

  await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      authorization: 'Bearer not-for-ana',
      cookie: 'svrngs=not-for-ana',
      'x-keep-me': 'yes',
    },
    body: JSON.stringify({ model: 'ana/coder', messages: [{ role: 'user', content: 'hi' }] }),
  });
  const heard = ana.requests.find((r) => r.path === '/v1/chat/completions');
  assert.equal(heard.headers.authorization, undefined);
  assert.equal(heard.headers.cookie, undefined);
  assert.equal(heard.headers['x-keep-me'], 'yes', 'everything else still passes');
});

// The address a person pasted into their interface stays across a restart.
test('serve answers at the same address across a restart and refuses a held one by name', async (t) => {
  const state = stateDir();
  const ana = await modelLend({ id: 'coder' });
  const record = await group([
    { group: 'g', key: 'k', name: 'ana-coder', person: 'ana', origin: ana.origin, kind: 'x', limit: null, using: 0 },
  ]);
  t.after(() => {
    rmSync(state, { recursive: true, force: true });
    return Promise.all([ana.close(), record.close()]);
  });

  const first = await serve({ record: record.origin, state });
  const port = first.port;
  await first.close();

  const again = await serve({ record: record.origin, state });
  t.after(() => again.close());
  assert.equal(again.port, port, 'the address moved across a restart');
  await again.close();

  const held = net.createServer();
  await new Promise((resolve) => held.listen(port, '127.0.0.1', resolve));
  t.after(() => held.close());
  await assert.rejects(serve({ record: record.origin, state }),
    new RegExp(`holds 127\\.0\\.0\\.1:${port}`), 'silently moved to another port');
  await new Promise((resolve) => held.close(resolve));

  const free = await new Promise((resolve) => {
    const probe = net.createServer();
    probe.listen(0, '127.0.0.1', () => {
      const chosen = probe.address().port;
      probe.close(() => resolve(chosen));
    });
  });
  const named = await serve({ record: record.origin, state, port: free });
  assert.equal(named.port, free, '--port names another');
  await named.close();
  const kept = await serve({ record: record.origin, state });
  t.after(() => kept.close());
  assert.equal(kept.port, free, 'and what --port named stays the address');
});
