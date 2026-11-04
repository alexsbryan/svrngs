// ported from commonwealth-ai sovereign/apps/ring-runtime/web/sw.js@e1a43bcd2
// The page's service worker: the app-shell cache.
//
// It serves no lend (DESIGN §3, "Lent apps", calls 1 and 6). This is the
// origin that holds each person's key, sealed in its store, so no lent
// app's page may run here: once every lend had an address under this
// worker's scope, `<scope>l/<lender>/<lend>/`, and a script in a lent page
// unsealed the key of whoever opened it (DESIGN §8, risk 5). Each lend now
// opens at an origin of its own, whose worker (lend.js) dials with a pass
// the page signs for it, never the key; the runtime, its trace with it,
// moved there.
//
// The app-shell cache. Cache-first, one version: __VERSION__ is the runtime's
// content hash, stamped at build time, so a rebuild replaces the cache rather
// than serving a stale runtime (the failure this exists to avoid is a guest
// stuck on last week's wasm). The version is also on every shell URL.
import * as store from './store.js?v=__VERSION__';

// `self` as a service worker's: the worker's lib names it a
// `WorkerGlobalScope`, which every kind of worker shares
// (jsconfig.workers.json). Its own first, so `addEventListener` gives each
// listener its event's type.
/** @typedef {ServiceWorkerGlobalScope & typeof self} Scope */
const scope = /** @type {Scope} */ (self);

// Relative to this file, so the page works wherever an origin serves it;
// the directory's trailing slash is what makes `./` the page. The wasm is
// not here: the page keeps it in this cache when it first fetches it
// (store.js `runtime`). Each module the page loads, app.js and the
// rest, is the build's to add, from what it copied (scripts/build-web.sh),
// so none is listed by hand.
const SHELL = [
  './',
  './sign.html',
  './look.css?v=__VERSION__',
  './wasm/svrngs_web.js?v=__VERSION__',
];

// The shell kept, and this worker in place of any older one at once.
async function installed() {
  const cache = await caches.open(store.CACHE);
  await cache.addAll(SHELL);
  await scope.skipWaiting();
}

// Every older version's cache gone, and the open pages served from here.
async function activated() {
  const names = await caches.keys();
  const older = names.filter((name) => name !== store.CACHE);
  await Promise.all(older.map((name) => caches.delete(name)));
  await scope.clients.claim();
}

scope.addEventListener('install', (installing) => installing.waitUntil(installed()));
scope.addEventListener('activate', (activation) => activation.waitUntil(activated()));

// The shell, from its cache. The dial is not HTTP to this origin — it is
// iroh — so requests to it never reach here.
scope.addEventListener('fetch', (fetchEvent) => {
  const url = new URL(fetchEvent.request.url);
  if (url.origin !== self.location.origin || fetchEvent.request.method !== 'GET') return;
  fetchEvent.respondWith(answered(fetchEvent.request));
});

// `request` from the shell's cache, whatever its version query, else the network.
/** @param {Request} request */
async function answered(request) {
  const hit = await caches.match(request, { ignoreSearch: true });
  return hit ?? fetch(request);
}
