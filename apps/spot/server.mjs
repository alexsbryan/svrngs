// SPDX-License-Identifier: AGPL-3.0-or-later
// Spot's server: a provider's models, lent to your group for a while on your
// account, chatted with from a phone or a computer.
//
//   node apps/spot/server.mjs --port <port> --provider <its OpenAI address>
//     --model <name>=<the provider's id> [--model ...] [--until HH:MM]
//     [--budget <tokens>]
//
// An app that runs, and asks svrngs nothing. It holds no key: you lend its
// port, and give your key as the lend's Authorization header. svrngs keeps
// it, and its door adds it to each request in place of any a friend sends
// (crates/svrngs-node/src/door.rs `rewrite_headers`). The server forwards
// that one, with content-type and accept and nothing else: no x-svrngs-*, no
// cookie, nobody's name.
//
// It serves its chat page (page.html, page.js, markdown.js), what the page
// shows (/spot.json), the names it lends (GET /v1/models) and a chat with one
// of them (POST /v1/chat/completions), each name turned into the provider's
// id. It counts the tokens the provider reports per person, by the key the
// door names them by (`x-svrngs-key`), and refuses a chat past `--until` or
// once `--budget` is spent.

import { readFileSync } from 'node:fs';
import http from 'node:http';
import { resolve } from 'node:path';
import { Readable } from 'node:stream';
import { fileURLToPath } from 'node:url';

const n = (x) => x.toLocaleString('en-US');

/** A header svrngs's door set, read as the UTF-8 it wrote (door.rs sets the name's bytes). */
const told = (req, name) => {
  const v = req.headers[name];
  return typeof v === 'string' && v.length ? Buffer.from(v, 'latin1').toString('utf8') : null;
};

/** `HH:MM` as its next time on this computer's clock, or a date and time; none for none. */
export function endOf(until, now = new Date()) {
  if (until == null) return null;
  const hm = String(until).match(/^(\d{1,2}):(\d{2})$/);
  const end = hm ? new Date(now) : new Date(until);
  if (hm) {
    end.setHours(Number(hm[1]), Number(hm[2]), 0, 0);
    if (end <= now) end.setDate(end.getDate() + 1);
  }
  if (isNaN(end) || end <= now) throw new Error(`--until ${until}: not "HH:MM" or a time to come`);
  return end;
}

/**
 * The lend, with no listener: `handle(req, res)` answers each request, and
 * `used` is each person's tokens by the door's key, `{ name, tokens }`.
 * `provider` is where /chat/completions goes on; `models` maps each name
 * lent to the provider's id.
 */
export async function spot({ provider, models, until = null, budget = null }) {
  if (!provider) throw new Error("say the provider's OpenAI address: --provider <url>");
  if (models.size === 0) throw new Error('lend a model: --model <name>=<the provider’s id>');
  const upstream = provider.replace(/\/$/, '');
  const host = new URL(upstream).hostname;
  const end = until instanceof Date ? until : endOf(until);
  const files = new Map(
    [
      ['/', 'page.html', 'text/html'],
      ['/page.js', 'page.js', 'text/javascript'],
      ['/markdown.js', 'markdown.js', 'text/javascript'],
    ].map(([at, f, type]) => [
      at,
      [`${type}; charset=utf-8`, readFileSync(new URL(f, import.meta.url))],
    ]),
  );
  const used = new Map();
  let spent = 0;
  const over = () =>
    end && Date.now() > end
      ? `this lend ended at ${end.toLocaleTimeString()}`
      : budget !== null && spent >= budget
        ? `its ${n(budget)} tokens are spent`
        : null;
  const json = (res, status, body) => {
    res.writeHead(status, { 'content-type': 'application/json', 'cache-control': 'no-store' });
    res.end(JSON.stringify(body));
  };
  const refuse = (res, name, status, why) => {
    console.log(`spot: ${name}, refused: ${why}`);
    json(res, status, { error: { message: why, type: 'spot_refused' } });
  };

  const handle = async (req, res) => {
    const [key, name] = [told(req, 'x-svrngs-key'), told(req, 'x-svrngs-person') ?? 'nobody'];
    const route = new URL(req.url, 'http://spot').pathname;
    if (req.method === 'GET' && files.has(route)) {
      const [type, bytes] = files.get(route);
      res.writeHead(200, { 'content-type': type, 'cache-control': 'no-cache' });
      return res.end(bytes);
    }
    if (req.method === 'GET' && route === '/spot.json')
      return json(res, 200, {
        models: [...models.keys()],
        provider: host,
        until: end?.toISOString() ?? null,
        budget,
        spent,
        yours: used.get(key)?.tokens ?? 0,
        ended: over(),
      });
    if (req.method === 'GET' && route === '/v1/models')
      return json(res, 200, {
        object: 'list',
        data: [...models.keys()].map((id) => ({ id, object: 'model', owned_by: 'spot' })),
      });
    if (!(req.method === 'POST' && route === '/v1/chat/completions'))
      return refuse(
        res,
        name,
        404,
        `only the page and two OpenAI paths are lent, not ${req.method} ${route}`,
      );
    if (over()) return refuse(res, name, 403, over());
    if (!key || !req.headers.authorization)
      return refuse(res, name, 401, 'a chat comes through svrngs, which adds the key');
    const chunks = [];
    let size = 0;
    for await (const c of req) {
      if ((size += c.length) > 16 << 20) return refuse(res, name, 413, 'a request over 16 MiB');
      chunks.push(c);
    }
    let body;
    try {
      body = JSON.parse(Buffer.concat(chunks));
    } catch {
      return refuse(res, name, 400, 'the body is not JSON');
    }
    const asked = body.model;
    if (!models.has(asked))
      return refuse(
        res,
        name,
        400,
        `${JSON.stringify(asked)} is not lent: ${[...models.keys()].join(', ')}`,
      );
    body.model = models.get(asked);
    // A stream says its tokens only when asked to, in its last event.
    if (body.stream) body.stream_options = { ...body.stream_options, include_usage: true };
    let answer;
    try {
      answer = await fetch(`${upstream}/chat/completions`, {
        method: 'POST',
        headers: {
          authorization: req.headers.authorization,
          'content-type': 'application/json',
          accept: req.headers.accept ?? '*/*',
        },
        body: JSON.stringify(body),
      });
    } catch (e) {
      return refuse(res, name, 502, `${host} did not answer: ${e.cause?.code ?? e.message}`);
    }
    res.writeHead(answer.status, {
      'content-type': answer.headers.get('content-type') ?? 'application/json',
    });
    let tail = '';
    const stream = Readable.fromWeb(answer.body);
    stream.on('data', (c) => {
      tail = (tail + c).slice(-8192);
      res.write(c);
    });
    stream.on('error', () => res.end());
    stream.on('end', () => {
      res.end();
      const usage = [...tail.matchAll(/"usage"\s*:\s*(\{[^{}]*\})/g)].pop()?.[1];
      const tokens = usage ? (JSON.parse(usage).total_tokens ?? 0) : 0;
      spent += tokens;
      used.set(key, { name, tokens: (used.get(key)?.tokens ?? 0) + tokens });
      const what = answer.ok
        ? `${tokens ? n(tokens) : 'unreported'} tokens`
        : `${host} answered ${answer.status}`;
      console.log(
        `spot: ${name}, ${asked}, ${what}${budget ? ` (${n(spent)} of ${n(budget)})` : ''}`,
      );
    });
  };
  return { handle, used };
}

/** The lend listening on this computer's loopback alone: svrngs carries everyone else to it. */
export async function start({ port, ...options }) {
  const lend = await spot(options);
  const server = http.createServer(lend.handle);
  await new Promise((done, failed) => server.once('error', failed).listen(port, '127.0.0.1', done));
  return { ...lend, server, port: server.address().port };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  const all = (flag) => process.argv.flatMap((a, i) => (a === flag ? [process.argv[i + 1]] : []));
  const one = (flag) => all(flag)[0] ?? null;
  const models = new Map(
    all('--model').map((m) => [m.split('=')[0], m.split('=').slice(1).join('=')]),
  );
  try {
    const port = Number(one('--port'));
    if (!Number.isInteger(port) || port <= 0) throw new Error('say its port: --port <port>');
    const budget = one('--budget') === null ? null : Number(one('--budget').replaceAll('_', ''));
    const lend = await start({
      port,
      provider: one('--provider'),
      models,
      until: one('--until'),
      budget,
    });
    console.log(`spot: ${[...models.keys()].join(', ')} at http://127.0.0.1:${lend.port}/`);
    const bye = () => {
      const who = [...lend.used.values()].map((u) => `${u.name} ${n(u.tokens)}`).join(', ');
      console.log(`spot: stopped. Used: ${who || 'nothing'}.`);
      process.exit(0);
    };
    process.on('SIGINT', bye);
    process.on('SIGTERM', bye);
  } catch (e) {
    console.error(`spot: ${e.message}`);
    process.exit(1);
  }
}
