// ported from commonwealth-ai sovereign/apps/ring-runtime/web/sw.js@e1a43bcd2
// The page's service worker, a module: the runtime, and the app-shell cache.
//
// The runtime (u4-6; DESIGN §3, "Joining", call 3): every lend has one
// address under this worker's scope, `<scope>l/<lender>/<lend>/<path>`,
// the lender's key in full, and the worker serves every request there —
// a lent service's pages, their subresources, fetch and XHR, a `<video>`'s
// byte ranges — by HTTP over a dial to the lender's computer, at its door's
// `/lend/<lend>/<path>` (web/src/http.rs). Each answer goes back as the
// door gave it, status and headers, a 206 and its `Content-Range` with
// them, its body streamed. A page under the scope reaches any lend by a
// path relative to its own (`../../<lender>/<lend>/…`), under this
// person's key: the phone dials each lender itself, so every lender's door
// counts the person watching and no computer carries another's lend (call
// 4). The worker opens its endpoint from the sealed key when it starts —
// the first request that needs it — and so again after the browser stops
// it, since a stopped worker keeps nothing.
//
// The app-shell cache. Cache-first, one version: __VERSION__ is the runtime's
// content hash, stamped at build time, so a rebuild replaces the cache rather
// than serving a stale runtime (the failure this exists to avoid is a guest
// stuck on last week's wasm). The version is also on every shell URL.
import init, { endpoint } from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';

const CACHE = 'svrngs-web-__VERSION__';
// Relative to this file, so the page works wherever an origin serves it;
// the directory's trailing slash is what makes `./` the page.
const SHELL = [
  './',
  './app.js?v=__VERSION__',
  './store.js?v=__VERSION__',
  './wasm/svrngs_web.js?v=__VERSION__',
  './wasm/svrngs_web_bg.wasm?v=__VERSION__',
];

self.addEventListener('install', (e) => {
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(SHELL)).then(() => self.skipWaiting()));
});

self.addEventListener('activate', (e) => {
  e.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

// A lend's address, or the shell. Only the shell is cached. The dial itself
// is not HTTP to this origin — it is iroh — so requests to it never reach
// here.
self.addEventListener('fetch', (e) => {
  const url = new URL(e.request.url);
  if (url.origin !== self.location.origin) return;
  if (url.pathname.startsWith(LENDS)) return e.respondWith(lent(e.request, url));
  if (e.request.method !== 'GET') return;
  e.respondWith(
    caches.match(e.request, { ignoreSearch: true }).then((hit) => hit || fetch(e.request)),
  );
});

// ── the runtime ─────────────────────────────────────────────────────────

// Where every lend's address starts: `l/` under the scope.
const LENDS = new URL('./l/', self.location).pathname;

// The dial's layers, and the worker's own steps, to the console at debug.
self.__trace = (l) => console.debug(l);
const trace = self.__trace;

// What this run of the worker holds, all of it lost when the browser stops
// it: the wasm module, an endpoint per group, a dial per group and lender.
let loaded = null;
const endpoints = new Map();
const dials = new Map();

// Remember `opening` under `key` in `held` until it fails, so the next
// request opens again rather than inheriting the failure.
function held(map, key, opening) {
  if (!map.has(key)) {
    const p = opening();
    map.set(key, p);
    p.catch(() => map.delete(key));
  }
  return map.get(key);
}

// The group whose key reaches `lender`: the one whose list last named
// them, else the one opened last.
async function groupFor(lender) {
  const kept = await store.groups();
  return kept.find((g) => g.lenders?.includes(lender)) ?? kept[0];
}

function endpointFor(group) {
  return held(endpoints, group.id, async () => {
    loaded ??= init().catch((e) => {
      loaded = null;
      throw e;
    });
    await loaded;
    trace(`RUNTIME: opening the endpoint for ${group.id} from the sealed key`);
    return endpoint(await store.unseal(group), group.token);
  });
}

function dialFor(group, lender) {
  return held(dials, `${group.id} ${lender}`, async () => (await endpointFor(group)).dial(lender));
}

// Words for a person, as a page: why a lend did not open.
function say(status, words) {
  return new Response(words, { status, headers: { 'content-type': 'text/plain; charset=utf-8' } });
}

// Serve `request`, at `url` under a lend's address, from its lender's door.
async function lent(request, url) {
  const m = /^([0-9a-f]{64})\/([^/]+)(\/.*)?$/.exec(url.pathname.slice(LENDS.length));
  if (!m) return say(404, 'Nothing is lent at this address.');
  const [, lender, lend, rest] = m;
  // A lend's front page ends in a slash, so its own relative paths stay in it.
  if (rest === undefined) return Response.redirect(`${url.origin}${url.pathname}/${url.search}`, 301);
  const group = await groupFor(lender);
  if (!group) return say(403, 'Open your group’s link on this phone first.');
  const path = `/lend/${lend}${rest}${url.search}`;
  const headers = [...request.headers];
  const body = ['GET', 'HEAD'].includes(request.method) ? null : (request.body ?? (await request.blob()).stream());
  // A dial gone stale since it was opened — the lender's computer
  // restarted, the relay dropped it — is opened again once, for a request
  // with no body to send again.
  for (let tries = 0; ; tries++) {
    try {
      const dialled = await dialFor(group, lender);
      try {
        return await dialled.fetch(request.method, path, headers, body);
      } catch (e) {
        dials.delete(`${group.id} ${lender}`);
        throw e;
      }
    } catch (e) {
      trace(`RUNTIME: ${request.method} ${path} at ${lender} failed (try ${tries + 1}): ${e}`);
      if (body || tries) return say(502, 'The computer lending this did not answer. Try again later.');
    }
  }
}
