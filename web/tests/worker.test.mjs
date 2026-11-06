// The page's worker's two decisions, without a browser (web/sw.js): this
// build's shell from its cache, a different build's from the network, and an
// update moving the open pages onto the build it kept. On 2026-11-03 a phone
// that had been open since 6 October met a device link: its cached page
// refused the answer's `person` field, because the worker matched the shell
// with `ignoreSearch: true`, so the old build answered requests for the new
// one's URLs, and nothing reloaded the page onto the build the node spoke.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';

// `sw.js` takes `self` as it loads, so it is here first.
const scope = {
  clients: { claim: async () => {}, matchAll: async () => [] },
  location: { origin: 'https://group.svrnme.sh' },
  skipWaiting: async () => {},
  addEventListener() {},
};
globalThis.self = scope;

const worker = await import('../sw.js?v=__VERSION__');

const asked = (url) => ({ url, method: 'GET' });

// A cache that matches as the browser does: one URL exactly, unless told to
// ignore the query, as the old `{ ignoreSearch: true }` was.
function cache(entries) {
  return {
    match: async (request, options = {}) => {
      const wanted = new URL(request.url);
      return entries.find((entry) => {
        const kept = new URL(entry.url);
        return options.ignoreSearch
          ? kept.origin === wanted.origin && kept.pathname === wanted.pathname
          : kept.href === wanted.href;
      });
    },
  };
}

// A page at `url` that records where the worker navigated it.
function page(url) {
  const navigated = [];
  return { navigated, client: { url, navigate: async (to) => navigated.push(to) } };
}

test("this build's shell is served from its cache", async () => {
  const kept = { url: 'https://group.svrnme.sh/app.js?v=__VERSION__', from: 'cache' };
  globalThis.caches = cache([kept]);
  globalThis.fetch = async (request) => ({ url: request.url, from: 'network' });
  const got = await worker.answered(asked(kept.url));
  assert.equal(got.from, 'cache');
});

test("an older build's cache does not answer a newer build's request", async () => {
  const stale = {
    url: 'https://group.svrnme.sh/wasm/svrngs_web_bg.wasm?v=3217ebf9ed58',
    from: 'the 6 October build',
  };
  globalThis.caches = cache([stale]);
  globalThis.fetch = async (request) => ({ url: request.url, from: 'network' });
  const got = await worker.answered(
    asked('https://group.svrnme.sh/wasm/svrngs_web_bg.wasm?v=6a3a8c04fa55'),
  );
  assert.equal(got.from, 'network');
});

test('an update reloads the open pages onto the new build', async () => {
  const open = page('https://group.svrnme.sh/#a-link');
  scope.clients = { claim: async () => {}, matchAll: async () => [open.client] };
  const deleted = [];
  globalThis.caches = {
    keys: async () => ['svrngs-web-OLD', 'svrngs-web-__VERSION__'],
    delete: async (name) => deleted.push(name),
  };
  await worker.activated();
  assert.deepEqual(deleted, ['svrngs-web-OLD']);
  assert.deepEqual(open.navigated, ['https://group.svrnme.sh/#a-link']);
});

test('a first install leaves the page the person is opening alone', async () => {
  const open = page('https://group.svrnme.sh/#a-link');
  scope.clients = { claim: async () => {}, matchAll: async () => [open.client] };
  globalThis.caches = {
    keys: async () => ['svrngs-web-__VERSION__'],
    delete: async () => {},
  };
  await worker.activated();
  assert.deepEqual(open.navigated, []);
});

// A lend's worker's saved pieces (KEEP_GOING_AT_HOME 10; web/lend.js), its
// functions cut from its text, as lend.js is a worker a law cannot import.
const LEND = readFileSync(new URL('../lend.js', import.meta.url), 'utf8');
const cut = (name) => {
  const at = LEND.search(new RegExp(`^(async )?function ${name}\\(`, 'm'));
  return LEND.slice(at, LEND.indexOf('\n}\n', at) + 2);
};
const lendWorker = (context) =>
  runInNewContext(
    `const SITE = '/';
     ${cut('saved')}${cut('activated')}${cut('installed')}
     ({ saved, activated, installed });`,
    {
      URL,
      trace: () => {},
      store: {
        CACHE: 'svrngs-web-__VERSION__',
        CACHES: 'svrngs-web-',
        SAVED: ['shim.js', 'open.html'],
        savedAt: (file) => `/${file}?v=__VERSION__`,
      },
      say: (status) => ({ status }),
      ...context,
    },
  );
const at = (path) => ({ url: `https://l.group.svrnme.sh${path}` });
const cacheOf = (files) => ({
  match: async (url) => (files.includes(url) ? { from: 'cache', url } : undefined),
});

test("a lend's worker answers its own build's piece, however the address goes on", async () => {
  const { saved } = lendWorker({
    caches: { open: async () => cacheOf(['/shim.js?v=__VERSION__']) },
    fetch: async () => ({ from: 'network' }),
  });
  const got = await saved(at('/shim.js?v=__VERSION__&l=label&c=cookies'));
  assert.deepEqual([got.from, got.url], ['cache', '/shim.js?v=__VERSION__']);
});

test("a lend's worker never answers a newer build's request with its own copy", async () => {
  const { saved } = lendWorker({
    caches: { open: async () => cacheOf(['/open.js?v=NEWER', '/open.js?v=__VERSION__']) },
    fetch: async () => ({ from: 'network' }),
  });
  assert.equal((await saved(at('/open.js?v=NEWER'))).from, 'network');
});

test('a piece neither saved nor reachable is said, not a broken load', async () => {
  const { saved } = lendWorker({
    caches: { open: async () => cacheOf([]) },
    fetch: async () => Promise.reject(new Error('the site is gone')),
  });
  assert.equal((await saved(at('/shim.js?v=__VERSION__'))).status, 503);
});

test("activation drops older builds' caches and leaves the app's own", async () => {
  const deleted = [];
  const { activated } = lendWorker({
    caches: {
      keys: async () => ['svrngs-web-OLD', 'svrngs-web-__VERSION__', 'the-apps-own'],
      delete: async (name) => deleted.push(name),
    },
    scope: { clients: { claim: async () => {} } },
    Promise,
  });
  await activated();
  assert.deepEqual(deleted, ['svrngs-web-OLD']);
});

test('one piece that cannot be fetched fails the install, so no worker runs half saved', async () => {
  const skipped = [];
  const { installed } = lendWorker({
    caches: {
      open: async () => ({ addAll: async () => Promise.reject(new Error('one piece is gone')) }),
    },
    scope: { skipWaiting: async () => skipped.push(true) },
  });
  await assert.rejects(installed());
  assert.deepEqual(skipped, []);
});

// KEEP_GOING_AT_HOME 12, 18: a tab whose app's computer is silent is shown
// its window, dark; where nothing on the way answered it is told that, since
// no computer was asked. Watched failing with `away` taking no error.
test("an app's window goes dark only for a computer that was reached for", async () => {
  const { unreached } = await import('../../crates/svrngs-node/src/home/question.js?v=__VERSION__');
  const lend = { label: 'l', person: 'ada', name: 'notes' };
  const away = runInNewContext(`${cut('away')}\naway;`, {
    QUIET: 'quiet',
    REFUSED: 'x-svrngs-refused',
    unreached,
    awayPage: async () => '{who}’s window is dark',
    trace: () => {},
    say: (status, words) => ({ status, words }),
    Response: class {
      constructor(body) {
        this.words = body;
      }
    },
  });
  const tab = { mode: 'navigate' };
  const silent = new Error('LAYER tls/alpn+relay: REFUSED — not answered in 8s: asleep');
  assert.equal((await away(tab, lend, silent)).words, 'ada’s window is dark');
  const nothing = new Error('LAYER relay-connect: REFUSED — no relay answered within 10s');
  assert.match((await away(tab, lend, nothing)).words, /Nothing on the way to your group/);
  assert.equal((await away({ mode: 'cors' }, lend, silent)).words, 'quiet');
});
