// The vision bridge, the worked example of an adapter (CHARTER.md, "The
// vision bridge is the worked example"; row models-bridge, bar C4): OpenAI
// in, OpenAI out, its one upstream the person's own `pantry serve`. A chat
// request for a model that cannot see arrives with each image swapped for
// the configured vision model's description under the bridge's one fixed
// prompt; `tools` and every other field pass through untouched, the answer
// streams, and both hops are named. A request with no image is untouched.
// The friend's door counts the person running the bridge, never another.

import assert from 'node:assert/strict';
import http from 'node:http';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

import { serve } from '../../pantry.mjs';
import { PROMPT, start } from './server.mjs';

const PANTRY = new URL('../../', import.meta.url).pathname;

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

function forward(req, res, at, raw, headers) {
  const target = http.request({
    hostname: at.hostname, port: at.port, path: `${at.pathname}${at.search}`,
    method: req.method, headers,
  }, (answer) => {
    res.writeHead(answer.statusCode, answer.headers);
    answer.pipe(res);
  });
  target.on('error', (e) => json(res, 502, { error: e.message }));
  target.end(raw);
}

async function standin(handler) {
  const requests = [];
  const server = http.createServer(async (req, res) => {
    const raw = await body(req);
    const seen = { method: req.method, path: req.url, headers: req.headers, body: raw };
    requests.push(seen);
    try {
      await handler(req, res, seen);
    } catch (e) {
      json(res, 500, { error: String(e) });
    }
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  return {
    origin: `http://127.0.0.1:${server.address().port}/`,
    requests,
    close: () => new Promise((resolve) => {
      server.close(resolve);
      server.closeAllConnections?.();
    }),
  };
}

/// A friend's model, behind the door on the person's computer. The door is
/// the only way from that pantry to this lend, and it names the caller from
/// the connection, so a forged identity header never reaches the friend.
async function lend(id, { sees = true } = {}) {
  const friend = await standin(async (req, res, seen) => {
    if (req.url === '/v1/models') return json(res, 200, { object: 'list', data: [{ id, object: 'model' }] });
    if (req.url !== '/v1/chat/completions') return json(res, 404, { error: 'not me' });
    const asked = JSON.parse(seen.body);
    const images = (asked.messages || []).flatMap((m) => (Array.isArray(m.content) ? m.content : []))
      .filter((p) => p && p.type === 'image_url');
    if (images.length && !sees) return json(res, 400, { error: `${id} cannot see images` });
    if (asked.stream) {
      res.writeHead(200, { 'content-type': 'text/event-stream' });
      res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'first' } }] })}\n\n`);
      await new Promise((r) => setTimeout(r, 150));
      seen.secondSent = true;
      res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'second' } }] })}\n\n`);
      return res.end('data: [DONE]\n\n');
    }
    const content = images.length
      ? images.map((p) => `seen ${p.image_url.url}`).join('; ')
      : 'ok';
    return json(res, 200, {
      object: 'chat.completion', model: asked.model,
      choices: [{ index: 0, message: { role: 'assistant', content } }],
    });
  });
  const door = await standin((req, res, seen) => {
    const headers = { ...req.headers };
    delete headers.host;
    delete headers.connection;
    delete headers['content-length'];
    delete headers['x-svrngs-person'];
    headers['x-svrngs-person'] = 'bo';
    headers['content-length'] = String(seen.body.length);
    forward(req, res, new URL(seen.path, friend.origin), seen.body, headers);
  });
  return {
    origin: door.origin,
    requests: friend.requests,
    close: () => Promise.all([friend.close(), door.close()]),
  };
}

function record(lends) {
  return standin((req, res) => {
    if (req.url === '/v1/groups') return json(res, 200, { groups: [{ id: 'g', name: 'g' }] });
    if (req.url === '/v1/lends?namespace=g') {
      return json(res, 200, {
        namespace: 'g',
        lent: lends.map((l) => ({ ...l, kind: 'anything', limit: null, using: 0 })),
        unanswered: [], unheard: [], refused: [], older: [],
      });
    }
    json(res, 404, { error: 'not me' });
  });
}

function hash(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

async function place(t, lends) {
  const held = await record(lends);
  const pantry = await serve({ record: held.origin, host: '127.0.0.1', port: 0 });
  t.after(() => Promise.all([held.close(), pantry.close()]));
  return pantry;
}

test('a screenshot reaches a blind model as the vision model\'s description, tools intact, both hops named', async (t) => {
  const coder = await lend('coder', { sees: false });
  const vision = await lend('vision');
  t.after(() => Promise.all([coder.close(), vision.close()]));
  const pantry = await place(t, [
    { group: 'g', name: 'ana-coder', person: 'ana', origin: coder.origin },
    { group: 'g', name: 'ana-vision', person: 'ana', origin: vision.origin },
  ]);
  const bridge = await start({ upstream: pantry.url, vision: 'ana/vision', host: '127.0.0.1', port: 0 });
  t.after(() => bridge.close());

  const tools = [{ type: 'function', function: { name: 'weather', parameters: { type: 'object' } } }];
  const images = ['data:image/png;base64,AAA', 'data:image/png;base64,BBB'];
  const request = {
    model: 'ana/coder', stream: true, temperature: 0.2, tools, tool_choice: 'auto',
    messages: [{
      role: 'user',
      content: [{ type: 'text', text: 'fix this page' }, ...images.map((url) => ({ type: 'image_url', image_url: { url } }))],
    }],
  };
  const res = await fetch(`${bridge.url}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'content-type': 'application/json', 'x-svrngs-person': 'ana' },
    body: JSON.stringify(request),
  });
  assert.equal(res.status, 200);
  assert.equal(res.headers.get('x-pantry-model'), 'ana/coder');
  assert.equal(res.headers.get('x-pantry-bridged-by'), 'ana/vision');

  const reader = res.body.getReader();
  const first = await reader.read();
  assert.match(Buffer.from(first.value).toString(), /first/);
  const heard = coder.requests.find((r) => r.path === '/v1/chat/completions');
  assert.equal(heard.secondSent, undefined, 'the first chunk arrived before the lender sent the second');
  await reader.cancel();

  const asked = JSON.parse(heard.body);
  assert.equal(asked.model, 'coder', 'the pantry wrote the lend its own id');
  assert.equal(asked.temperature, 0.2);
  assert.deepEqual(asked.tools, tools, 'tools pass through untouched');
  assert.equal(asked.tool_choice, 'auto');
  const parts = asked.messages[0].content;
  assert.equal(parts[0].text, 'fix this page');
  assert.deepEqual(parts.slice(1), images.map((url) => ({ type: 'text', text: `seen ${url}` })),
    'each image is swapped for its own description');
  assert.ok(!JSON.stringify(asked).includes('image_url'), 'the blind model is sent no image');

  const describes = vision.requests.filter((r) => r.path === '/v1/chat/completions');
  assert.equal(describes.length, images.length, 'one describe per image');
  const described = JSON.parse(describes[0].body);
  assert.equal(described.model, 'vision');
  assert.equal(described.stream, false);
  assert.deepEqual(described.messages[0].content[0], { type: 'text', text: PROMPT });
  assert.equal(described.messages[0].content[1].image_url.url, images[0]);

  for (const seen of [...coder.requests, ...vision.requests]) {
    assert.equal(seen.headers['x-svrngs-person'], 'bo',
      'the door counts the person running the bridge, never the name a client claims');
  }
});

test('a request with no image passes through untouched, and the vision model is never asked', async (t) => {
  const coder = await lend('coder', { sees: false });
  const vision = await lend('vision');
  t.after(() => Promise.all([coder.close(), vision.close()]));
  const pantry = await place(t, [
    { group: 'g', name: 'ana-coder', person: 'ana', origin: coder.origin },
    { group: 'g', name: 'ana-vision', person: 'ana', origin: vision.origin },
  ]);
  const bridge = await start({ upstream: pantry.url, vision: 'ana/vision', host: '127.0.0.1', port: 0 });
  t.after(() => bridge.close());

  const tools = [{ type: 'function', function: { name: 'weather', parameters: { type: 'object' } } }];
  const res = await fetch(`${bridge.url}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: 'ana/coder', messages: [{ role: 'user', content: 'hello' }], tools }),
  });
  assert.equal(res.status, 200);
  assert.equal(res.headers.get('x-pantry-model'), null);
  assert.equal(res.headers.get('x-pantry-bridged-by'), null);

  const chat = coder.requests.find((r) => r.path === '/v1/chat/completions');
  const asked = JSON.parse(chat.body);
  assert.deepEqual(asked, { model: 'coder', messages: [{ role: 'user', content: 'hello' }], tools });
  assert.equal(vision.requests.filter((r) => r.path === '/v1/chat/completions').length, 0);
});

test('the pantry\'s own files do not change for the bridge', async (t) => {
  const files = ['pantry.mjs', 'mcp.mjs'].map((f) => `${PANTRY}${f}`);
  const before = files.map(hash);
  const coder = await lend('coder', { sees: false });
  const vision = await lend('vision');
  t.after(() => Promise.all([coder.close(), vision.close()]));
  const pantry = await place(t, [
    { group: 'g', name: 'ana-coder', person: 'ana', origin: coder.origin },
    { group: 'g', name: 'ana-vision', person: 'ana', origin: vision.origin },
  ]);
  const bridge = await start({ upstream: pantry.url, vision: 'ana/vision', host: '127.0.0.1', port: 0 });
  t.after(() => bridge.close());

  await fetch(`${bridge.url}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      model: 'ana/coder',
      messages: [{ role: 'user', content: [{ type: 'image_url', image_url: { url: 'data:image/png;base64,CCC' } }] }],
    }),
  });
  assert.deepEqual(files.map(hash), before, 'the bridge taught the pantry nothing');
});
