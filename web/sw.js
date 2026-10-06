// ported from commonwealth-ai sovereign/apps/ring-runtime/web/sw.js@e1a43bcd2
// The page's service worker: the app-shell cache.
//
// It serves no lend (u5-2; DESIGN §3, "Lent apps", calls 1 and 6). This is
// the origin that holds each person's key, sealed in its store, so no lent
// app's page may run here: until u5-2 every lend had an address under this
// worker's scope, `<scope>l/<lender>/<lend>/`, and a script in a lent page
// unsealed the key of whoever opened it (DESIGN §11, risk 5). Each lend now
// opens at an origin of its own, whose worker (lend.js) dials with a pass
// the page signs for it, never the key; the runtime, its trace with it,
// moved there.
//
// The app-shell cache. Cache-first, one version: __VERSION__ is the runtime's
// content hash, stamped at build time, so a rebuild replaces the cache rather
// than serving a stale runtime (the failure this exists to avoid is a guest
// stuck on last week's wasm). The version is also on every shell URL.
import * as store from './store.js?v=__VERSION__';

const CACHE = store.CACHE;
// Relative to this file, so the page works wherever an origin serves it;
// the directory's trailing slash is what makes `./` the page. The wasm is
// not here: the page keeps it in this cache when it first fetches it
// (store.js `runtime`, u4-9).
const SHELL = ['./', './app.js?v=__VERSION__', './store.js?v=__VERSION__', './wasm/svrngs_web.js?v=__VERSION__'];

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

// The shell, from its cache. The dial is not HTTP to this origin — it is
// iroh — so requests to it never reach here.
self.addEventListener('fetch', (e) => {
  const url = new URL(e.request.url);
  if (url.origin !== self.location.origin || e.request.method !== 'GET') return;
  e.respondWith(caches.match(e.request, { ignoreSearch: true }).then((hit) => hit || fetch(e.request)));
});
