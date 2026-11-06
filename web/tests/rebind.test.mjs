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
    // The wasm's version, which the page sends as x-svrngs-version
    // (SCOPE_PARALLEL.md): the merged dial asks it of every list.
    current_version: () => 5,
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

// Review F4: a guest's lent apps named the link's first relay as the
// lender's, so the page's and the worker's `reach` took that one route.
// They name none, so every route the link carries is raced (spec item 5).
// Watched failing at 5cd42620: the lend named https://fallback/.
test("a guest's lends name no one relay, so every route the link carries is raced", () => {
  const stay = runInNewContext(`${cut('page/visit.js', 'stay')}\nstay;`, {});
  const core = {
    issuer_of: () => 'ab',
    relays_of: () => ['https://fallback/', 'https://own/'],
    label: (key, name) => `${name}.${key}`,
  };
  const visit = { lent: { films: 'jellyfin' }, host: 'Kim', until: 0, version: 1 };
  assert.deepEqual(
    Array.from(stay(core, 'token', visit).stay.lent, (lend) => lend.relay),
    [null],
  );
});

// Review F5: a pass kept before relay fallbacks names `relay` and no
// `relays`, and the wasm glue reads `relays.length` for its Vec<String>
// (svrngs_web.js `passArrayJsValueToWasm0`), so the worker's open threw.
// Watched failing at 11c5c342: "Cannot read properties of undefined".
test('a pass kept before relay fallbacks opens at its one relay', async () => {
  const context = {
    handed: { seed: 'ab', relay: 'https://own/' },
    init: async () => {},
    store: { runtime: () => '' },
    fromHex: () => new Uint8Array(32),
    trace: () => {},
    // As the glue passes a Vec<String>: its length, then each.
    holder_endpoint: async (_, relays) =>
      Array.from({ length: relays.length }, (_, i) => relays[i]),
  };
  const opened = runInNewContext(
    `let loaded, linked;\n${cut('lend.js', 'opened')}\nopened;`,
    context,
  );
  assert.deepEqual(Array.from(await opened()), ['https://own/']);
});

// Review F6: a relay the record names for a lend that the page's policy
// refuses went to the worker as none, unsaid, and the worker said the
// group names none. The page says it refused that one, by name. Watched
// failing at fd50dd58: nothing traced.
test('a relay the page refuses for a lend is said by name as it goes as none', () => {
  const said = [];
  const lent = [{ label: 'films.ab', name: 'Films', relay: 'http://10.0.0.2:3340/' }];
  const context = { trace: (line) => said.push(line) };
  const passLends = runInNewContext(
    `${cut('page/hand.js', 'passLends')}${cut('page/hand.js', 'taken')}\npassLends;`,
    context,
  );
  const lends = passLends({ relay_taken: () => false }, { lent, label: 'films.ab' }, lent[0], {});
  assert.equal(lends[0].relay, null);
  assert.ok(
    said.some((line) => line.includes('Films') && line.includes('http://10.0.0.2:3340/')),
    said.join('\n'),
  );
});

// KEEP_GOING_AT_HOME 4: relays learned in a read reach the next dial, not
// the next visit, and the endpoint they replace is closed, as a relay keeps
// one per key. Watched failing with `endpoint` keeping what it bound: bound
// once, and the second dial asked through the link's relays alone.
test("the page's endpoint binds again, closing the old, when the group's relays change", async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const opened = [];
  const closed = [];
  const reached = { fetch: async () => 'lends', close: async () => {} };
  const core = {
    relay_of: () => 'https://relay.example/',
    issuer_of: () => 'ab',
    endpoint: async (_, __, meets) => {
      opened.push(meets);
      return { dial_link: async () => reached, close: async () => closed.push(meets) };
    },
    current_version: () => 5,
  };
  hold(new Uint8Array(32));
  const group = { token: 'token', name: 'House' };
  await lends(core, group, []);
  await lends(core, group, []);
  assert.deepEqual(opened, [undefined], 'nothing changed, so one endpoint');
  const home = ['https://home.example/'];
  await lends(core, { ...group, meets: home }, []);
  await lends(core, { ...group, meets: home }, []);
  assert.deepEqual(opened, [undefined, home]);
  await new Promise((done) => setImmediate(done));
  assert.deepEqual(closed, [undefined], 'the first closed, and the second kept');
});
