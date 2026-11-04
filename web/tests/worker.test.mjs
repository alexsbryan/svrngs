// The page's worker's two decisions, without a browser (web/sw.js): this
// build's shell from its cache, a different build's from the network, and an
// update moving the open pages onto the build it kept. On 2026-11-03 a phone
// that had been open since 6 October met a device link: its cached page
// refused the answer's `person` field, because the worker matched the shell
// with `ignoreSearch: true`, so the old build answered requests for the new
// one's URLs, and nothing reloaded the page onto the build the node spoke.
import assert from 'node:assert/strict';
import { test } from 'node:test';

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
