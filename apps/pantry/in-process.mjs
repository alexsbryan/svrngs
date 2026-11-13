// The pantry's tests' stand-in for a socket: each origin a request handler
// in this process, asked as node's `http.request` asks a server (the
// pantry's and the bridge's relay) or as `fetch` does (discovery, the
// record, a client). A handler is the one http.createServer is given, so
// a stand-in lend is written as it would be served. The pantry keeps what
// it found by origin (pantry.mjs `discovered`, `keptLends`), so every
// origin here is new in this process.

import { EventEmitter } from 'node:events';
import { PassThrough, Readable } from 'node:stream';

/** A lend whose protocol never answers: asked, it is the probe's bound that ends it. */
export const TIMED_OUT = Symbol('timed out');

let next = 41000;
const served = new Map();

/** `handler` (or TIMED_OUT) at a new origin; its URL with a trailing slash, and how to stop it. */
export function serveAt(handler) {
  const origin = `http://127.0.0.1:${next++}`;
  served.set(origin, handler);
  return { origin: `${origin}/`, close: () => served.delete(origin) };
}

function refused(origin) {
  return Object.assign(new Error(`connect ECONNREFUSED ${origin.slice(7)}`), {
    code: 'ECONNREFUSED',
  });
}

/** The answer `origin`'s handler starts: a readable stream with its status and headers. */
function ask(origin, { method, path, headers, body }) {
  const handler = served.get(origin);
  if (!handler) return Promise.reject(refused(origin));
  const req = Readable.from(body.length ? [body] : []);
  req.method = method;
  req.url = path;
  req.headers = Object.fromEntries(
    Object.entries(headers ?? {}).map(([k, v]) => [k.toLowerCase(), String(v)]),
  );
  const res = new PassThrough();
  res.statusCode = 200;
  res.headers = {};
  res.headersSent = false;
  return new Promise((headed, failed) => {
    res.setHeader = (name, value) => {
      res.headers[name.toLowerCase()] = value;
    };
    res.writeHead = (status, head = {}) => {
      res.statusCode = status;
      for (const [k, v] of Object.entries(head)) res.headers[k.toLowerCase()] = v;
      res.headersSent = true;
      headed(res);
      return res;
    };
    const write = res.write.bind(res);
    const end = res.end.bind(res);
    res.write = (...chunk) => {
      if (!res.headersSent) res.writeHead(res.statusCode);
      return write(...chunk);
    };
    res.end = (...chunk) => {
      if (!res.headersSent) res.writeHead(res.statusCode);
      return end(...chunk);
    };
    Promise.resolve()
      .then(() => handler(req, res))
      .catch((e) => (res.headersSent ? res.destroy(e) : failed(e)));
  });
}

/** fetch, each origin answered by its handler here. */
export async function fetch(url, init = {}) {
  const asked = new Request(url, init);
  const body = Buffer.from(await asked.arrayBuffer());
  const at = new URL(asked.url);
  if (served.get(at.origin) === TIMED_OUT) {
    if (!init.signal) throw new Error(`${url} was asked with no bound, so it would wait forever`);
    throw new DOMException('The operation was aborted due to timeout', 'TimeoutError');
  }
  const res = await ask(at.origin, {
    method: asked.method,
    path: `${at.pathname}${at.search}`,
    headers: Object.fromEntries(asked.headers),
    body,
  });
  const headers = new Headers();
  for (const [k, v] of Object.entries(res.headers))
    for (const one of [v].flat()) headers.append(k, String(one));
  const empty = [204, 304].includes(res.statusCode) || asked.method === 'HEAD';
  return new Response(empty ? null : Readable.toWeb(res), { status: res.statusCode, headers });
}

/** node's http.request, each origin answered by its handler here. */
export function request(options, answered) {
  const target = new EventEmitter();
  const chunks = [];
  target.write = (chunk) => chunks.push(Buffer.from(chunk));
  target.end = (chunk) => {
    if (chunk) chunks.push(Buffer.from(chunk));
    const origin = `http://${options.hostname}:${options.port}`;
    const sent = { ...options, body: Buffer.concat(chunks) };
    (served.get(origin) === TIMED_OUT
      ? Promise.reject(new Error('socket hang up'))
      : ask(origin, sent)
    ).then(answered, (e) => target.emit('error', e));
    return target;
  };
  return target;
}
