// The transcription recipe (CHARTER.md, "Speech to text: a small wrapper
// recipe"): an OpenAI-in, OpenAI-out server in front of whisper.cpp's
// server, so a lend that only knows /inference is still a model by asking.
// The pantry learns nothing of it.

import assert from 'node:assert/strict';
import http from 'node:http';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

import { list } from '../../pantry.mjs';
import { start } from './server.mjs';

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

async function whisper() {
  const server = http.createServer(async (req, res) => {
    const raw = await body(req);
    if (req.url !== '/inference') return json(res, 404, { error: 'not me' });
    const form = await new Response(raw, { headers: req.headers }).formData();
    return json(res, 200, { text: await form.get('file').text() });
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  return {
    origin: `http://127.0.0.1:${server.address().port}/`,
    close: () => new Promise((resolve) => {
      server.close(resolve);
      server.closeAllConnections?.();
    }),
  };
}

function record(origin) {
  const server = http.createServer((req, res) => {
    if (req.url === '/v1/groups') return json(res, 200, { groups: [{ id: 'g', name: 'g' }] });
    if (req.url === '/v1/lends?namespace=g') {
      return json(res, 200, {
        namespace: 'g',
        lent: [{ name: 'sam-stt', person: 'sam', origin, kind: 'whisper-server', limit: null, using: 0 }],
        unanswered: [], unheard: [], refused: [], older: [],
      });
    }
    json(res, 404, { error: 'not me' });
  });
  return new Promise((resolve) => server.listen(0, '127.0.0.1', () => resolve({
    origin: `http://127.0.0.1:${server.address().port}/`,
    close: () => new Promise((done) => {
      server.close(done);
      server.closeAllConnections?.();
    }),
  })));
}

function hash(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

test('the wrapper answers /v1/models and OpenAI transcription in front of whisper.cpp', async (t) => {
  const upstream = await whisper();
  const at = await start({ upstream: upstream.origin, port: 0, host: '127.0.0.1', model: 'whisper-1' });
  t.after(() => Promise.all([upstream.close(), at.close()]));

  const models = await (await fetch(`${at.url}/v1/models`)).json();
  assert.deepEqual(models.data.map((m) => m.id), ['whisper-1']);

  const form = new FormData();
  form.set('model', 'whisper-1');
  form.set('language', 'en');
  form.set('file', new Blob([Buffer.from('the quick brown fox')], { type: 'audio/wav' }), 'say.wav');
  const answer = await (await fetch(`${at.url}/v1/audio/transcriptions`, { method: 'POST', body: form })).json();
  assert.deepEqual(answer, { text: 'the quick brown fox' });
});

test('a pantry lists and routes the wrapper as it is, and its own files do not change', async (t) => {
  const upstream = await whisper();
  const at = await start({ upstream: upstream.origin, port: 0, host: '127.0.0.1', model: 'whisper-1' });
  const held = await record(at.url + '/');
  t.after(() => Promise.all([upstream.close(), at.close(), held.close()]));

  const files = ['pantry.mjs', 'mcp.mjs'].map((f) => `${PANTRY}${f}`);
  const before = files.map(hash);
  const found = await list({ record: held.origin });
  assert.equal(found.lends[0].protocol, 'model');
  assert.deepEqual(found.lends[0].models.map((m) => m.as), ['sam/whisper-1']);
  assert.deepEqual(files.map(hash), before, 'the wrapper taught the pantry nothing');
});
