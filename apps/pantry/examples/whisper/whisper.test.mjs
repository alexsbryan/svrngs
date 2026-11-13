// The transcription recipe (CHARTER.md, "Speech to text: a small wrapper
// recipe"): an OpenAI-in, OpenAI-out server in front of whisper.cpp's
// server, so a lend that only knows /inference is still a model by asking.
// The pantry learns nothing of it. Each server is a handler in this
// process (../../in-process.mjs), asked with no listener.

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

import { fetch, request, serveAt } from '../../in-process.mjs';
import { list } from '../../pantry.mjs';
import { wrapper } from './server.mjs';

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

function whisper() {
  return serveAt(async (req, res) => {
    const raw = await body(req);
    if (req.url !== '/inference') return json(res, 404, { error: 'not me' });
    const form = await new Response(raw, { headers: req.headers }).formData();
    return json(res, 200, { text: await form.get('file').text() });
  });
}

/** The wrapper in front of `upstream`, at an origin of its own, as `start` gives it its listener. */
function start({ upstream, model }) {
  const at = serveAt(wrapper({ upstream, model, request }));
  return { url: at.origin.replace(/\/$/, ''), close: at.close };
}

function record(origin) {
  return serveAt((req, res) => {
    if (req.url === '/v1/groups') return json(res, 200, { groups: [{ id: 'g', name: 'g' }] });
    if (req.url === '/v1/lends?namespace=g') {
      return json(res, 200, {
        namespace: 'g',
        lent: [
          { name: 'sam-stt', person: 'sam', origin, kind: 'whisper-server', limit: null, using: 0 },
        ],
        unanswered: [],
        unheard: [],
        refused: [],
        older: [],
      });
    }
    json(res, 404, { error: 'not me' });
  });
}

function hash(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

test('the wrapper answers /v1/models and OpenAI transcription in front of whisper.cpp', async (t) => {
  const upstream = whisper();
  const at = start({ upstream: upstream.origin, model: 'whisper-1' });
  t.after(() => Promise.all([upstream.close(), at.close()]));

  const models = await (await fetch(`${at.url}/v1/models`)).json();
  assert.deepEqual(
    models.data.map((m) => m.id),
    ['whisper-1'],
  );

  const form = new FormData();
  form.set('model', 'whisper-1');
  form.set('language', 'en');
  form.set(
    'file',
    new Blob([Buffer.from('the quick brown fox')], { type: 'audio/wav' }),
    'say.wav',
  );
  const answer = await (
    await fetch(`${at.url}/v1/audio/transcriptions`, { method: 'POST', body: form })
  ).json();
  assert.deepEqual(answer, { text: 'the quick brown fox' });
});

test('a pantry lists and routes the wrapper as it is, and its own files do not change', async (t) => {
  const upstream = whisper();
  const at = start({ upstream: upstream.origin, model: 'whisper-1' });
  const held = record(at.url + '/');
  t.after(() => Promise.all([upstream.close(), at.close(), held.close()]));

  const files = ['pantry.mjs', 'mcp.mjs'].map((f) => `${PANTRY}${f}`);
  const before = files.map(hash);
  const found = await list({ record: held.origin, fetch });
  assert.equal(found.lends[0].protocol, 'model');
  assert.deepEqual(
    found.lends[0].models.map((m) => m.as),
    ['sam/whisper-1'],
  );
  assert.deepEqual(files.map(hash), before, 'the wrapper taught the pantry nothing');
});
