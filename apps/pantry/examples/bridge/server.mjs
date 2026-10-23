#!/usr/bin/env node
// The vision bridge, the worked example of an adapter (CHARTER.md, "The
// vision bridge is the worked example"; bar C4): OpenAI in, OpenAI out, in
// front of the person's own `pantry serve`. A model that cannot see still
// gets the screenshot: each image is described by the vision model you name,
// and the description takes the image's place under this one fixed prompt.
// Every other field, `tools` included, passes through untouched. It runs on
// your computer and reaches every friend through your pantry, as you.
//
//   pantry serve --port 8787
//   node server.mjs --upstream http://127.0.0.1:8787 --vision ana/vision [--port 8120]

import http from 'node:http';

export const PROMPT = 'You are describing a screenshot of a web page to a '
  + 'programmer who cannot see it. Describe the layout, every visible string, '
  + 'the element hierarchy, and any visual defects: clipping, overlap, '
  + 'cut-off content, or errors. Name the exact element and what is wrong '
  + 'with it. Be concrete.';

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

/// One request to the person's pantry, its answer streamed back as it came.
function relay(req, res, base, raw, extra = {}) {
  const at = new URL(req.url, base);
  const headers = { ...req.headers };
  delete headers.host;
  delete headers.connection;
  delete headers['content-length'];
  headers['content-length'] = String(raw.length);
  const target = http.request({
    hostname: at.hostname, port: at.port, path: `${at.pathname}${at.search}`,
    method: req.method, headers,
  }, (answer) => {
    res.writeHead(answer.statusCode, { ...answer.headers, ...extra });
    answer.pipe(res);
  });
  target.on('error', (e) => {
    if (res.headersSent) return res.destroy(e);
    json(res, 502, { error: `the pantry did not answer: ${e.message}` });
  });
  target.end(raw);
}

/// The vision model's description of one image, asked through the pantry.
async function describe(base, vision, part) {
  const res = await fetch(`${base}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      model: vision,
      stream: false,
      messages: [{ role: 'user', content: [{ type: 'text', text: PROMPT }, part] }],
    }),
  });
  const said = await res.json().catch(() => null);
  if (!res.ok) {
    const why = typeof said?.error === 'string' ? said.error : said?.error?.message;
    throw new Error(why || `the pantry answered ${res.status}`);
  }
  const text = said?.choices?.[0]?.message?.content;
  if (typeof text !== 'string' || !text.trim()) throw new Error('the vision model answered no description');
  return text;
}

/// Every message as it came, each image part replaced by its description.
async function withSight(messages, look) {
  const out = [];
  for (const message of messages) {
    if (!Array.isArray(message?.content)) {
      out.push(message);
      continue;
    }
    const content = [];
    for (const part of message.content) {
      content.push(part?.type === 'image_url' ? { type: 'text', text: await look(part) } : part);
    }
    out.push({ ...message, content });
  }
  return out;
}

const imageParts = (messages) => (Array.isArray(messages) ? messages : [])
  .flatMap((m) => (Array.isArray(m?.content) ? m.content : []))
  .filter((part) => part?.type === 'image_url');

export async function start({ upstream, vision, port = 8120, host = '127.0.0.1' } = {}) {
  if (!upstream) throw new Error('the bridge needs the person\'s pantry as --upstream');
  if (!vision) throw new Error('the bridge needs the seeing model as --vision');
  const base = upstream.replace(/\/+$/, '');
  const server = http.createServer(async (req, res) => {
    try {
      const raw = await collect(req);
      let asked = null;
      if (req.method === 'POST' && req.url.split('?')[0] === '/v1/chat/completions') {
        try {
          asked = JSON.parse(raw.toString('utf8'));
        } catch {
          asked = null;
        }
      }
      const images = imageParts(asked?.messages);
      if (typeof asked?.model !== 'string' || !images.length) {
        return relay(req, res, base, raw);
      }
      let seen;
      try {
        seen = { ...asked, messages: await withSight(asked.messages, (part) => describe(base, vision, part)) };
      } catch (e) {
        return json(res, 502, {
          error: `the bridge could not describe an image with ${vision}: ${e.message}`,
          model: asked.model,
          bridged_by: vision,
        });
      }
      return relay(req, res, base, Buffer.from(JSON.stringify(seen)), {
        'x-pantry-model': asked.model,
        'x-pantry-bridged-by': vision,
      });
    } catch (e) {
      if (!res.headersSent) json(res, 500, { error: `the bridge could not answer: ${e.message}` });
      else res.destroy(e);
      return undefined;
    }
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, host, resolve);
  });
  const actual = server.address().port;
  return {
    port: actual,
    url: `http://${host}:${actual}`,
    close: () => new Promise((resolve) => {
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
  const vision = arg('--vision');
  start({ upstream: arg('--upstream'), vision, port: Number(arg('--port', '8120')) })
    .then((at) => console.log(`Point your interface at ${at.url}/v1 — screenshots are described by ${vision} through your pantry`))
    .catch((e) => {
      console.error(`bridge: ${e.message}`);
      process.exitCode = 1;
    });
}
