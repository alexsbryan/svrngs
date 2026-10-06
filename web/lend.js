// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's worker: the runtime, at the lend's own origin (u5-2; DESIGN §3,
// "Lent apps", calls 1, 3 and 6; §7). Moved from the page's worker, where
// every lend ran under the origin that holds each person's key (sw.js).
//
// It serves every request under its scope — the lend's pages, their
// subresources, fetch and XHR, a `<video>`'s byte ranges — by HTTP over a
// dial to the lender's computer, at its door's `/lend/<label>/<path>`, on
// the pass holders' protocol, as the fresh key the page made, presenting
// the pass the page signed (open.js hands both over); the door lets it in
// as the person. A request a lent page makes to another lend's address is
// served the same way when the pass names that lend too, which its lender
// declared it reads; else refused. Each answer goes back as the door gave
// it, a 206 and its `Content-Range` with it, its body streamed.
//
// What a browser does for an origin and a worker cannot see, it does here:
// - Cookies (a worker's request carries no `Cookie`, and its answer sets
//   none): kept per lend, each in its own jar in this origin's store, sent
//   only on that lend's requests, and handed to that lend's pages, which
//   read them through the shim (shim.js) it puts in each of its pages.
// - A redirect to a path: read under the lend's own address.
// - A WebSocket, which a worker never sees: the shim carries it here, and
//   the worker speaks its frames over a socket the door switches (`socket`).
// - The app's own worker: the shim hands its script here, and its listeners
//   answer before the lend's (`appWorker`).
//
// Its trace is at `<scope>svrngs-trace`: a phone has no console to read.
import init, { holder_endpoint } from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';

const SCOPE = new URL(self.registration.scope);
// Where the page's files are on this origin, the shell this worker leaves
// to the network: any version of them, so a newer page starts here too.
const SITE = new URL('./', self.location).pathname;
const SHELL = /^(open\.html|open\.js|lend\.js|shim\.js|store\.js|wasm\/svrngs_web(_bg\.wasm|\.js))$/;
// This worker's lend: `l/<label>/` under one address for every app, else
// the first name of an address of its own.
const UNDER = /\/l\/([a-z0-9]{62})\/$/.exec(SCOPE.pathname)?.[1];
const OWN = UNDER ?? SCOPE.hostname.split('.')[0];
const PLAIN = { 'content-type': 'text/plain; charset=utf-8' };
const TRACE = `${SCOPE.pathname}svrngs-trace`;

const traced = [];
self.__trace = (l) => {
  console.debug(l);
  traced.push(`${new Date().toISOString()} ${l}`);
  if (traced.length > 300) traced.shift();
};
const trace = self.__trace;

self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (e) => e.waitUntil(self.clients.claim()));

// What this run of the worker holds, all of it lost when the browser stops
// it: what the page handed it, the wasm, its endpoint, a dial per lender,
// each lend's cookies, the app's worker.
let handed = null;
let loaded = null;
let endpoint = null;
const dials = new Map();
const jars = new Map();
let app;

// Where `label`'s pages are: this worker's lend under its scope; another,
// under one address for every app, at `l/<label>/` beside it, else at an
// address of its own beside this one.
const baseOf = (label) =>
  label === OWN ? SCOPE.href : UNDER ? new URL(`../${label}/`, SCOPE).href : `${SCOPE.protocol}//${label}${SCOPE.host.slice(OWN.length)}/`;

// The lend `url` is at, and its path there; or none.
function at(url) {
  for (const label of [OWN, ...(handed?.lends ?? []).map((l) => l.label)]) {
    const base = new URL(baseOf(label));
    if (url.origin === base.origin && url.pathname.startsWith(base.pathname)) {
      return { label, path: `/${url.pathname.slice(base.pathname.length)}${url.search}` };
    }
  }
  return null;
}

async function holding() {
  if (handed) return handed;
  try {
    handed = (await store.lendKept(`pass ${OWN}`)) ?? null;
    trace(`RUNTIME: the pass for ${OWN} ${handed ? 'read from this origin’s store' : 'is not kept here'}`);
  } catch (e) {
    trace(`RUNTIME: this origin's store cannot be read here: ${e}`);
  }
  return handed;
}

function dialFor(lender) {
  if (!dials.has(lender)) {
    const opening = (async () => {
      loaded ??= init({ module_or_path: store.runtime() });
      await loaded;
      const holder = Uint8Array.from(handed.holder.match(/../g), (h) => parseInt(h, 16));
      if (!endpoint) trace(`RUNTIME: opening the endpoint as the pass's holder`);
      endpoint ??= holder_endpoint(holder, handed.relay);
      trace(`RUNTIME: dialling ${lender} as the pass's holder`);
      return (await endpoint).dial_lend(lender);
    })();
    dials.set(lender, opening);
    opening.catch(() => dials.delete(lender));
  }
  return dials.get(lender);
}

// ── cookies, a jar per lend ─────────────────────────────────────────────

async function jarOf(label) {
  if (!jars.has(label)) jars.set(label, (await store.lendKept(`cookies ${label}`).catch(() => null)) ?? []);
  return jars.get(label);
}

const live = (c) => c.expires === null || c.expires > Date.now();

// Keep a `Set-Cookie` value in `jar`: its name, value, path, lapse and
// whether pages may read it. Any other attribute is the browser's, and
// there is no other site here to keep it from.
function setCookie(jar, line) {
  const [pair, ...attrs] = line.split(';');
  const eq = pair.indexOf('=');
  if (eq < 1) return;
  const c = { name: pair.slice(0, eq).trim(), value: pair.slice(eq + 1).trim(), path: '/', expires: null, page: true };
  for (const attr of attrs) {
    const [k, ...v] = attr.split('=');
    const [key, value] = [k.trim().toLowerCase(), v.join('=').trim()];
    if (key === 'path' && value.startsWith('/')) c.path = value;
    if (key === 'max-age') c.expires = Date.now() + Number(value) * 1000;
    if (key === 'expires' && c.expires === null) c.expires = Date.parse(value);
    if (key === 'httponly') c.page = false;
  }
  const kept = jar.filter((o) => o.name !== c.name || o.path !== c.path);
  jar.splice(0, jar.length, ...kept, ...(live(c) ? [c] : []));
}

const cookieFor = (jar, path, page) =>
  jar.filter((c) => live(c) && path.startsWith(c.path) && (c.page || !page)).map((c) => `${c.name}=${c.value}`).join('; ');

async function keepJar(label) {
  await store.lendKeep(`cookies ${label}`, await jarOf(label)).catch((e) => trace(`RUNTIME: cookies not kept: ${e}`));
}

// ── a request, through the door ─────────────────────────────────────────

// A range that runs on is asked of the door as a part of PART bytes (u4-9;
// moved from sw.js): an open-ended `bytes=N-`, and a `<video>`'s closed
// range longer than a part, WebKit's `bytes=N-<end>`. A page's own fetch
// of a closed range gets what it asked.
const PART = 2 * 1024 * 1024;

function bounded(headers, destination) {
  return headers.map(([name, value]) => {
    const range = name.toLowerCase() === 'range' && /^bytes=(\d+)-(\d*)$/.exec(value.trim());
    if (!range) return [name, value];
    const from = Number(range[1]);
    const media = destination === 'video' || destination === 'audio';
    if (range[2] !== '' && !(media && Number(range[2]) - from + 1 > PART)) return [name, value];
    trace(`RUNTIME: ${value} for ${destination || 'a fetch'} runs on: asked as a part`);
    return [name, `bytes=${from}-${from + PART - 1}`];
  });
}

const say = (status, words) => new Response(words, { status, headers: PLAIN });

// The shim (shim.js) for a lend's page, with the cookies its pages read,
// and the nonce of the page's script policy when it names one: under
// `script-src 'nonce-…' 'strict-dynamic'` (Nextcloud's) a script without
// it does not run (u5-catalogue-dry).
const shim = (label, jar, policy) => {
  const nonce = /'nonce-([^']+)'/.exec(policy ?? '')?.[1];
  return `<script${nonce ? ` nonce="${nonce}"` : ''} src="${SITE}shim.js?v=__VERSION__&l=${label}&c=${encodeURIComponent(cookieFor(jar, '/', true))}"></script>`;
};

// Serve `request`, at `url`, from the lend `where` names.
async function lent(request, url, where) {
  const h = await holding();
  if (!h) {
    if (request.mode !== 'navigate') return say(503, 'Open this again from your group’s page.');
    trace(`RUNTIME: no pass for ${OWN}: its start page reads it again`);
    return Response.redirect(`${SCOPE.origin}${SITE}open.html?v=__VERSION__&for=${OWN}&back=${encodeURIComponent(url.pathname + url.search)}`, 302);
  }
  const lend = h.lends.find((l) => l.label === where.label);
  if (!lend) return say(403, 'This is not open to this page.');
  const jar = await jarOf(where.label);
  const headers = bounded(
    [...request.headers].filter(([n]) => n.toLowerCase() !== 'cookie'),
    request.destination,
  );
  const cookie = cookieFor(jar, where.path, false);
  if (cookie) headers.push(['cookie', cookie]);
  headers.push(['x-svrngs-pass', h.pass]);
  const body = ['GET', 'HEAD'].includes(request.method) ? null : (request.body ?? (await request.blob()).stream());
  const path = `/lend/${where.label}${where.path}`;
  let res;
  for (let tries = 0; !res; tries++) {
    try {
      res = await (await dialFor(lend.key)).fetch(request.method, path, headers, body);
    } catch (e) {
      dials.delete(lend.key);
      trace(`RUNTIME: ${request.method} ${path} failed (try ${tries + 1}): ${e}`);
      if (body || tries) return say(502, 'The computer lending this did not answer. Try again later.');
    }
  }
  const set = res.headers.get('x-svrngs-set-cookies');
  if (set) {
    for (const line of JSON.parse(set)) setCookie(jar, line);
    await keepJar(where.label);
  }
  if (res.status === 403 && res.headers.get('content-type')?.includes('json')) {
    const why = await res.clone().json().catch(() => ({}));
    if (why.kind === 'pass' && request.mode === 'navigate') {
      trace(`RUNTIME: the pass for ${where.label} is refused: its start page asks the group's page for another`);
      return Response.redirect(`${SCOPE.origin}${SITE}open.html?v=__VERSION__&for=${OWN}&again`, 302);
    }
  }
  const to = res.headers.get('location');
  if (res.status >= 300 && res.status < 400 && to) {
    const there = new URL(to.startsWith('/') ? `.${to}` : to, baseOf(where.label)).href;
    trace(`RUNTIME: ${path} redirects to ${to}, read as ${there}`);
    return Response.redirect(there, [301, 302, 303, 307, 308].includes(res.status) ? res.status : 302);
  }
  const headed = new Headers(res.headers);
  headed.delete('x-svrngs-set-cookies');
  const html = request.method === 'GET' && (res.headers.get('content-type') ?? '').includes('text/html');
  if (!html) return new Response(res.body, { status: res.status, statusText: res.statusText, headers: headed });
  headed.delete('content-length');
  const text = await res.text();
  const head = /<head[^>]*>/i.exec(text) ?? /<html[^>]*>/i.exec(text) ?? /<!doctype[^>]*>/i.exec(text);
  const at = head ? head.index + head[0].length : 0;
  const page = `${text.slice(0, at)}${shim(where.label, jar, res.headers.get('content-security-policy'))}${text.slice(at)}`;
  return new Response(page, { status: res.status, statusText: res.statusText, headers: headed });
}

// ── the app's own worker ────────────────────────────────────────────────
//
// An app that registers a worker of its own would put it over this one,
// which serves its every request; the shim hands its script here instead,
// kept in this origin's store, and its listeners are asked first.

function appWorker(code) {
  const on = { fetch: [], install: [], activate: [], message: [] };
  const own = {
    addEventListener: (type, f) => on[type]?.push(f),
    skipWaiting: async () => {},
    fetch: (input, init) => {
      const request = new Request(new URL(input?.url ?? input, SCOPE), init);
      const where = at(new URL(request.url));
      return where ? lent(request, new URL(request.url), where) : fetch(request);
    },
  };
  const scope = new Proxy(self, {
    get: (o, k) => (k in own ? own[k] : typeof o[k] === 'function' ? o[k].bind(o) : o[k]),
  });
  new Function('self', 'addEventListener', 'skipWaiting', 'fetch', code)(scope, own.addEventListener, own.skipWaiting, own.fetch);
  for (const f of [...on.install, ...on.activate]) f({ waitUntil: () => {} });
  trace(`RUNTIME: the app's worker runs, ${on.fetch.length} fetch listener(s)`);
  return on;
}

async function appLoaded() {
  if (app === undefined) {
    const code = await store.lendKept(`worker ${OWN}`).catch(() => null);
    app = code ? appWorker(code) : null;
  }
  return app;
}

// The app's listeners first: an answer one of them gives, else the lend's.
async function answered(e, url, where) {
  const on = where.label === OWN && (await appLoaded());
  let given = null;
  if (on) {
    const event = { request: e.request, clientId: e.clientId, respondWith: (p) => (given ??= p), waitUntil: () => {} };
    for (const f of on.fetch) f.call(self, event);
  }
  return given ? given : lent(e.request, url, where);
}

self.addEventListener('fetch', (e) => {
  const url = new URL(e.request.url);
  if (url.origin === self.location.origin && url.pathname === TRACE) {
    return e.respondWith(new Response(`svrngs lend worker __VERSION__ for ${OWN}\n${traced.join('\n')}`, { headers: PLAIN }));
  }
  const shell = url.origin === self.location.origin && url.searchParams.has('v') && url.pathname.startsWith(SITE);
  if (shell && SHELL.test(url.pathname.slice(SITE.length))) return;
  const where = at(url);
  if (!where) return;
  e.respondWith(answered(e, url, where));
});

// ── what a lend's page hands over ───────────────────────────────────────

self.addEventListener('message', (e) => {
  const reply = e.ports[0];
  const { handed: given, cookie, worker, socket: asked } = e.data ?? {};
  if (given?.label === OWN && typeof given.pass === 'string') {
    handed = given;
    dials.clear();
    endpoint = null;
    trace(`RUNTIME: the page handed over the pass for ${OWN}, ${given.lends.length} lend(s)`);
    return reply?.postMessage('held');
  }
  if (typeof cookie === 'string') {
    return e.waitUntil(jarOf(OWN).then((jar) => (setCookie(jar, cookie), keepJar(OWN))).then(() => reply?.postMessage('kept')));
  }
  if (typeof worker === 'string') {
    const url = new URL(worker, SCOPE);
    const where = at(url);
    return e.waitUntil(
      (async () => {
        const res = where && (await lent(new Request(url), url, where));
        if (!res?.ok) return reply?.postMessage({ error: `its worker answered ${res?.status}` });
        const code = await res.text();
        await store.lendKeep(`worker ${OWN}`, code);
        app = appWorker(code);
        reply?.postMessage('running');
      })().catch((err) => reply?.postMessage({ error: String(err) })),
    );
  }
  if (asked && reply) return e.waitUntil(socket(asked, reply));
  trace(`RUNTIME: a message that hands over nothing, refused: ${Object.keys(e.data ?? {})}`);
});

// ── a WebSocket, over a socket the door switches ────────────────────────

// One frame from this end: masked, as a client's must be (RFC 6455 §5.3).
function frame(op, data) {
  const bytes = typeof data === 'string' ? new TextEncoder().encode(data) : new Uint8Array(data);
  const n = bytes.length;
  const head = n < 126 ? [0x80 | op, 0x80 | n] : n < 65536 ? [0x80 | op, 0xfe, n >> 8, n & 255] : [0x80 | op, 0xff, 0, 0, 0, 0, n >>> 24, (n >>> 16) & 255, (n >>> 8) & 255, n & 255];
  const mask = crypto.getRandomValues(new Uint8Array(4));
  const out = new Uint8Array(head.length + 4 + n);
  out.set(head);
  out.set(mask, head.length);
  for (let i = 0; i < n; i++) out[head.length + 4 + i] = bytes[i] ^ mask[i & 3];
  return out;
}

// Carry the WebSocket the shim opened at `url` to `port`: open, each
// message both ways, ping answered, close.
async function socket({ url, protocols }, port) {
  const http = new URL(url.replace(/^ws/, 'http'));
  const where = at(http);
  const h = await holding();
  const lend = where && h?.lends.find((l) => l.label === where.label);
  if (!lend) return port.postMessage({ closed: 1006, reason: 'not open to this page' });
  const key = btoa(String.fromCharCode(...crypto.getRandomValues(new Uint8Array(16))));
  const headers = [['connection', 'upgrade'], ['upgrade', 'websocket'], ['sec-websocket-version', '13'], ['sec-websocket-key', key], ['x-svrngs-pass', h.pass]];
  if (protocols?.length) headers.push(['sec-websocket-protocol', protocols.join(', ')]);
  const cookie = cookieFor(await jarOf(where.label), where.path, false);
  if (cookie) headers.push(['cookie', cookie]);
  let s;
  try {
    s = await (await dialFor(lend.key)).socket(`/lend/${where.label}${where.path}`, headers);
  } catch (e) {
    trace(`RUNTIME: a socket at ${where.path} failed: ${e}`);
    return port.postMessage({ closed: 1006, reason: String(e) });
  }
  if (s.status !== 101) return port.postMessage({ closed: 1006, reason: `answered ${s.status}` });
  const writer = s.writable.getWriter();
  const chosen = s.headers.find(([n]) => n.toLowerCase() === 'sec-websocket-protocol')?.[1] ?? '';
  port.postMessage({ open: chosen });
  port.onmessage = ({ data }) => {
    if ('send' in data) writer.write(frame(typeof data.send === 'string' ? 1 : 2, data.send));
    if ('close' in data) writer.write(frame(8, new Uint8Array([data.close >> 8, data.close & 255])));
  };
  const reader = s.readable.getReader();
  let buf = new Uint8Array(0);
  let parts = [];
  for (;;) {
    const { value, done } = await reader.read();
    if (done) return port.postMessage({ closed: 1006, reason: 'the stream ended' });
    buf = Uint8Array.from([...buf, ...value]);
    while (buf.length >= 2) {
      let n = buf[1] & 127;
      let at = 2;
      if (n === 126) [n, at] = [(buf[2] << 8) | buf[3], 4];
      if (n === 127) [n, at] = [buf[6] * 2 ** 24 + ((buf[7] << 16) | (buf[8] << 8) | buf[9]), 10];
      if (buf.length < at + n) break;
      const [op, fin, data] = [buf[0] & 15, buf[0] & 128, buf.slice(at, at + n)];
      buf = buf.slice(at + n);
      if (op === 9) writer.write(frame(10, data));
      if (op === 8) {
        port.postMessage({ closed: data.length >= 2 ? (data[0] << 8) | data[1] : 1005, reason: new TextDecoder().decode(data.slice(2)) });
        return writer.close().catch(() => {});
      }
      if (op === 0 || op === 1 || op === 2) parts.push({ op, data });
      if (fin && (op === 0 || op === 1 || op === 2)) {
        const whole = Uint8Array.from(parts.flatMap((p) => [...p.data]));
        port.postMessage({ message: parts[0].op === 1 ? new TextDecoder().decode(whole) : whole.buffer });
        parts = [];
      }
    }
  }
}
