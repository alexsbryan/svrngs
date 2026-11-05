// A refused bind is let go, so the next dial binds again (review F3): the
// wasm's bind closes its endpoint and refuses when no relay answers within
// dial::LINK_WITHIN (web/src/lib.rs `bind`), and the page's one endpoint
// (web/page/dial.js `endpoint`) and a lend worker's (web/lend.js) were
// cached as that rejected promise until the group was opened again. Each
// asked twice, the first bind refused: the second dial binds and is
// answered. Watched failing at d4733200: both refused the second dial
// with the first bind's error, bound once.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { register } from 'node:module';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';

register('./pages/layout.mjs', import.meta.url);
const { hold, lends } = await import('../page/dial.js?v=__VERSION__');

// A bind that is refused the first time and answers after.
const binder = () => {
  const bound = { binds: 0 };
  bound.bind = (dialler) =>
    ++bound.binds === 1 ? Promise.reject(new Error('no relay answered')) : Promise.resolve(dialler);
  return bound;
};

test("the page's dial binds again after a refused bind", async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const bound = binder();
  const reached = { fetch: async () => 'lends', close: async () => {} };
  const core = {
    relay_of: () => 'https://relay.example/',
    issuer_of: () => 'ab',
    endpoint: () => bound.bind({ dial_link: async () => reached }),
  };
  hold(new Uint8Array(32));
  const group = { token: 'token', name: 'House' };
  await assert.rejects(lends(core, group, []));
  assert.equal((await lends(core, group, []))[2], 'lends');
  assert.equal(bound.binds, 2);
});

// The top-level function or arrow `name` as web/`file` declares it, up to
// its closing brace, or nothing where it declares none.
const cut = (file, name) => {
  const text = readFileSync(new URL(`../${file}`, import.meta.url), 'utf8');
  const at = text.search(new RegExp(`^((async )?function ${name}\\(|const ${name} = )`, 'm'));
  return at < 0 ? '' : text.slice(at, text.indexOf('\n}', at) + 3);
};

test("a lend worker's dial opens its endpoint again after a refused open", async () => {
  const binding = binder();
  const context = { binding, trace: () => {}, handed: { lends: [] } };
  const source = `let endpoint = null;
    const opened = () => binding.bind({ dial_lend: async () => 'reached' });
    ${cut('lend.js', 'dialled')}${cut('lend.js', 'bound')}
    dialled;`;
  const dialled = runInNewContext(source, context);
  await assert.rejects(dialled('ab'));
  assert.equal(await dialled('ab'), 'reached');
  assert.equal(binding.binds, 2);
});
