// The pantry's minimum (CHARTER.md, "The minimum"; bars C1, C2, C3, C6):
// stand-in people in a group lend stand-in servers, and the pantry finds
// them by asking their protocol, never by a kind it keeps a list of.

import assert from 'node:assert/strict';
import http from 'node:http';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { list, serve } from './pantry.mjs';

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
    close: () => new Promise((resolve) => server.close(resolve)),
  };
}

async function group(lends, unlisted = {}) {
  return standin((req, res) => {
    if (req.url === '/v1/groups') {
      return json(res, 200, { groups: [...new Set(lends.map((l) => l.group))].map((id) => ({ id, name: id })) });
    }
    const at = new URL(req.url, 'http://record');
    if (at.pathname === '/v1/lends') {
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
  const full = await modelLend({ id: 'full' });
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
  assert.match(limitedBody, /limit/);
  assert.equal(full.requests.filter((r) => r.path === '/v1/chat/completions').length, 0);

  const unknown = await fetch(`${at.url}/v1/chat/completions`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'nobody/none', messages: [{ role: 'user', content: 'hi' }] }),
  });
  assert.equal(unknown.status, 404);
  assert.match(await unknown.text(), /nobody\/none/);
  assert.equal(sam.requests.filter((r) => r.path === '/v1/chat/completions').length, 0);
});
