// How a program on this computer asks svrngs as you: the one way, and safe
// ("Never send the key", svrngs V1.13.3). Copy this file beside your
// program, run with Node 24 or later, and import it:
//
//   import { ask } from './svrngs.mjs';
//   const { groups } = await ask('/v1/groups');
//   await ask(`/v1/rail/append?namespace=${group}`, { op: 'record', payload });
//
// `ask(route)` asks with GET, `ask(route, body)` posts `body` as JSON, and
// either answers svrngs's answer, parsed, or throws an Error that says why
// not: svrngs refused (`.status`, `.answer`), or svrngs never was asked
// (`.withheld`: away, older, unanswered or unproven). `route` is a path on
// svrngs, as `/v1/groups`, or a lend's address on this computer,
// `http://<name>.localhost:<port>/…`; it asks nothing else.
//
// `svrngs(dir)` is the same for the svrngs whose data dir is `dir`, with
// `fetch(route, init)` beside its `ask`: the global fetch's shape, for an
// answer streamed as it comes, a method or headers of your own. Read or
// cancel its body, as with any fetch.
//
// svrngs's address is its data dir's `api` file ($SVRNGS_DIR, else
// ~/.svrngs), and this file reads the `token` beside it itself. The token
// never leaves this file, to svrngs or to your program. On each connection
// svrngs first proves it holds the token, for a challenge made here
// (POST /v1/proof); only then is a request sent, on that same connection,
// with a code made for that connection alone (`Authorization: Svrngs
// <code>`), useless on any other. So a program holding svrngs's address
// while svrngs is down is sent a challenge and nothing more.
//
// NODE_DEBUG=svrngs traces each connection: opened, proved or why not,
// used again, gone. Never the token, never a code.

import { createHmac, randomBytes, timingSafeEqual } from 'node:crypto';
import { readFileSync } from 'node:fs';
import http from 'node:http';
import net from 'node:net';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { Readable } from 'node:stream';
import { debuglog } from 'node:util';

const trace = debuglog('svrngs');

/** How long svrngs is given to prove itself on a connection, as svrngs's
 * own command waits (crates/svrngs-node/src/api.rs `PROVE_WITHIN`): past
 * svrngs's start, which holds its address before it answers there. */
export const PROVE_WITHIN = 10_000;

/** The most of an answer read before svrngs has proved itself: a proof
 * and a nonce are some 150 bytes. */
const UNPROVEN_MOST = 4096;

/** How many proved connections are kept open between asks. */
const KEPT = 4;

const HEX64 = /^[0-9a-f]{64}$/;

/** svrngs's proof that it holds `token`, for `challenge` on the connection
 * from `client` to `server`, each as Rust's `SocketAddr` prints it
 * (crates/svrngs-node/src/api.rs `proof`, the one decider; held to its
 * vectors in svrngs.test.mjs). Exported for a stand-in svrngs in tests. */
export function proof(token, challenge, client, server) {
  return mac(token, `svrngs proof v1\n${challenge}\n${client}\n${server}`);
}

/** The code each request on one connection asks with: for the `nonce`
 * svrngs gave that connection and its two ends. Exported for a stand-in
 * svrngs in tests. */
export function code(token, nonce, client, server) {
  return mac(token, `svrngs ask v1\n${nonce}\n${client}\n${server}`);
}

const mac = (token, message) => createHmac('sha256', token.trim()).update(message).digest('hex');

/** A connection's end as Rust's `SocketAddr` prints it, and so as svrngs
 * names it in a proof: `127.0.0.1:50000`, `[::1]:50001`. */
const end = (address, port) =>
  net.isIPv6(address) ? `[${address}]:${port}` : `${address}:${port}`;

/** Why svrngs was never asked: a closed set, as svrngs's command's
 * (crates/svrngs-node/src/api.rs `Withheld`). Each sentence says what to do. */
const WITHHELD = Object.freeze({
  away: (at) =>
    `svrngs isn't running on this computer: nothing answers at ${at}. Start svrngs, then try again.`,
  older: (at) =>
    `What answers at ${at} is a svrngs older than this program, which asks without ever sending your token, or isn't svrngs; it was sent none of your token. Run \`svrngs restart\` (after an update, if svrngs is older), then try again.`,
  unanswered: (at) =>
    `Nothing at ${at}, where svrngs listens, proved it is your svrngs within ${PROVE_WITHIN / 1000} seconds; it was sent none of your token. Try again in a moment; if this keeps happening, run \`svrngs restart\`.`,
  unproven: (at) =>
    `Something other than your svrngs answers at ${at}, where svrngs listens; it was sent none of your token. Stop that program (\`lsof -nP -iTCP -sTCP:LISTEN\` on a Mac, or \`ss -ltnp\` on Linux, names each program listening), then start svrngs again with \`svrngs restart\`.`,
});

function withheld(why, at, words = WITHHELD[why](at)) {
  trace('not asked, %s: %s', why, at);
  return Object.assign(new Error(words), { withheld: why });
}

/** Said when the connection a code was made for is gone before a request
 * went on it: nothing was sent, so it is proved again on a new one. */
const MOVED = Symbol('moved');

/** This computer's svrngs data dir: `$SVRNGS_DIR`, else `~/.svrngs`. */
export const dataDir = () => process.env.SVRNGS_DIR || join(homedir(), '.svrngs');

const instances = new Map();

/** svrngs's answer to `route`, the svrngs of this computer's data dir. */
export function ask(route, body) {
  const dir = dataDir();
  if (!instances.has(dir)) instances.set(dir, svrngs(dir));
  return instances.get(dir).ask(route, body);
}

/**
 * The svrngs whose data dir is `dir`: its `ask` and its `fetch`.
 * `connect` opens a connection to svrngs's address (node:net's, or a stand-in's in a test).
 */
export function svrngs(dir = dataDir(), { connect = net.connect } = {}) {
  const idle = [];

  function read(file, missing) {
    try {
      return readFileSync(join(dir, file), 'utf8').trim();
    } catch {
      throw withheld('away', dir, missing);
    }
  }

  /** svrngs's address, from the `api` file it writes as it starts. */
  function api() {
    const said = read(
      'api',
      `svrngs isn't running on this computer: ${join(dir, 'api')}, where svrngs writes its address as it starts, isn't there. Start svrngs, then try again.`,
    );
    return new URL(said);
  }

  /** One connection to svrngs at `at`, svrngs proved on it, and its code. */
  async function open() {
    const at = api();
    const token = read(
      'token',
      `${join(dir, 'token')} isn't there, so this program can't ask svrngs as you: is ${dir} svrngs's data dir?`,
    );
    let socket = null;
    const agent = new http.Agent({ keepAlive: true, maxSockets: 1 });
    // The one connection this code is for: never another, so a request
    // can't go where svrngs would refuse it, or anywhere else.
    agent.createConnection = (_, made) => {
      if (socket)
        return void process.nextTick(made, Object.assign(new Error('moved'), { [MOVED]: true }));
      socket = connect({ host: at.hostname.replace(/^\[|\]$/g, ''), port: Number(at.port) });
      return socket;
    };
    const line = {
      at,
      agent,
      alive: () => Boolean(socket && !socket.destroyed),
      close: () => agent.destroy(),
    };
    const challenge = randomBytes(32).toString('hex');
    const sent = {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ challenge }),
    };
    let timer;
    const late = new Promise((_, failed) => {
      timer = setTimeout(() => failed(withheld('unanswered', at.origin)), PROVE_WITHIN);
    });
    const asked = whole(send(line, at.host, '/v1/proof', sent), at);
    asked.catch(() => {});
    let answer;
    try {
      answer = await Promise.race([asked, late]);
    } catch (e) {
      line.close();
      throw e.withheld ? e : withheld('away', at.origin);
    } finally {
      clearTimeout(timer);
    }
    const client = end(socket.localAddress, socket.localPort);
    const server = end(socket.remoteAddress, socket.remotePort);
    const said = answer.status === 200 ? parsed(answer.body) : null;
    const wanted = Buffer.from(proof(token, challenge, client, server));
    const shown = Buffer.from(typeof said?.proof === 'string' ? said.proof : '');
    const proved = shown.length === wanted.length && timingSafeEqual(shown, wanted);
    trace(
      'connection %s to %s: %s',
      client,
      server,
      proved ? 'proved' : `not proved (${answer.status})`,
    );
    // A 404 is a svrngs from before the proof; a proof with no nonce, one
    // from before it stopped taking the token.
    let why = proved ? null : 'unproven';
    if (answer.status === 404 || (proved && !HEX64.test(said.nonce))) why = 'older';
    if (why) {
      line.close();
      throw withheld(why, at.origin);
    }
    line.code = code(token, said.nonce, client, server);
    return line;
  }

  /** The proof's answer, whole, if it is no more than UNPROVEN_MOST. */
  async function whole(asked, at) {
    const { res, req } = await asked;
    const chunks = [];
    let size = 0;
    for await (const chunk of res) {
      size += chunk.length;
      if (size > UNPROVEN_MOST) {
        req.destroy();
        throw withheld('unproven', at.origin);
      }
      chunks.push(chunk);
    }
    return { status: res.statusCode, body: Buffer.concat(chunks) };
  }

  /** One request on `line`'s connection; its answer once its head is in. */
  function send(line, host, path, { method = 'GET', headers = {}, body, signal } = {}) {
    return new Promise((done, failed) => {
      const head = { ...headers, host, connection: 'keep-alive' };
      if (line.code) head.authorization = `Svrngs ${line.code}`;
      const req = http.request({
        agent: line.agent,
        host: line.at.hostname,
        port: line.at.port,
        path,
        method,
        headers: head,
        signal,
      });
      req.on('response', (res) => done({ res, req }));
      req.on('error', failed);
      req.end(body);
    });
  }

  /** Where `route` asks: svrngs's API, or a lend's address on this computer. */
  function place(route, at) {
    if (typeof route === 'string' && route.startsWith('/') && !route.startsWith('//'))
      return { host: at.host, path: route };
    let url = null;
    try {
      url = new URL(route);
    } catch {}
    const lend =
      url?.protocol === 'http:' && url.port === at.port && url.hostname.endsWith('.localhost');
    if (url && (url.origin === at.origin || lend))
      return { host: url.host, path: `${url.pathname}${url.search}` };
    throw new TypeError(
      `this asks svrngs alone: a route on it, as /v1/groups, or a lend's address on this computer, http://<name>.localhost:${at.port}/…; not ${route}`,
    );
  }

  /** svrngs's answer to `route`, as the global fetch answers. */
  async function fetch(route, init = {}) {
    const headers = Object.fromEntries(new Headers(init.headers));
    if (headers.authorization !== undefined)
      throw new TypeError(
        'svrngs takes no Authorization from a program: leave it out, and this asks svrngs as you, without ever sending its token',
      );
    const { host, path } = place(route, api());
    const body = init.body === undefined || init.body === null ? undefined : Buffer.from(init.body);
    for (;;) {
      let line = idle.pop();
      while (line && !line.alive()) {
        line.close();
        line = idle.pop();
      }
      if (line) trace('connection used again: %s', path);
      line ??= await open();
      let asked;
      try {
        asked = await send(line, host, path, { ...init, headers, body });
      } catch (e) {
        line.close();
        if (!e[MOVED]) throw e;
        trace('its connection gone before %s was sent: proving svrngs on a new one', path);
        continue;
      }
      return answered(asked, line, init.method);
    }
  }

  /** `res` as a fetch Response; its connection kept for the next ask once it ends. */
  function answered({ res, req }, line, method = 'GET') {
    res.on('end', () => {
      if (!req.shouldKeepAlive || idle.length >= KEPT) line.close();
      else idle.push(line);
    });
    res.on('error', () => line.close());
    const headers = new Headers();
    for (const [name, value] of Object.entries(res.headers))
      for (const one of [value].flat()) headers.append(name, one);
    const empty = [204, 304].includes(res.statusCode) || method.toUpperCase() === 'HEAD';
    if (empty) res.resume();
    return new Response(empty ? null : Readable.toWeb(res), {
      status: res.statusCode,
      statusText: res.statusMessage,
      headers,
    });
  }

  /** svrngs's answer to `route`, or to `body` posted there as JSON, parsed. */
  async function ask(route, body) {
    const init =
      body === undefined
        ? {}
        : {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify(body),
          };
    const res = await fetch(route, init);
    const text = await res.text();
    const said = text ? (parsed(text) ?? text) : null;
    if (res.ok) return said;
    const why = typeof said?.error === 'string' ? said.error : text;
    const path = String(route).split('?')[0];
    throw Object.assign(new Error(`svrngs refused ${path}: ${res.status} ${why}`), {
      status: res.status,
      answer: said,
    });
  }

  return { ask, fetch, dir };
}

function parsed(text) {
  try {
    return JSON.parse(String(text));
  } catch {
    return null;
  }
}
