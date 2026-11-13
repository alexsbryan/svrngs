#!/usr/bin/env node
// A transcription recipe (CHARTER.md, "Speech to text: a small wrapper
// recipe"): OpenAI in, OpenAI out, in front of whisper.cpp's server. It
// answers `/v1/models` and a multipart `/v1/audio/transcriptions`, so the
// pantry finds it by asking, and learns nothing of it.
//
//   whisper-server -m ggml-base.en.bin --host 127.0.0.1 --port 8106 -l en
//   node server.mjs --upstream http://127.0.0.1:8106 [--port 8100]

import http from 'node:http';

function collect(req) {
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

/// The wrapper's answer to one request: `upstream` is whisper.cpp's server,
/// `model` the id it is listed by, and `request` how whisper.cpp is asked,
/// node's `http.request` when it listens.
export function wrapper({ upstream, model = 'whisper-1', request = http.request } = {}) {
  if (!upstream) throw new Error("the wrapper needs whisper.cpp's server as --upstream");
  return async (req, res) => {
    if (req.method === 'GET' && req.url === '/v1/models') {
      return json(res, 200, { object: 'list', data: [{ id: model, object: 'model' }] });
    }
    if (req.method !== 'POST' || req.url !== '/v1/audio/transcriptions') {
      return json(res, 404, {
        error: 'the wrapper answers /v1/models and /v1/audio/transcriptions',
      });
    }
    const body = await collect(req);
    const at = new URL('/inference', upstream);
    const answer = request(
      {
        hostname: at.hostname,
        port: at.port,
        path: at.pathname,
        method: 'POST',
        headers: { 'content-type': req.headers['content-type'], 'content-length': body.length },
      },
      (heard) => {
        const chunks = [];
        heard.on('data', (c) => chunks.push(c));
        heard.on('end', () => {
          try {
            const said = JSON.parse(Buffer.concat(chunks).toString('utf8'));
            if (typeof said.text !== 'string') throw new Error('whisper.cpp answered no text');
            json(res, 200, { text: said.text });
          } catch (e) {
            json(res, 502, { error: `whisper.cpp did not answer text: ${e.message}` });
          }
        });
      },
    );
    answer.on('error', (e) =>
      json(res, 502, { error: `whisper.cpp did not answer: ${e.message}` }),
    );
    answer.end(body);
    return undefined;
  };
}

export async function start({
  upstream,
  port = 8100,
  host = '127.0.0.1',
  model = 'whisper-1',
} = {}) {
  const server = http.createServer(wrapper({ upstream, model }));
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, host, resolve);
  });
  const actual = server.address().port;
  return {
    port: actual,
    url: `http://${host}:${actual}`,
    close: () =>
      new Promise((resolve) => {
        server.close(resolve);
        server.closeAllConnections?.();
      }),
  };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const arg = (name, fallback) => {
    const i = process.argv.indexOf(name);
    return i > 0 ? process.argv[i + 1] : fallback;
  };
  start({
    upstream: arg('--upstream'),
    port: Number(arg('--port', '8100')),
    model: arg('--model', 'whisper-1'),
  })
    .then((at) =>
      console.log(
        `lend ${at.url} with: svrngs share ${at.port} --program whisper-server --name stt --group <group>`,
      ),
    )
    .catch((e) => {
      console.error(`whisper wrapper: ${e.message}`);
      process.exitCode = 1;
    });
}
