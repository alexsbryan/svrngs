#!/usr/bin/env node
// The pantry: one place that shows what your groups lend and one OpenAI-
// compatible address that routes each request to the friend who lends it.
//
//   node apps/pantry/pantry.mjs list
//   node apps/pantry/pantry.mjs serve [--port N]
//
// Both read this computer's record through its data dir's `api` file
// (`$SVRNGS_DIR/api`, else `~/.svrngs/api`). `--record URL` names another.
// The pantry knows a lend by asking it: OpenAI's `/v1/models` is a model;
// MCP's initialize and tools/list over Streamable HTTP is a tool. It keeps
// no kind list, no card and no matcher.

import http from 'node:http';
import { readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { toolsAt } from './mcp.mjs';

const DISCOVERY_TIMEOUT = 4000;

/// Where this computer's record answers, from the data dir's `api` file.
export function recordAt() {
  if (process.env.SVRNGS_API) return process.env.SVRNGS_API.replace(/\/+$/, '');
  const dir = process.env.SVRNGS_DIR || join(homedir(), '.svrngs');
  return readFileSync(join(dir, 'api'), 'utf8').trim().replace(/\/+$/, '');
}

async function get(url) {
  const res = await fetch(url, { signal: AbortSignal.timeout(DISCOVERY_TIMEOUT) });
  if (!res.ok) throw new Error(`${url} answered ${res.status}`);
  return res.json();
}

/// Every lend this computer's groups hold now, and the people whose
/// computer did not say what they lend.
export async function lendsNow(record) {
  const base = record.replace(/\/+$/, '');
  const held = await get(`${base}/v1/groups`);
  const groups = Array.isArray(held.groups) ? held.groups : [];
  const lends = [];
  const unlisted = [];
  const seen = new Set();
  for (const group of groups) {
    const id = group.id;
    const body = await get(`${base}/v1/lends?namespace=${encodeURIComponent(id)}`);
    for (const lend of body.lent || []) {
      const key = `${lend.person}\u0000${lend.name}\u0000${lend.origin}`;
      if (seen.has(key)) continue;
      seen.add(key);
      lends.push({ ...lend, group: id });
    }
    for (const [why, people] of [
      ['asleep', body.unanswered],
      ['not admitted', body.unheard],
      ['refused', body.refused],
      ['older svrngs', body.older],
    ]) {
      for (const person of people || []) unlisted.push({ person, why, group: id });
    }
  }
  return { lends, unlisted };
}

/// What a lend answers when it is asked by protocol: a model, a tool, or
/// neither, with the reasons kept.
export async function discover(origin) {
  const base = origin.replace(/\/+$/, '');
  let why = [];
  try {
    const res = await fetch(`${base}/v1/models`, {
      headers: { accept: 'application/json' },
      signal: AbortSignal.timeout(DISCOVERY_TIMEOUT),
    });
    if (res.ok) {
      const body = await res.json();
      if (Array.isArray(body.data)) {
        const models = body.data
          .filter((m) => m && m.id !== undefined)
          .map((m) => ({ ...m, id: String(m.id) }));
        if (models.length) return { protocol: 'model', models };
        why.push('/v1/models answered no models');
      } else {
        why.push('/v1/models answered no model list');
      }
    } else {
      why.push(`/v1/models answered ${res.status}`);
    }
  } catch (e) {
    why.push(`/v1/models did not answer (${e.message})`);
  }
  try {
    const tools = await toolsAt(base);
    return { protocol: 'tool', tools };
  } catch (e) {
    why.push(`MCP did not answer (${e.message})`);
  }
  return { protocol: 'neither', why: why.join('; ') };
}

/// What your groups lend now, each lend asked by its protocol.
export async function list({ record = recordAt() } = {}) {
  const { lends, unlisted } = await lendsNow(record);
  const found = [];
  for (const lend of lends) {
    const seen = await discover(lend.origin);
    if (seen.protocol === 'model') {
      seen.models = seen.models.map((m) => ({ ...m, as: `${lend.person}/${m.id}` }));
    }
    found.push({
      group: lend.group,
      name: lend.name,
      person: lend.person,
      origin: lend.origin,
      kind: lend.kind,
      limit: lend.limit,
      using: lend.using,
      ...seen,
    });
  }
  return { lends: found, unlisted };
}

export async function formatList({ lends, unlisted }) {
  const lines = ['What your groups lend now:'];
  for (const lend of lends) {
    const who = `${lend.person}'s computer`;
    if (lend.protocol === 'model') {
      const models = lend.models.map((m) => `${lend.person}/${m.id}`).join(', ');
      lines.push(`  ${who} — ${lend.origin} — model ${models}`);
    } else if (lend.protocol === 'tool') {
      const tools = lend.tools.map((t) => `${t.name} (${t.effect})`).join(', ');
      lines.push(`  ${who} — ${lend.origin} — tool ${tools}`);
    } else {
      lines.push(`  ${who} — ${lend.origin} — neither: ${lend.why}`);
    }
  }
  for (const person of unlisted) {
    lines.push(`  ${person.person}'s computer — ${person.why}`);
  }
  return lines.join('\n');
}

/// The models every lend answers now, each id prefixed by its lender.
async function modelsNow(record) {
  const { lends, unlisted } = await lendsNow(record);
  const data = [];
  const index = new Map();
  for (const lend of lends) {
    const seen = await discover(lend.origin).catch((e) => ({
      protocol: 'neither',
      why: e.message,
    }));
    if (seen.protocol !== 'model') continue;
    for (const model of seen.models) {
      const id = `${lend.person}/${model.id}`;
      if (index.has(id)) continue;
      index.set(id, { lend, model: model.id });
      data.push({ ...model, id, object: model.object || 'model', owned_by: id });
    }
  }
  return { data, index, unlisted };
}

function unlistedWhy(unlisted, person) {
  const hit = unlisted.find((u) => u.person === person);
  return hit ? hit.why : null;
}

function fault(res, status, error, extra = {}) {
  res.writeHead(status, { 'content-type': 'application/json' });
  res.end(JSON.stringify({ error, ...extra }));
}

function collect(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => resolve(Buffer.concat(chunks)));
    req.on('error', reject);
  });
}

/// The multipart form's `model`, and a way to write the lend's own id back
/// without touching another byte of the body.
function multipartModel(raw, contentType) {
  const found = /boundary=(?:"([^"]+)"|([^;]+))/.exec(contentType || '');
  if (!found) return null;
  const boundary = (found[1] || found[2]).trim();
  const text = raw.toString('latin1');
  const lower = text.toLowerCase();
  const name = lower.indexOf('name="model"');
  if (name < 0) return null;
  const part = text.lastIndexOf(`--${boundary}`, name);
  if (part < 0) return null;
  const headersEnd = text.indexOf('\r\n\r\n', name);
  if (headersEnd < 0) return null;
  const start = headersEnd + 4;
  const end = text.indexOf(`\r\n--${boundary}`, start);
  if (end < 0) return null;
  return {
    model: text.slice(start, end),
    rewrite(value) {
      return Buffer.from(text.slice(0, start) + value + text.slice(end), 'latin1');
    },
  };
}

/// The lend's own id written back into a JSON or multipart body, and the
/// body to send: nothing else in it changes.
function rewriteModel(raw, contentType, model) {
  if ((contentType || '').includes('multipart/form-data')) {
    const found = multipartModel(raw, contentType);
    if (!found) return null;
    return { body: found.rewrite(model), model: found.model };
  }
  let asked;
  try {
    asked = JSON.parse(raw.toString('utf8'));
  } catch {
    return null;
  }
  if (!asked || typeof asked.model !== 'string') return null;
  return { body: Buffer.from(JSON.stringify({ ...asked, model })), model: asked.model };
}

/// One OpenAI-compatible address on 127.0.0.1, routed by `model`.
export async function serve({ record = recordAt(), port = 0, host = '127.0.0.1' } = {}) {
  const server = http.createServer(async (req, res) => {
    try {
      if (req.method === 'GET' && req.url === '/v1/models') {
        const { data } = await modelsNow(record);
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ object: 'list', data }));
      }
      const raw = await collect(req);
      const asked = rewriteModel(raw, req.headers['content-type'], '');
      if (!asked) {
        return fault(res, 400, 'a request to the pantry names its model');
      }
      const { index, unlisted } = await modelsNow(record);
      const found = index.get(asked.model);
      if (!found) {
        const person = asked.model.split('/')[0];
        const why = unlistedWhy(unlisted, person);
        if (why) {
          return fault(res, 503,
            `${person}'s computer did not answer (${why}), so ${asked.model} cannot be answered`,
            { model: asked.model, why });
        }
        return fault(res, 404, `no lend answers ${asked.model}`, { model: asked.model });
      }
      const { lend, model } = found;
      if (lend.limit && lend.using >= lend.limit) {
        return fault(res, 429,
          `${lend.person}'s lend ${lend.name} is at its limit (${lend.using} of ${lend.limit}), so ${asked.model} cannot be answered`,
          { model: asked.model, lend: lend.name, why: 'at its limit' });
      }
      const written = rewriteModel(raw, req.headers['content-type'], model);
      if (!written) {
        return fault(res, 400, `the body's model cannot be rewritten for ${asked.model}`);
      }
      const upstream = new URL(req.url, lend.origin);
      const headers = { ...req.headers };
      delete headers.host;
      delete headers.connection;
      delete headers['content-length'];
      headers['content-length'] = String(written.body.length);
      const target = http.request({
        hostname: upstream.hostname,
        port: upstream.port,
        path: `${upstream.pathname}${upstream.search}`,
        method: req.method,
        headers,
      }, (answer) => {
        res.writeHead(answer.statusCode, answer.headers);
        answer.pipe(res);
      });
      target.on('error', (e) => {
        if (res.headersSent) return res.destroy(e);
        fault(res, 502,
          `${lend.person}'s lend ${lend.name} did not answer, so ${asked.model} cannot be answered`,
          { model: asked.model, lend: lend.name, why: e.message });
      });
      target.end(written.body);
      return undefined;
    } catch (e) {
      if (!res.headersSent) fault(res, 500, `the pantry could not answer: ${e.message}`);
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

async function main(argv) {
  const [verb, ...rest] = argv;
  const options = {};
  for (let i = 0; i < rest.length; i += 1) {
    if (rest[i] === '--record') options.record = rest[++i];
    else if (rest[i] === '--port') options.port = Number(rest[++i]);
    else throw new Error(`unknown option ${rest[i]}`);
  }
  if (verb === 'list') {
    console.log(await formatList(await list(options)));
    return;
  }
  if (verb === 'serve') {
    const at = await serve(options);
    console.log(`Point your interface at ${at.url}/v1`);
    return;
  }
  console.error('usage: pantry list | pantry serve [--port N] [--record URL]');
  process.exitCode = 2;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main(process.argv.slice(2)).catch((e) => {
    console.error(`pantry: ${e.message}`);
    process.exitCode = 1;
  });
}
