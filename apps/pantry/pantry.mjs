#!/usr/bin/env node
// The pantry: one place that shows what your groups lend and one OpenAI-
// compatible address that routes each request to the friend who lends it.
//
//   node apps/pantry/pantry.mjs list
//   node apps/pantry/pantry.mjs serve [--port N]
//
// Both read this computer's record through its data dir's `api` file
// (`$SVRNGS_DIR/api`, else `~/.svrngs/api`), asking with the token in the
// `token` file beside it (give-token). `--record URL` names another.
// The pantry knows a lend by asking it: OpenAI's `/v1/models` is a model;
// MCP's initialize and tools/list over Streamable HTTP is a tool. It keeps
// no kind list, no card and no matcher.

import http from 'node:http';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { toolsAt } from './mcp.mjs';

/// How long what discovery found is kept in this process: a friend's computer
/// that did not answer is not dialled again on every message, and what it said
/// is said with its age.
export const FOUND_FOR = 4000;

/// How long the record's `/v1/lends` answer is kept: its fan-out waits up to
/// `ANSWER_WITHIN` (3 s) on each friend (`crates/svrngs-node/src/node.rs:109`),
/// the asleep one's dial included, so a friend whose computer is asleep costs
/// one ask per half minute, not one per message. Named once, and apart from
/// how long a lend's answer is kept.
export const LENDS_FOR = 30000;

/// How long one probe waits for a lend's `/v1/models` or MCP answer:
/// `crates/svrngs-node/src/dial.rs:30`'s 4 s, the slowest working dial on this
/// host times 3. Its own constant, not how long an answer is kept.
const DISCOVERY_TIMEOUT = 4000;

/// This computer's data dir: where its record answers and where the pantry
/// keeps the address a person pasted into their interface.
function dataDir(option) {
  return option || process.env.SVRNGS_DIR || join(homedir(), '.svrngs');
}

/// Where this computer's record answers, from the data dir's `api` file.
export function recordAt() {
  if (process.env.SVRNGS_API) return process.env.SVRNGS_API.replace(/\/+$/, '');
  return readFileSync(join(dataDir(), 'api'), 'utf8').trim().replace(/\/+$/, '');
}

/// The token this computer's record asks of a program, from the data dir's
/// `token` file; none where there is no such file, which the record refuses
/// naming the file.
function tokenAt() {
  try {
    return readFileSync(join(dataDir(), 'token'), 'utf8').trim();
  } catch {
    return '';
  }
}

/// The address the last start took, if one was kept.
function keptPort(dir) {
  try {
    const port = Number(readFileSync(join(dir, 'pantry-port'), 'utf8').trim());
    return Number.isInteger(port) && port > 0 && port < 65536 ? port : null;
  } catch {
    return null;
  }
}

/// Keep the address this start took, so the next one takes it again.
function keepPort(dir, port) {
  if (!port) return;
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, 'pantry-port'), `${port}\n`);
}

/// Glassbox: with PANTRY_LOG=debug, every probe, route and skip is written.
function note(...words) {
  if (process.env.PANTRY_LOG === 'debug') console.error('pantry:', ...words);
}

function secondsAgo(at, now) {
  return Math.max(0, Math.round((now() - at) / 1000));
}

async function get(url, token) {
  const headers = token === undefined ? {} : { authorization: `Bearer ${token}` };
  const res = await fetch(url, { headers, signal: AbortSignal.timeout(DISCOVERY_TIMEOUT) });
  if (!res.ok) throw new Error(`${url} answered ${res.status}`);
  return res.json();
}

/// Every lend this computer's groups hold now, and the people whose
/// computer did not say what they lend. A person's lend is told apart by the
/// key their computer is dialled by, so two people of one name keep two ids.
/// The record's answer is kept for [`LENDS_FOR`] in this process, so a friend
/// asleep is not dialled on every message; `refresh` asks it again before
/// that time.
const keptLends = new Map();

export async function lendsNow(record, { clock = Date.now, refresh = false } = {}) {
  const base = record.at.replace(/\/+$/, '');
  const now = clock();
  const kept = keptLends.get(base);
  if (!refresh && kept && now - kept.at < LENDS_FOR) {
    note('kept lends', base, `looked ${secondsAgo(kept.at, clock)}s ago`);
    return { ...kept.seen, at: kept.at, asked: false };
  }
  const held = await get(`${base}/v1/groups`, record.token);
  const groups = Array.isArray(held.groups) ? held.groups : [];
  const lends = [];
  const unlisted = [];
  const seen = new Set();
  for (const group of groups) {
    const id = group.id;
    const body = await get(`${base}/v1/lends?namespace=${encodeURIComponent(id)}`, record.token);
    for (const lend of body.lent || []) {
      const key = `${lend.person}\u0000${lend.key}\u0000${lend.name}\u0000${lend.origin}`;
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
  const found = { lends: handles(lends), unlisted };
  keptLends.set(base, { at: now, seen: found });
  note('asked lends', base, `${found.lends.length} lend, ${unlisted.length} unlisted`);
  return { ...found, at: now, asked: true };
}

/// Whether the record names this prefix at all: a lend's handle, or a person
/// whose computer did not answer. A prefix it does not name sends a request
/// back to the record, in case a lend has appeared since.
function namesLender({ lends, unlisted }, prefix) {
  return lends.some((lend) => lend.handle === prefix)
    || unlisted.some((person) => person.person === prefix);
}

/// What the record says, from what is kept when it is fresh and names the
/// prefix; otherwise asked now. A prefix it kept but does not name is asked
/// again, so a lend that appeared since is found.
async function lendsFor(record, clock, prefix) {
  const seen = await lendsNow(record, { clock });
  if (prefix === undefined || seen.asked || namesLender(seen, prefix)) return seen;
  note('asked lends again', record.at, `does not name ${prefix}`);
  return lendsNow(record, { clock, refresh: true });
}

/// A readable handle for each lend's person, unique by the key their
/// computer is dialled by: the name alone when it is one person's, else the
/// name and as much of the key as tells them apart. Never one dropped.
function handles(lends) {
  const byName = new Map();
  for (const lend of lends) {
    if (!byName.has(lend.person)) byName.set(lend.person, new Set());
    byName.get(lend.person).add(lend.key || '');
  }
  for (const lend of lends) {
    const keys = [...byName.get(lend.person)];
    if (keys.length === 1) {
      lend.handle = lend.person;
      continue;
    }
    const mine = lend.key || lend.name;
    let width = 8;
    while (width < mine.length
      && keys.filter((other) => other.slice(0, width) === mine.slice(0, width)).length > 1) {
      width += 4;
    }
    lend.handle = `${lend.person}~${mine.slice(0, width)}`;
  }
  return lends;
}

/// What a lend answers when it is asked by protocol: a model, a tool, or
/// neither, with the reasons kept. Its origin on this computer answers a
/// program only with this computer's `token`, which svrngs takes off before
/// the lend's service is asked (give-session).
async function probe(origin, token) {
  const base = origin.replace(/\/+$/, '');
  const why = [];
  let couldNotJudge = false;
  const timedOut = (e) => e?.name === 'TimeoutError' || e?.name === 'AbortError'
    || /abort|timeout/i.test(e?.message || '');
  try {
    const res = await fetch(`${base}/v1/models`, {
      headers: { accept: 'application/json', ...(token && { authorization: `Bearer ${token}` }) },
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
    couldNotJudge = couldNotJudge || timedOut(e);
  }
  try {
    const tools = await toolsAt(base, DISCOVERY_TIMEOUT, token);
    return { protocol: 'tool', tools };
  } catch (e) {
    why.push(`MCP did not answer (${e.message})`);
    couldNotJudge = couldNotJudge || timedOut(e);
  }
  return { protocol: 'neither', why: why.join('; '), couldNotJudge };
}

/// What discovery found, kept for [`FOUND_FOR`] in this process: a friend
/// who is asleep is not dialled on every message. Its age is said by `list`
/// and in each refusal.
const discovered = new Map();

export async function discover(origin, clock = Date.now, token = undefined) {
  const now = clock();
  const kept = discovered.get(origin);
  if (kept && now - kept.at < FOUND_FOR) {
    note('kept', origin, `looked ${secondsAgo(kept.at, clock)}s ago`, kept.seen.protocol);
    return { ...kept.seen, at: kept.at };
  }
  const seen = await probe(origin, token);
  discovered.set(origin, { at: now, seen });
  note('probed', origin, '→', seen.protocol,
    seen.models ? `${seen.models.length} model` : seen.tools ? `${seen.tools.length} tool` : seen.why);
  return { ...seen, at: now };
}

/// The id a lend's model is listed and routed by: its lender's handle and
/// the model's own id, and the lend's name between them when another lend
/// of that person's already took it. The one home of the form `list` shows
/// and `serve` routes by.
function modelId(lend, model, taken) {
  const id = `${lend.handle}/${model}`;
  return taken.has(id) ? `${lend.handle}/${lend.name}/${model}` : id;
}

/// What your groups lend now, each lend asked by its protocol.
export async function list({ record = recordAt(), token = tokenAt(), clock = Date.now } = {}) {
  const { lends, unlisted, at } = await lendsNow({ at: record, token }, { clock });
  const found = [];
  const used = new Set();
  for (const lend of lends) {
    const seen = await discover(lend.origin, clock, token);
    if (seen.protocol === 'model') {
      seen.models = seen.models.map((m) => {
        const id = modelId(lend, m.id, used);
        used.add(id);
        return { ...m, as: id };
      });
    } else {
      note('skipped', lend.name, seen.why || seen.protocol);
    }
    found.push({
      group: lend.group,
      key: lend.key,
      name: lend.name,
      person: lend.person,
      handle: lend.handle,
      origin: lend.origin,
      kind: lend.kind,
      limit: lend.limit,
      using: lend.using,
      ...seen,
      looked: secondsAgo(seen.at, clock),
    });
  }
  return { lends: found, unlisted, looked: secondsAgo(at, clock) };
}

export async function formatList({ lends, unlisted, looked: lendsLooked }) {
  const lines = [lendsLooked === undefined
    ? 'What your groups lend now:'
    : `What your groups lend now (the record looked ${lendsLooked}s ago):`];
  for (const lend of lends) {
    const who = `${lend.person}'s computer`;
    const looked = ` — looked ${lend.looked}s ago`;
    if (lend.protocol === 'model') {
      const models = lend.models.map((m) => m.as).join(', ');
      lines.push(`  ${who} — ${lend.origin} — model ${models}${looked}`);
    } else if (lend.protocol === 'tool') {
      const tools = lend.tools.map((t) => `${t.name} (${t.effect})`).join(', ');
      lines.push(`  ${who} — ${lend.origin} — tool ${tools}${looked}`);
    } else if (lend.couldNotJudge) {
      lines.push(`  ${who} — ${lend.origin} — could-not-judge: ${lend.why}${looked}`);
    } else {
      lines.push(`  ${who} — ${lend.origin} — neither: ${lend.why}${looked}`);
    }
  }
  for (const person of unlisted) {
    lines.push(`  ${person.person}'s computer — ${person.why}`);
  }
  return lines.join('\n');
}

/// The models every lend answers now, each id prefixed by its lender's
/// handle, and why a lend could not be judged. With `prefix`, only that
/// lender's lends are asked, so another friend's asleep computer is not
/// waited on to answer a request that does not name it.
async function modelsNow(record, clock, prefix) {
  const { lends, unlisted, at } = await lendsFor(record, clock, prefix);
  const data = [];
  const index = new Map();
  const unjudged = new Map();
  for (const lend of lends) {
    if (prefix !== undefined && lend.handle !== prefix) continue;
    const seen = await discover(lend.origin, clock, record.token);
    if (seen.protocol !== 'model') {
      if (seen.protocol === 'neither') {
        if (!unjudged.has(lend.handle)) unjudged.set(lend.handle, []);
        unjudged.get(lend.handle).push({ lend, why: seen.why, at: seen.at });
      }
      note('skipped', lend.name, seen.why || seen.protocol);
      continue;
    }
    for (const model of seen.models) {
      const id = modelId(lend, model.id, index);
      if (index.has(id)) continue;
      index.set(id, { lend, model: model.id, probed: seen.at });
      data.push({ ...model, id, object: model.object || 'model', owned_by: id });
    }
  }
  return { data, index, unjudged, unlisted, at };
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

/// One OpenAI-compatible address on 127.0.0.1, routed by `model`. Without
/// `--port` it takes the address it took last time, from the data dir's
/// `pantry-port`, so the address a person pasted into their interface stays;
/// another program holding it is said by name, never moved past in silence.
export async function serve({ record: at = recordAt(), token = tokenAt(), port, host = '127.0.0.1', state, clock = Date.now } = {}) {
  const record = { at, token };
  const dir = dataDir(state);
  const wanted = port === undefined ? (keptPort(dir) ?? 0) : port;
  const server = http.createServer(async (req, res) => {
    try {
      if (req.method === 'GET' && req.url === '/v1/models') {
        const { data, at } = await modelsNow(record, clock);
        res.writeHead(200, {
          'content-type': 'application/json',
          'x-pantry-looked': `${secondsAgo(at, clock)}s`,
        });
        return res.end(JSON.stringify({ object: 'list', data }));
      }
      const raw = await collect(req);
      const asked = rewriteModel(raw, req.headers['content-type'], '');
      if (!asked) {
        return fault(res, 400, 'a request to the pantry names its model');
      }
      const prefix = asked.model.split('/')[0];
      const { index, unjudged, unlisted, at } = await modelsNow(record, clock, prefix);
      const found = index.get(asked.model);
      if (!found) {
        const missed = unjudged.get(prefix);
        if (missed?.length) {
          const { lend, why, at: probed } = missed[0];
          return fault(res, 503,
            `${lend.person}'s lend ${lend.name} could not be judged (${why}); it was looked ${secondsAgo(probed, clock)}s ago, so ${asked.model} cannot be answered`,
            { model: asked.model, lend: lend.name, why, looked: secondsAgo(probed, clock) });
        }
        const why = unlistedWhy(unlisted, prefix);
        const looked = secondsAgo(at, clock);
        if (why) {
          return fault(res, 503,
            `${prefix}'s computer did not answer (${why}) as last looked ${looked}s ago, so ${asked.model} cannot be answered`,
            { model: asked.model, why, looked });
        }
        return fault(res, 404, `no lend answers ${asked.model} (the record looked ${looked}s ago)`,
          { model: asked.model, looked });
      }
      const { lend, model, probed } = found;
      const written = rewriteModel(raw, req.headers['content-type'], model);
      if (!written) {
        return fault(res, 400, `the body's model cannot be rewritten for ${asked.model}`);
      }
      const upstream = new URL(req.url, lend.origin);
      const headers = { ...req.headers };
      delete headers.host;
      delete headers.connection;
      delete headers['content-length'];
      // A friend's lend never receives the person's own credentials. This
      // computer's token lets pantry in at the lend's origin, and svrngs
      // takes it off before the lend's service is asked (give-session).
      delete headers.authorization;
      delete headers.cookie;
      if (record.token) headers.authorization = `Bearer ${record.token}`;
      headers['content-length'] = String(written.body.length);
      note('routed', asked.model, '→', lend.handle, lend.name, lend.origin);
      const target = http.request({
        hostname: upstream.hostname,
        port: upstream.port,
        path: `${upstream.pathname}${upstream.search}`,
        method: req.method,
        headers,
      }, (answer) => {
        res.writeHead(answer.statusCode, {
          ...answer.headers,
          'x-pantry-person': lend.handle,
          'x-pantry-lend': lend.name,
          'x-pantry-looked': `${secondsAgo(at, clock)}s`,
          'x-pantry-probed': `${secondsAgo(probed, clock)}s`,
        });
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
    server.once('error', (e) => {
      if (e.code === 'EADDRINUSE') {
        reject(new Error(`Another program on this computer holds 127.0.0.1:${wanted}; pantry serve stops rather than move the address`));
      } else {
        reject(e);
      }
    });
    server.listen(wanted, host, resolve);
  });
  const actual = server.address().port;
  if (port === undefined || port > 0) keepPort(dir, actual);
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
    else if (rest[i] === '--state') options.state = rest[++i];
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
  console.error('usage: pantry list | pantry serve [--port N] [--record URL] [--state DIR]');
  process.exitCode = 2;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main(process.argv.slice(2)).catch((e) => {
    console.error(`pantry: ${e.message}`);
    process.exitCode = 1;
  });
}
