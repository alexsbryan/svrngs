// A lend's worker (web/lend.js) and its start page (web/open.js), with no
// browser: what the browser step's lend-worker tests held of their logic.
// Each is a page or a worker whose imports are a browser's, so their
// functions are cut from their text, as lost-answer.test.mjs and
// hand.test.mjs cut them, and run over stand-ins for what only a browser
// gives: a worker run, a page's load, the origin's store, a dial.
//
// - The hand-over, as Safari on an iPhone loses every key handed its worker
//   (browser/main.rs
//   `a_lend_worker_that_loses_every_key_handed_it_takes_the_holders_bytes_in_memory`):
//   open.js `start` → store.js `handOver` → lend.js's `message` and
//   `messageerror` listeners and `holdPass`, then the worker the browser
//   stopped (`holding`, `lent`, `unheldAt`), the start page again
//   (`handedKept`, `handedOn`), and the group's page signing to a new
//   holder (among.js `wanted`, hand.js `holderOf`). That a real WebKit
//   worker, stopped by closing the browser, comes back through its start
//   page with a real CryptoKey kept stays in
//   `a_lend_worker_that_reads_no_store_dials_with_the_pass_its_start_page_hands_it`.
// - A lent film's ranges and its lenders (`a_lent_film_comes_under_the_worker_which_reaches_each_lender_itself`):
//   `lent` → `proxied` → `headersFor`/`bounded` → a door's answer →
//   `served`. That a `<video>` plays, and its own ranges reach the door as
//   parts, is the playback step's (playback.rs).
// - A lend that answers only in gzip asked for gzip, and its page unpacked
//   (`a_phone_opens_a_lend_that_answers_only_in_gzip`, which keeps the
//   browser's DecompressionStream in WebKit's worker).
// - A lend's computer that did not answer shown as its person's window,
//   dark (`a_phone_is_shown_a_lend_s_computer_that_did_not_answer_as_its_window_dark`):
//   `away` filling away.html as built.
// - A socket opened again once its lender answers again
//   (`a_phone_s_socket_opens_again_once_its_lender_answers_again`):
//   `socket`/`switched` letting go of the dial a socket failed on.
// - What a lent page is told of the lends it reads (`told`, `toldOf`), of
//   `a_lent_page_is_told_the_lends_it_reads_and_opens_one_at_a_path`, which
//   keeps the navigation and its Back.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { register } from 'node:module';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { gzipSync } from 'node:zlib';
import { framing, fromHex, holderMatches, pkcs8, signerFor } from '../authority.js?v=__VERSION__';
import { silent, unreached } from '../../crates/svrngs-node/src/home/question.js?v=__VERSION__';

register('./pages/layout.mjs', import.meta.url);
const { takes: among } = await import('../page/among.js?v=__VERSION__');

const text = (path) => readFileSync(new URL(path, import.meta.url), 'utf8');
const LEND = text('../lend.js');
// A start page's script is a module: `import.meta.url` names it, as `META`.
const OPEN = text('../open.js').replaceAll('import.meta.url', 'META');
const STORE = text('../store.js').replaceAll(/^export /gm, '');
const HAND = text('../page/hand.js').replaceAll(/^export /gm, '');
// The page a lend's worker shows a tab when its computer did not answer, as
// scripts/build-web.sh:107 copies it beside the worker.
const AWAY = text('../../crates/svrngs-node/src/home/away.html');

// `name`, a function in `source`, up to its closing brace.
const defined = (source, name) => {
  const at = source.search(new RegExp(`^(async )?function ${name}\\(`, 'm'));
  assert.notEqual(at, -1, `declares ${name}`);
  return source.slice(at, source.indexOf('\n}\n', at) + 2);
};
// `name`, a constant in `source`, to its first statement's end; `block`,
// one whose arrow's body is a block.
const constant = (source, name) => {
  const found = new RegExp(`^const ${name} = [^]*?;\\n`, 'm').exec(source)?.[0];
  assert.ok(found, `declares ${name}`);
  return found;
};
const block = (source, name) => {
  const found = new RegExp(`^const ${name} = [^]*?\\n\\};\\n`, 'm').exec(source)?.[0];
  assert.ok(found, `declares ${name}`);
  return found;
};
// lend.js's listener for `type`, as it adds it to the worker.
const listener = (type) => {
  const found = new RegExp(`^scope\\.addEventListener\\('${type}', [^]*?\\n\\}\\);\\n`, 'm').exec(
    LEND,
  )?.[0];
  assert.ok(found, `lend.js listens for ${type}`);
  return found;
};

const OWN = 'page-label';
const ORIGIN = `https://${OWN}.page.example`;
const PAGE = 'https://page.example/';
const hex = (bytes) => Buffer.from(bytes).toString('hex');

// ── the hand-over ────────────────────────────────────────────────────────

const WORKER = [
  'let handed = null;',
  'const dials = new Map();',
  'let asks = 0;',
  'let endpoint = null;',
  ...['handedNow', 'PLAIN', 'REFUSED', 'say', 'unheld', 'fromHere', 'handsPass'].map((name) =>
    constant(LEND, name),
  ),
  ...['holding', 'lent', 'unheldAt', 'holdPass'].map((name) => defined(LEND, name)),
  listener('message'),
  listener('messageerror'),
  '({ lent, held: () => handed })',
].join('\n');

const HANDOVER = [
  constant(STORE, 'LOST'),
  constant(STORE, 'HANDING'),
  defined(STORE, 'handOver'),
  '({ LOST, handOver })',
].join('\n');

const START = [
  constant(OPEN, 'holderKey'),
  ...['decode', 'start', 'handedHere', 'withHolder', 'handedKept', 'handedOn'].map((name) =>
    defined(OPEN, name),
  ),
  'start',
].join('\n');

// A port pair that delivers at once, in turn, as the browser's does.
class Channel {
  port1 = {};
  port2 = { postMessage: (data) => this.port1.onmessage?.({ data }) };
}

// Any CryptoKey in `value`, as Safari's worker loses one (web/tests/
// handover.mjs `KEYS_LOST`; demo/runs/ios/2026-10-30-observe/keyprobe/phone.log).
const holdsKey = (value) =>
  value instanceof CryptoKey ||
  (value !== null && typeof value === 'object' && Object.values(value).some(holdsKey));

// A lend's origin on a phone: its store, its pages' log, its worker's runs
// and where its pages went. Its worker reads nothing of the store (Safari's
// on an iPhone, 2026-10-05; handover.mjs `PLANT`); with `keysLost` it loses
// every message that holds a key, `messageerror` fired in its place.
function lendOrigin({ keysLost }) {
  const kept = new Map();
  const logs = [];
  const traced = [];
  const went = [];
  const pages = [];
  const client = {
    url: `${ORIGIN}/open.html`,
    postMessage: (data) => pages.forEach((heard) => heard({ data })),
  };
  const { LOST, handOver } = runInNewContext(HANDOVER, {
    navigator: { serviceWorker: { addEventListener: (_, heard) => pages.push(heard) } },
    MessageChannel: Channel,
    // HANDING passes once all that is pending has run: a worker's answer,
    // or its word that a message was lost, comes in microtasks before it,
    // and one that never comes is a no at once, not five seconds on.
    setTimeout: (passed) => setTimeout(passed, 0),
    console: { debug: (line) => logs.push(line) },
  });
  let worker = null;
  let run = null;
  // A run of the worker, as the browser starts one: nothing held.
  const started = () => {
    const on = {};
    const lines = [];
    run = runInNewContext(WORKER, {
      scope: {
        addEventListener: (type, heard) => (on[type] = heard),
        clients: { matchAll: async () => [client] },
      },
      store: { lendKept: async () => undefined, LOST },
      SCOPE: new URL(`${ORIGIN}/`),
      SITE: '/',
      OWN,
      trace: (line) => (lines.push(line), traced.push(line)),
      crypto,
      Uint8Array,
      Response,
      URL,
      encodeURIComponent,
      cookieKept: () => assert.fail('no cookie here'),
      workerRun: () => assert.fail('no app worker here'),
      socket: () => assert.fail('no socket here'),
    });
    worker = {
      postMessage(data, [port]) {
        if (keysLost && holdsKey(data)) void on.messageerror();
        else void on.message({ source: client, ports: [port], data, waitUntil: () => {} });
      },
    };
    return { ...run, lines };
  };
  // The start page loaded at `search` and `hash`, run to its end.
  const opened = (search, hash = '') =>
    runInNewContext(START, {
      store: {
        labelOf: (hostname) => hostname.split('.')[0],
        lendKept: async (name) => kept.get(name),
        // IndexedDB keeps a clone, a CryptoKey as itself.
        lendKeep: async (name, value) => kept.set(name, structuredClone(value)),
        whole: async () => false,
        handOver,
      },
      location: {
        hash,
        search,
        pathname: '/open.html',
        hostname: new URL(ORIGIN).hostname,
        origin: ORIGIN,
        href: `${ORIGIN}/open.html${search}${hash}`,
        replace: (to) => went.push(to),
      },
      history: { replaceState: () => {} },
      navigator: { serviceWorker: { register: async () => ({}) } },
      running: async () => worker,
      // The group's signer, framed, said this browser issued the open.
      checked: async () => true,
      say: (words) => logs.push(`said: ${words}`),
      console: { debug: (line) => logs.push(line) },
      META: `${ORIGIN}/open.js?v=__VERSION__`,
      holderMatches,
      fromHex,
      pkcs8,
      crypto,
      atob,
      URL,
      URLSearchParams,
      TextDecoder,
      Uint8Array,
      JSON,
      encodeURIComponent,
    })();
  return { kept, logs, traced, went, started, opened, held: () => run.held() };
}

// What the group's page hands for `seed`, a new holder's (web/page/hand.js
// `handing`), as the fragment it opens the start page with.
async function handoff(seed) {
  const key = await crypto.subtle.importKey('pkcs8', pkcs8(seed), 'Ed25519', true, ['sign']);
  const { x } = await crypto.subtle.exportKey('jwk', key);
  const handed = {
    label: OWN,
    lends: [{ label: OWN, key: 'ada', person: 'ada', name: 'page', kind: 'k' }],
    pass: 'PASS',
    writes: null,
    group: 'g',
    holder: Buffer.from(x, 'base64url').toString('hex'),
    seed: hex(seed),
    raw: false,
    relays: [],
    page: PAGE,
    unanswered: [],
  };
  return {
    holder: handed.holder,
    hash: `#${Buffer.from(JSON.stringify(handed)).toString('base64url')}`,
  };
}

// A navigation to `url`, which a page's `fetch` cannot make.
const navigating = (url) => {
  const request = new Request(url);
  return new Proxy(request, {
    get: (target, name) => {
      if (name === 'mode') return 'navigate';
      const value = target[name];
      return typeof value === 'function' ? value.bind(target) : value;
    },
  });
};

const BYTES = 'and its holder’s bytes, imported here and kept nowhere';
const NEW_HOLDER = 'no bytes of it are here: the group’s page signs a pass to a new holder';
const HEARD_LOST = 'STORE: the worker said a message was lost';

// The browser step's hand-over, both of its runs (handover.mjs): opened
// from the group's page, the worker stopped, a page's fetch, the lend's
// address opened again; and, its key lost there, the group's page asked
// for a new holder.
async function handedOver(keysLost) {
  const origin = lendOrigin({ keysLost });
  const first = origin.started();
  const seed = crypto.getRandomValues(new Uint8Array(32));
  const { holder, hash } = await handoff(seed);
  await origin.opened('?v=__VERSION__', hash);
  const handed = origin.held();
  const keptPass = origin.kept.get(`pass ${OWN}`);

  // The browser stops the worker; one starts again, holding nothing.
  const stopped = origin.started();
  const probe = new URL(`${ORIGIN}/probe`);
  const unheld = await stopped.lent(new Request(probe), probe, { label: OWN, path: '/probe' });
  const home = new URL(`${ORIGIN}/`);
  const sent = await stopped.lent(navigating(home), home, { label: OWN, path: '/' });
  const back = new URL(sent.headers.get('location'));
  await origin.opened(back.search);
  // Its key lost again, the start page asked the group's page for a new
  // holder: the group's page forgets the one it held and signs anew.
  const anew = origin.went.at(-1).startsWith(PAGE)
    ? await handoff(crypto.getRandomValues(new Uint8Array(32)))
    : null;
  if (anew) await origin.opened('?v=__VERSION__&back=%2F', anew.hash);
  return {
    origin,
    holders: [holder, anew?.holder ?? holder],
    handed,
    keptPass,
    firstTrace: first.lines.join('\n'),
    stoppedTrace: stopped.lines.join('\n'),
    unheld,
    sent,
    back,
  };
}

for (const keysLost of [false, true]) {
  test(`the worker dials with the pass its start page hands it${keysLost ? ', its keys lost: as the holder’s bytes' : ''}`, async () => {
    const run = await handedOver(keysLost);
    const { origin, firstTrace, stoppedTrace } = run;
    const logged = (words) => origin.logs.some((line) => line.includes(words));
    assert.match(firstTrace, /the page handed over the pass/);
    assert.equal(
      firstTrace.includes(BYTES),
      keysLost,
      'the first run takes bytes only where the key was lost',
    );
    assert.ok(run.handed.key instanceof CryptoKey, 'the worker holds the holder as a key');
    assert.equal(run.handed.key.extractable, false, 'a key no script can copy');
    assert.ok(
      await holderMatches(run.handed.key, run.holders[0]),
      'the key the pass names as its holder',
    );
    assert.equal(
      logged(HEARD_LOST),
      keysLost,
      'the start page hears at once that its key was lost',
    );
    // What the lend's origin keeps (store.js `lendKeep`): the holder as a
    // key, never its bytes.
    assert.ok(run.keptPass.key instanceof CryptoKey);
    assert.ok(
      !Object.keys(run.keptPass)
        .filter((k) => run.keptPass[k] !== undefined)
        .includes('seed'),
    );

    // The stopped worker reads nothing and holds nothing: a page's fetch is
    // answered by the worker itself, marked svrngs's (mnp-1), a navigation
    // sent to its start page.
    assert.equal(run.unheld.status, 503);
    assert.equal(run.unheld.headers.get('x-svrngs-refused'), 'pass');
    assert.equal(run.sent.status, 302);
    assert.equal(
      `${run.back.origin}${run.back.pathname}${run.back.search}`,
      `${ORIGIN}/open.html?v=__VERSION__&for=${OWN}&back=%2F`,
    );
    assert.match(stoppedTrace, /is not kept here/);
    assert.match(stoppedTrace, /its start page reads it again/);
    assert.match(stoppedTrace, /the page handed over the pass/);
    assert.equal(
      logged(NEW_HOLDER),
      keysLost,
      'a new holder asked for only where the key was lost',
    );
    assert.equal(
      stoppedTrace.includes(BYTES),
      keysLost,
      'the stopped run takes bytes only where the key was lost',
    );
    const held = origin.held();
    assert.ok(
      await holderMatches(held.key, run.holders[1]),
      'the restarted worker holds the holder',
    );
    // The lend's address opened again, where it was.
    assert.equal(origin.went.at(-1), `${ORIGIN}/`);
    if (keysLost) assert.equal(origin.went.at(-2), `${PAGE}#open=${OWN}&anew&at=%2F`);
  });
}

// The group's page, asked by a start page holding no bytes of its key
// (open.js `handedOn`, `&anew`): it forgets the holder it kept (among.js
// `wanted`, its io `forget`), so `holderOf` makes a new one, its seed
// handed with the pass (hand.js).
test('a start page with no key the worker takes has the group’s page sign to a new holder', async () => {
  const state = {
    at: { among: 'unasked' },
    group: { name: 'house' },
    lent: [{ label: OWN, name: 'page', person: 'ada', key: 'ada', kind: 'k' }],
    people: [],
  };
  const asked = (wants) => among.wanted(state, { type: 'wanted', wants }).effects[0];
  assert.deepEqual(
    [asked({ open: OWN, at: '/', anew: '' }).run, asked({ open: OWN, at: '/' }).run],
    ['forget', 'send'],
  );
  const holders = new Map([[OWN, 'the one kept']]);
  const holderOf = runInNewContext(`${defined(HAND, 'holderOf')}\nholderOf`, {
    store: {
      holder: async (label, key) => {
        if (key === undefined) return holders.get(label) ?? undefined;
        holders.set(label, key);
      },
    },
    crypto,
    Uint8Array,
    trace: () => {},
  });
  const core = { links: () => true, actor: (seed) => `holder of ${hex(seed)}` };
  const kept = await holderOf(core, OWN, [{ version: 2 }]);
  assert.deepEqual([kept.holder, kept.seed], ['the one kept', null]);
  holders.delete(OWN);
  const anew = await holderOf(core, OWN, [{ version: 2 }]);
  assert.equal(anew.seed.length, 32);
  assert.equal(anew.holder, `holder of ${hex(anew.seed)}`);
  assert.equal(holders.get(OWN), anew.holder, 'kept for the next open');
});

// ── a lend's requests through its door ───────────────────────────────────

const SERVED = [
  ...[
    'PLAIN',
    'QUIET',
    'UNSURE',
    'READS',
    'ASK',
    'REFUSED',
    'DATA',
    'INERT',
    'APPEND',
    'PAGE_COOKIES',
    'PART',
    'FIRST',
    'SLICE',
    'ends',
    'baseOf',
    'say',
    'closed',
    'unheld',
    'posts',
    'live',
    'cookieFor',
    'partAt',
  ].map((name) => constant(LEND, name)),
  block(LEND, 'partAsked'),
  'let walled = null;',
  'const dials = new Map();',
  'let asks = 0;',
  ...[
    'whereOf',
    'told',
    'toldOf',
    'lent',
    'proxied',
    'headersFor',
    'bounded',
    'ranged',
    'counted',
    'bodyFor',
    'doorAnswer',
    'sent',
    'refusedAppend',
    'redirected',
    'served',
    'inert',
    'unpacked',
    'shimmed',
    'shim',
    'away',
    'dialFor',
    'lenderOf',
    'dialAt',
    'letGo',
    'socket',
    'switched',
    'relayed',
    'frameAt',
    'closedBy',
    'gathered',
    'frame',
    'headOf',
  ].map((name) => defined(LEND, name)),
  '({ whereOf, lent, told, bounded, socket })',
].join('\n');

// The worker at `own`'s origin, holding `lends` (the first its own), each
// lender's door answering as `door({ lender, method, path, headers })`
// says, a Response or thrown; a lender in `unreached` never dialled at all,
// as web/src/lib.rs:353 refuses a computer silent past the dial's cap.
function lendWorker({ lends, door = () => assert.fail('no door asked'), unreached: lost = [] }) {
  const own = lends[0].label;
  const scope = new URL(`https://${own}.page.example/`);
  const held = { lends, pass: 'PASS', holder: 'HOLDER', page: PAGE, unanswered: ['dee'] };
  const asked = [];
  const dialled = [];
  const traced = [];
  let shut = false;
  const made = [];
  const run = runInNewContext(SERVED, {
    SCOPE: scope,
    OWN: own,
    SITE: '/',
    SIGNER: signerFor(scope, '/'),
    self: { location: { origin: scope.origin } },
    store: { lendOrigin: (protocol, label, host) => `${protocol}//${label}.${host}` },
    holding: async () => held,
    handedNow: () => held,
    jarOf: async () => [],
    keepCookies: async () => {},
    appended: () => assert.fail('no acts here'),
    actsPosted: () => assert.fail('no acts posted here'),
    opening: () => assert.fail('no lend it reads opened here'),
    unheldAt: () => assert.fail('the pass is held'),
    passed: async () => [['x-svrngs-pass', 'PASS']],
    awayPage: async () => AWAY,
    failed: new Set(),
    // A dial to `lender`, as the wasm's `dial_lend` answers one.
    dialled: async (lender) => {
      dialled.push(lender);
      if (shut || lost.includes(lender))
        throw new Error('LAYER tls/alpn+relay: REFUSED — not answered in 4s: asleep');
      const dial = {
        alive: true,
        fetch: async (method, path, headers, body) => {
          asked.push({ lender, method, path, headers: Object.fromEntries(headers) });
          if (!dial.alive) throw new Error('connection lost');
          return door({ lender, method, path, headers: new Headers(headers), body });
        },
        socket: async (path) => {
          asked.push({ lender, method: 'SOCKET', path });
          if (!dial.alive) throw new Error('connection lost');
          return {
            status: 101,
            headers: [],
            readable: new ReadableStream({ start: (c) => c.close() }),
            writable: new WritableStream(),
          };
        },
      };
      made.push(dial);
      return dial;
    },
    trace: (line) => traced.push(line),
    framing,
    silent,
    unreached,
    crypto,
    btoa,
    Response,
    Headers,
    Blob,
    ReadableStream,
    WritableStream,
    DecompressionStream,
    TextEncoder,
    TextDecoder,
    URL,
    Date,
  });
  // What the page at `url` is answered, by the path the fetch listener takes.
  const fetched = (url, init, request = new Request(url, init)) => {
    const at = new URL(url);
    const where = run.whereOf(at);
    assert.ok(where, `${url} is a lend's`);
    return run.lent(request, at, where, !where.read);
  };
  // The way to every lender shut, as a closed lid leaves a computer reached
  // through it, each dial through it gone; and open again.
  const lid = (open) => {
    shut = !open;
    if (shut) for (const dial of made) dial.alive = false;
  };
  return { ...run, fetched, asked, dialled, traced, lid };
}

const page = { label: OWN, key: 'ada', person: 'ada', name: 'page', kind: 'k' };
const shelf = { label: 'shelf-label', key: 'cy', person: 'cy', name: 'shelf', kind: 'k' };
const SHELF = 'https://shelf-label.page.example';

// A file of `size` bytes at a door, its range answered 206 as upstream's
// Range origin answers (tests/door/origins.rs), the whole 200.
const FILM = 3 * 1024 * 1024;
const OTHER = 16 * 1024 * 1024;
const byteAt = (i) => (i * 7 + (i >> 10)) & 255;
const file = (size) => Uint8Array.from({ length: size }, (_, i) => byteAt(i));
const FILES = { 'title.mp4': file(FILM), 'title.bin': file(OTHER) };
function ranges({ path, headers }) {
  const bytes = FILES[path.split('/').at(-1)];
  const type = path.endsWith('.mp4') ? 'video/mp4' : 'application/octet-stream';
  const [, from, to] = /^bytes=(\d+)-(\d*)$/.exec(headers.get('range') ?? '') ?? [];
  if (from === undefined) return new Response(bytes, { headers: { 'content-type': type } });
  const end = Math.min(to ? Number(to) : bytes.length - 1, bytes.length - 1);
  return new Response(bytes.slice(Number(from), end + 1), {
    status: 206,
    headers: { 'content-type': type, 'content-range': `bytes ${from}-${end}/${bytes.length}` },
  });
}

// What a page's fetch of a range read: page.mjs `fetched`, runtime.mjs `range`.
async function read(res) {
  const bytes = new Uint8Array(await res.arrayBuffer());
  return {
    status: res.status,
    type: res.headers.get('content-type'),
    contentRange: res.headers.get('content-range'),
    bytes,
  };
}

test('a lend’s ranges are its own lender’s, asked as the page asked them, the answer whole', async () => {
  const worker = lendWorker({ lends: [page, shelf], door: ranges });
  const MIB = 1 << 20;
  const film = await read(
    await worker.fetched(`${ORIGIN}/title.mp4`, { headers: { range: `bytes=0-${MIB - 1}` } }),
  );
  assert.deepEqual(
    { ...film, bytes: film.bytes.length },
    { status: 206, type: 'video/mp4', contentRange: `bytes 0-${MIB - 1}/${FILM}`, bytes: MIB },
  );
  assert.deepEqual(
    film.bytes,
    FILES['title.mp4'].slice(0, MIB),
    'the film’s first MiB, byte for byte',
  );
  // A range of cy's shelf, which ada's page reads, at the shelf's own
  // origin: asked of cy's computer, never through ada's.
  const [from, to] = [1_000_000, 1_000_000 + 4 * MIB - 1];
  const other = await read(
    await worker.fetched(`${SHELF}/title.bin`, { headers: { range: `bytes=${from}-${to}` } }),
  );
  assert.equal(other.status, 206);
  assert.equal(other.contentRange, `bytes ${from}-${to}/${OTHER}`);
  assert.deepEqual(other.bytes, FILES['title.bin'].slice(from, to + 1));
  assert.deepEqual(
    worker.asked.map(({ lender, path, headers }) => [lender, path, headers.range]),
    [
      ['ada', `/lend/${OWN}/title.mp4`, `bytes=0-${MIB - 1}`],
      ['cy', '/lend/shelf-label/title.bin', `bytes=${from}-${to}`],
    ],
  );
  assert.deepEqual(worker.dialled, ['ada', 'cy'], 'each lender dialled itself');
});

test('an open range comes back a part, 206, named of the whole, never the rest of the file', async () => {
  const worker = lendWorker({ lends: [page, shelf], door: ranges });
  const from = 1_000_000;
  const part = await read(
    await worker.fetched(`${SHELF}/title.bin`, { headers: { range: `bytes=${from}-` } }),
  );
  const got = part.bytes.length;
  assert.equal(part.status, 206);
  assert.ok(got > 0 && got < OTHER - from, `${got} bytes, not the ${OTHER - from} to the end`);
  assert.equal(part.contentRange, `bytes ${from}-${from + got - 1}/${OTHER}`);
  assert.deepEqual(part.bytes, FILES['title.bin'].slice(from, from + got));
  // Each part that runs on asks twice the last, up to PART (lend.js:454).
  const sizes = [got];
  for (let at = from + got; sizes.length < 6; at += sizes.at(-1)) {
    const next = await read(
      await worker.fetched(`${SHELF}/title.bin`, { headers: { range: `bytes=${at}-` } }),
    );
    sizes.push(next.bytes.length);
  }
  const K = 1024;
  assert.deepEqual(sizes, [256 * K, 512 * K, 1024 * K, 2048 * K, 2048 * K, 2048 * K]);
  assert.ok(
    worker.asked.every(({ headers }) => /^bytes=\d+-\d+$/.test(headers.range)),
    'every range closed at the door',
  );
});

// What a `<video>` asks, which a page's fetch cannot: Chromium's
// `bytes=N-`, WebKit's `bytes=N-<the end>` (playback.rs). Each reaches the
// door a part of at most PART, 2 MiB; a closed range within it as asked.
test('a <video>’s every range reaches the door a part of at most 2 MiB', () => {
  const { bounded } = lendWorker({ lends: [page] });
  const asked = (range) => bounded([['Range', range]], 'video', '/title.mp4')[0][1];
  const PART = 2 * 1024 * 1024;
  assert.equal(asked('bytes=0-'), `bytes=0-${256 * 1024 - 1}`);
  assert.equal(asked('bytes=5000000-29999999'), `bytes=5000000-${5000000 + PART - 1}`);
  assert.equal(asked('bytes=100-199'), 'bytes=100-199');
  for (const range of ['bytes=0-', 'bytes=262144-', 'bytes=7-29999999']) {
    const [, from, to] = /^bytes=(\d+)-(\d+)$/.exec(asked(range));
    assert.ok(to - from + 1 <= PART, `${range} asked as ${asked(range)}`);
  }
});

test('whose word a refusal is goes through the worker as the door said it', async () => {
  const worker = lendWorker({
    lends: [page, shelf],
    door: ({ path }) =>
      path.endsWith('/private/x')
        ? new Response('lent without /private', {
            status: 403,
            headers: { 'x-svrngs-refused': 'path' },
          })
        : new Response('no', { status: 403 }),
  });
  const seen = [];
  for (const path of ['x', 'private/x']) {
    const res = await worker.fetched(`${SHELF}/${path}`);
    seen.push({ path, status: res.status, refused: res.headers.get('x-svrngs-refused') });
  }
  assert.deepEqual(seen, [
    { path: 'x', status: 403, refused: null },
    { path: 'private/x', status: 403, refused: 'path' },
  ]);
});

// llama-server's chat page, 415 unless asked gzip (browser/main.rs
// `gzip_only`, demo/runs/start/2026-10-13-story/ 9).
const CHAT = '<!doctype html><title>llama chat</title><p>Unpacked on the phone.';
test('a lend answering only in gzip is asked for it, but for a range, and its page unpacked', async () => {
  const worker = lendWorker({
    lends: [page],
    door: ({ headers }) =>
      /gzip/.test(headers.get('accept-encoding') ?? '')
        ? new Response(gzipSync(CHAT), {
            headers: { 'content-type': 'text/html; charset=utf-8', 'content-encoding': 'gzip' },
          })
        : new Response('gzip is not supported by this one', { status: 415 }),
  });
  const res = await worker.fetched(`${ORIGIN}/`);
  assert.equal(res.status, 200);
  assert.equal(res.headers.get('content-encoding'), null);
  const html = await res.text();
  assert.match(html, /<title>llama chat<\/title>/);
  assert.match(html, /Unpacked on the phone\./);
  await worker.fetched(`${ORIGIN}/`, { headers: { range: 'bytes=0-9' } });
  assert.deepEqual(
    worker.asked.map(({ headers }) => headers['accept-encoding'] ?? null),
    ['gzip', null],
  );
});

test('a tab whose lend’s computer did not answer is shown its person’s window, dark', async () => {
  const worker = lendWorker({ lends: [{ ...shelf }], unreached: ['cy'] });
  const res = await worker.fetched(`${SHELF}/`, undefined, navigating(`${SHELF}/`));
  assert.equal(res.status, 502);
  assert.equal(res.headers.get('x-svrngs-refused'), 'away');
  assert.equal(res.headers.get('content-type'), 'text/html; charset=utf-8');
  const html = await res.text();
  assert.equal(/<title>(.*)<\/title>/.exec(html)?.[1], 'cy’s window is dark');
  for (const words of [
    'cy’s computer didn’t answer, so cy’s shelf can’t open right now.',
    'Goodnight from cy.',
  ])
    assert.ok(html.includes(words), `the page lacks ${words}`);
  // Asleep past the dial's cap, it is not dialled again (lend.js
  // `doorAnswer`), so the tab waits one cap: DIAL_WITHIN, which the
  // computer's `lend_origin::a_lend_s_computer_asleep_is_said_away_within_the_cap`
  // holds, web/src/lib.rs:36 dialling by the same dial.rs.
  assert.deepEqual(worker.dialled, ['cy']);
});

// The shim carries a page's WebSocket to the worker (shim.js), which asks
// the lend's door to switch a socket over a dial (`switched`). One that
// fails lets go of its dial, as `lent` does, so the next dials afresh once
// the computer answers again (demo/runs/doc/2026-10-20/found/lid-45s.jsonl).
test('a socket opens again once its lender answers again, the page never loaded again', async () => {
  const worker = lendWorker({ lends: [page, shelf] });
  const opened = async () => {
    const told = [];
    await worker.socket(
      { url: 'wss://shelf-label.page.example/sync' },
      { postMessage: (m) => told.push(m) },
    );
    // Told in the worker's realm, read into this one.
    return { ...told[0] };
  };
  assert.deepEqual(await opened(), { open: '' }, 'a socket opens');
  worker.lid(false);
  const stopped = await opened();
  assert.notDeepEqual(stopped, { open: '' }, 'and does not while its computer cannot be reached');
  assert.equal(stopped.closed, 1006);
  worker.lid(true);
  assert.deepEqual(await opened(), { open: '' }, 'and opens again once it can');
  assert.deepEqual(
    worker.dialled,
    ['cy', 'cy'],
    'the dial it failed on let go, the next dialled afresh',
  );
});

test('a lent page is told the lends it reads and who did not answer, never the pass or its holder', async () => {
  const worker = lendWorker({ lends: [page, shelf] });
  const res = await worker.told();
  const body = await res.text();
  assert.equal(res.status, 200);
  for (const secret of ['PASS', 'HOLDER']) assert.ok(!body.includes(secret), body);
  assert.deepEqual(JSON.parse(body), {
    lends: [
      { kind: 'k', name: 'shelf', lender: 'cy', address: `${ORIGIN}/svrngs-reads/shelf-label/` },
    ],
    unanswered: ['dee'],
  });
});
