// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's worker: the runtime, at the lend's own origin (DESIGN §3,
// "Lent apps", calls 1, 3 and 6; §7). Moved from the page's worker, where
// every lend ran under the origin that holds each person's key (sw.js).
//
// It serves every request under its scope — the lend's pages, their
// subresources, fetch and XHR, a `<video>`'s byte ranges — by HTTP over a
// dial to the lender's computer, at its door's `/lend/<label>/<path>`, on
// the pass holders' protocol, as a key of this run's that the pass's
// holder names in a link, presenting the pass the page signed and the link
// (open.js hands the pass and the holder over); the door lets it in as the
// person. A request a lent page makes to another lend's address is
// served the same way when the pass names that lend too, which its lender
// declared it reads; else refused. Each answer goes back as the door gave
// it, a 206 and its `Content-Range` with it, its body streamed.
//
// Its pages are told the lends they read at `READS` on their own origin
// (DESIGN §3, "Lent apps", call 6): each one's kind, name, lender
// and an address under `READS` to ask it at, and whose computer did not
// answer — what the group's page handed, never the pass or its key. A
// request at such an address is that lend's; a page navigated there opens
// that lend at the path asked, through the group's page, which signs its
// pass, and its start page (open.js), which goes there.
//
// A lend that asks (DESIGN §3, "What an app gets") is told with
// its lender's words, `asks`, and `ask`, `<origin>/svrngs-ask?lend=&app=`,
// the shape a computer's `/svrngs-reads` gives (surface.rs `reads`), to
// which the page adds `&back=<its path>`: navigated, it opens the group's
// page at `#ask=<lend>&app=<this lend>&at=<back>`, where only the person
// says yes. A page stops its app using a lend, `POST` at `stop`,
// `<origin><READS>/<label>/svrngs-stop`, told beside `ask`, sent to that
// lend's door with the pass, whose app is this lend; the door takes no yes
// with a pass.
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
// It is kept in this origin's store as it is written, so it outlives the
// run of the worker that wrote it: each run starts its own lines, after
// the last runs' (a worker WebKit ended took its trace with it,
// demo/runs/u5/2026-10-06-worker-gone/).
import init, {
  actor,
  app_act,
  holder_endpoint,
  link,
  links,
  unsigned_line,
} from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';
import { silent, unreached } from './question.js?v=__VERSION__';
import { framing, fromHex, maySign, openMessage, signerFor } from './authority.js?v=__VERSION__';
/** @import { Handoff, Passed } from './authority.js?v=__VERSION__' */
/** @import { Dialled, Endpoint } from './wasm/svrngs_web.js?v=__VERSION__' */

// `self` as a service worker's: the worker's lib names it a
// `WorkerGlobalScope`, which every kind of worker shares
// (jsconfig.workers.json). Its own first, so `addEventListener` gives each
// listener its event's type.
/** @typedef {ServiceWorkerGlobalScope & typeof self} Scope */
const scope = /** @type {Scope} */ (self);

// The open the page hands over (open.js), as this worker holds it: the
// holder's key imported, and the lines it signed, the last and those not
// yet carried (`writeSigned`).
/**
 * @typedef {Handoff & {
 *   key: CryptoKey,
 *   line?: { seq: number, id: string },
 *   unsent?: Unsent[],
 * }} Held
 */
// A signed line not yet carried to the lend's computer: its group, and the
// line, of which this worker reads its seq and id.
/** @typedef {[string, { seq: number, id: string }]} Unsent */
// The lend a request is at (`at`), its path there, and whether it is one
// this lend's pages read, asked under `READS`.
/** @typedef {{ label: string, path: string, read?: boolean }} Where */
// A cookie kept in a lend's jar (`setCookie`): `expires` in ms, none for
// the session's; `page` whether its pages may read it.
/**
 * @typedef {{
 *   name: string,
 *   value: string,
 *   path: string,
 *   expires: number | null,
 *   page: boolean,
 * }} Cookie
 */
// A header sent through the door, as the dial takes it: `[name, value]`.
/** @typedef {[string, string]} Header */
// How a range's trace says what came of it (`ranged`).
/** @typedef {(what: string) => void} Said */
// What this worker's own answer says it is: the three of the door's kinds
// it gives (door.rs, `LendRefusal`).
/** @typedef {'pass' | 'closed' | 'away'} Refusal */
// An app's own worker's listener, as `appWorker` hands it its events.
/** @typedef {(event: object) => void} Listener */
/** @typedef {Record<'fetch' | 'install' | 'activate' | 'message', Listener[]>} On */

const SCOPE = new URL(scope.registration.scope);
// Where the page's files are on this origin, and those it saves, so a phone
// reopens with the public site gone (KEEP_GOING_AT_HOME 10). `self.location`
// is a `WorkerLocation`, which `URL` reads as its `href`.
const SITE = new URL('./', /** @type {string} */ (/** @type {unknown} */ (self.location))).pathname;
// This worker's lend: the first name of its address.
const OWN = store.labelOf(SCOPE.hostname);
const SIGNER = signerFor(SCOPE, SITE);
const PLAIN = { 'content-type': 'text/plain; charset=utf-8' };
const QUIET = `${silent('The computer lending this')} Try again later.`;
const TRACE = `${SCOPE.pathname}svrngs-trace`;
// Where its pages ask the lends they read, as a computer's origin answers
// them too (crates/svrngs-node/src/surface.rs, `READS_PATH`).
const READS = `${SCOPE.pathname}svrngs-reads`;
const ASK = `${SCOPE.pathname}svrngs-ask`;
const STOP = new RegExp(`^${READS}/([^/]+)/svrngs-stop$`);

// Enough for a film's seeks, each range's lines with them, over
// this run and the last.
const LINES = 2000;
/** @type {string[]} */
const traced = [];
// The last runs' lines, read once; this run's are kept after them, a
// second at most after each is written.
const before = store.lendKept(`trace ${OWN}`).then(
  (lines) => lines ?? [],
  () => [],
);
/** @type {ReturnType<typeof setTimeout> | null} */
let keeping = null;
const kept = async () => [...(await before), ...traced].slice(-LINES);
self.__trace = (line) => {
  console.debug(line);
  traced.push(`${new Date().toISOString()} ${line}`);
  if (traced.length > LINES) traced.shift();
  keeping ??= setTimeout(async () => {
    keeping = null;
    await store
      .lendKeep(`trace ${OWN}`, await kept())
      .catch((error) => console.debug(`RUNTIME: the trace was not kept: ${error}`));
  }, 1000);
};
const trace = self.__trace;
trace(`RUNTIME: this run of the worker begins`);

// This build's pieces kept whole or not at all: one that fails fails the
// install, so an older worker keeps serving and open.js says this could not
// open, and a worker that runs has everything it saves.
async function installed() {
  const cache = await caches.open(store.CACHE);
  await cache.addAll(store.SAVED.map(store.savedAt));
  trace(`RUNTIME: this build's ${store.SAVED.length} pieces are saved here`);
  await scope.skipWaiting();
}

// Older builds' caches gone, and only those: a lend's own pages keep theirs
// on this origin. Unlike the page's worker it reloads nothing, since a page
// here is a person's app at work. This worker's script is one address
// whatever the build (open.js `handedOn`), so the browser finds a new one
// itself after a navigation here; until then this worker sends a newer
// build's requests to the network, and they go on working.
async function activated() {
  const names = await caches.keys();
  const older = names.filter((name) => name.startsWith(store.CACHES) && name !== store.CACHE);
  await Promise.all(older.map((name) => caches.delete(name)));
  trace(`RUNTIME: ${older.length} older build(s) dropped`);
  await scope.clients.claim();
}

scope.addEventListener('install', (event) => event.waitUntil(installed()));
scope.addEventListener('activate', (event) => event.waitUntil(activated()));

// What this run of the worker holds, all of it lost when the browser stops
// it: what the page handed it, the wasm, its endpoint, a dial per lender,
// each lend's cookies, the app's worker.
//
// `handed` is none until a page hands it over or `holding` reads it back,
// and is read as held: every reader but `holding` and `whereOf` runs after
// `holding` found it, or after a page handed it, as `lent` and `socket` do.
// Those two read it as it is, by `handedNow`.
/** @type {Held} */
let handed = /** @type {Held} */ (/** @type {unknown} */ (null));
const handedNow = () => /** @type {Held | null} */ (handed);
/** @type {ReturnType<typeof init> | null} */
let loaded = null;
/** @type {Promise<Endpoint> | null} */
let endpoint = null;
/** @type {string | null} */
let linked = null;
/** @type {Map<string, Promise<Dialled>>} */
const dials = new Map();
/** @type {Map<string, Cookie[]>} */
const jars = new Map();
/** @type {On | null | undefined} */
let app;
/** @type {Promise<unknown>} */
let signing = Promise.resolve();
// The last page served whose own `<meta>` sets a policy (`lent`).
/** @type {string | null} */
let walled = null;

// Where `label`'s pages are: this worker's lend under its scope; another
// at an address of its own beside this one.
/** @param {string} label */
const baseOf = (label) =>
  label === OWN
    ? SCOPE.href
    : `${store.lendOrigin(SCOPE.protocol, label, SCOPE.host.slice(OWN.length + 1))}/`;

// The lend `url` is at, and its path there; or none. Under `READS`, a
// lend this page reads, by its label.
/**
 * @param {URL} url
 * @returns {Where | null}
 */
function whereOf(url) {
  const reading =
    url.origin === SCOPE.origin ? url.pathname.match(new RegExp(`^${READS}/([^/]+)(/.*)?$`)) : null;
  const [, read, path] = reading ?? [];
  if (read) return { label: read, path: `${path ?? '/'}${url.search}`, read: true };
  for (const label of [OWN, ...(handedNow()?.lends ?? []).map((lend) => lend.label)]) {
    const base = new URL(baseOf(label));
    if (url.origin === base.origin && url.pathname.startsWith(base.pathname)) {
      return { label, path: `/${url.pathname.slice(base.pathname.length)}${url.search}` };
    }
  }
  return null;
}

/** @returns {Promise<Held | null>} */
async function holding() {
  if (handedNow()) return handed;
  try {
    handed = (await store.lendKept(`pass ${OWN}`)) ?? null;
    const found = handedNow() ? 'read from this origin’s store' : 'is not kept here';
    trace(`RUNTIME: the pass for ${OWN} ${found}`);
  } catch (error) {
    trace(`RUNTIME: this origin's store cannot be read here: ${String(error)}`);
  }
  return handed;
}

/**
 * @param {string} lender
 * @returns {Promise<Dialled>}
 */
function dialFor(lender) {
  if (!dials.has(lender)) {
    const opening = dialled(lender);
    dials.set(lender, opening);
    opening.catch(() => dials.delete(lender));
  }
  // Set just above when it was not.
  return /** @type {Promise<Dialled>} */ (dials.get(lender));
}

// A new dial to `lender`, through the relay its group names for it.
/**
 * @param {string} lender
 * @returns {Promise<Dialled>}
 */
async function dialled(lender) {
  trace(`RUNTIME: dialling ${lender} for the pass's holder`);
  const relay = handed.lends.find((lend) => lend.key === lender)?.relay ?? null;
  return (await bound()).dial_lend(lender, relay);
}

// This run's endpoint, opened at the first dial; one refused (no relay
// answered in time, web/src/lib.rs `bind`) is let go, so the next dial
// opens one again rather than the refusal for good (review F3).
function bound() {
  if (endpoint) return endpoint;
  const opening = (endpoint = opened());
  opening.catch((error) => {
    if (endpoint !== opening) return;
    trace(`RUNTIME: the endpoint was not opened, so the next dial opens one: ${error}`);
    endpoint = null;
  });
  return opening;
}

// This run's endpoint (DESIGN §3): a key of this run's alone, which the
// holder, a key no page can copy, names in a link the door checks beside
// the pass (`linked`); or, when a lend of the open is at a door taking no
// link, the holder's own bytes, as a door before links took.
async function opened() {
  await (loaded ??= init({ module_or_path: store.runtime() }));
  const seed = handed.seed;
  const own = !seed;
  const bytes = seed ? fromHex(seed) : crypto.getRandomValues(new Uint8Array(32));
  linked = null;
  if (own) {
    const to = actor(bytes);
    const sig = new Uint8Array(
      await crypto.subtle.sign(
        'Ed25519',
        handed.key,
        // The glue's bytes are a copy of its own, in a buffer of their own.
        /** @type {Uint8Array<ArrayBuffer>} */ (link(handed.pass, to)),
      ),
    );
    linked = new TextDecoder().decode(link(handed.pass, to, sig));
  }
  const as = own
    ? 'of this run, named by the holder’s link'
    : 'as the pass’s holder, for an older door';
  trace(`RUNTIME: opening the endpoint ${as}`);
  // A pass kept before relay fallbacks names its one relay, and no `relays`.
  if (!handed.relays) trace('RUNTIME: the pass names one relay, as one kept before fallbacks');
  return holder_endpoint(bytes, handed.relays ?? (handed.relay ? [handed.relay] : []));
}

// The pass a request carries, and the link naming this run's endpoint
// when the holder signed one (`opened`).
/**
 * @param {Held} handoff
 * @returns {Promise<Header[]>}
 */
async function passed(handoff) {
  await bound();
  /** @type {Header[]} */
  const headers = [['x-svrngs-pass', handoff.pass]];
  if (linked) headers.push(['x-svrngs-link', linked]);
  return headers;
}

// ── cookies, a jar per lend ─────────────────────────────────────────────

/**
 * @param {string} label
 * @returns {Promise<Cookie[]>}
 */
async function jarOf(label) {
  if (!jars.has(label)) {
    const stored = await store.lendKept(`cookies ${label}`).catch(() => null);
    jars.set(label, stored ?? []);
  }
  // Set just above when it was not.
  return /** @type {Cookie[]} */ (jars.get(label));
}

/** @param {Cookie} cookie */
const live = (cookie) => cookie.expires === null || cookie.expires > Date.now();

// Keep a `Set-Cookie` value in `jar`: its name, value, path, lapse and
// whether pages may read it. Any other attribute is the browser's, and
// there is no other site here to keep it from.
/**
 * @param {Cookie[]} jar
 * @param {string} line
 */
function setCookie(jar, line) {
  // `split` gives at least one part, the empty string's included.
  const [pair, ...attrs] = /** @type {[string, ...string[]]} */ (line.split(';'));
  const equals = pair.indexOf('=');
  if (equals < 1) return;
  /** @type {Cookie} */
  const cookie = {
    name: pair.slice(0, equals).trim(),
    value: pair.slice(equals + 1).trim(),
    path: '/',
    expires: null,
    page: true,
  };
  for (const attr of attrs) {
    const [name, ...rest] = /** @type {[string, ...string[]]} */ (attr.split('='));
    const key = name.trim().toLowerCase();
    const value = rest.join('=').trim();
    if (key === 'path' && value.startsWith('/')) cookie.path = value;
    if (key === 'max-age') cookie.expires = Date.now() + Number(value) * 1000;
    if (key === 'expires' && cookie.expires === null) cookie.expires = Date.parse(value);
    if (key === 'httponly') cookie.page = false;
  }
  const others = jar.filter((one) => one.name !== cookie.name || one.path !== cookie.path);
  // A cookie set already lapsed takes the one it names out, and is not kept.
  if (live(cookie)) others.push(cookie);
  jar.splice(0, jar.length, ...others);
}

/**
 * @param {Cookie[]} jar
 * @param {string} path
 * @param {boolean} page whether a page reads them, which sees no `HttpOnly` one
 */
const cookieFor = (jar, path, page) =>
  jar
    .filter((cookie) => live(cookie) && path.startsWith(cookie.path) && (cookie.page || !page))
    .map((cookie) => `${cookie.name}=${cookie.value}`)
    .join('; ');

/** @param {string} label */
async function keepJar(label) {
  const jar = await jarOf(label);
  await store
    .lendKeep(`cookies ${label}`, jar)
    .catch((error) => trace(`RUNTIME: cookies not kept: ${error}`));
}

// ── a request, through the door ─────────────────────────────────────────

// A range that runs on is asked of the door as a part of at most PART: an
// open-ended `bytes=N-`, and a `<video>`'s closed
// range longer than a part, WebKit's `bytes=N-<end>`. A page's own fetch
// of a closed range gets what it asked.
//
// An open-ended range's part is FIRST where no part of the file ended just
// there — a seek, or a first look — and twice the last each time the
// reading runs on from where the last ended, up to PART. Chromium
// lets a seek's request go without telling the worker (neither engine
// aborts its fetch event), and the door sent each one's 2 MiB into a
// phone's link ahead of the last seek's bytes; each now costs it at most
// FIRST. A closed range stays a part of PART: WebKit, which asks those,
// took longer to play again when its parts started smaller, and read them
// through when they were whole (demo/runs/u5/2026-10-06-seek-stall/).
const PART = 2 * 1024 * 1024;
const FIRST = 256 * 1024;
// Where a part of a file ends, `<path> <offset>`, and that part's size.
/** @type {Map<string, number>} */
const ends = new Map();
// The part a range of `path` asks from `from`: PART for a closed range,
// else FIRST, or twice the part that ended at `from`, up to PART.
/**
 * @param {string} path
 * @param {number} from
 * @param {boolean} closed
 */
const partAt = (path, from, closed) =>
  closed ? PART : Math.min(PART, 2 * (ends.get(`${path} ${from}`) ?? FIRST / 2));
// A part of `path` from `from`, of `size` bytes, kept as where the next
// part runs on from, the oldest let go of past 64.
/**
 * @param {string} path
 * @param {number} from
 * @param {number} size
 */
const partAsked = (path, from, size) => {
  ends.delete(`${path} ${from}`);
  ends.set(`${path} ${from + size}`, size);
  if (ends.size > 64) ends.delete(/** @type {string} */ (ends.keys().next().value));
};

/**
 * @param {Header[]} headers
 * @param {RequestDestination} destination
 * @param {string} path
 * @returns {Header[]}
 */
function bounded(headers, destination, path) {
  return headers.map(
    /** @returns {Header} */ ([name, value]) => {
      const range =
        name.toLowerCase() === 'range' ? /^bytes=(\d+)-(\d*)$/.exec(value.trim()) : null;
      if (!range) return [name, value];
      const from = Number(range[1]);
      const ended = range[2] !== '';
      const media = destination === 'video' || destination === 'audio';
      const size = partAt(path, from, ended);
      const longer = media && Number(range[2]) - from + 1 > size;
      if (ended && !longer) return [name, value];
      partAsked(path, from, size);
      const asker = destination || 'a fetch';
      trace(`RUNTIME: ${value} for ${asker} runs on: asked as a part of ${size} bytes`);
      return [name, `bytes=${from}-${from + size - 1}`];
    },
  );
}

// A range, or a media element's ask, traced from asked to its end:
// answered, read whole, cancelled by the browser after so many
// bytes, or let go of by the browser before its answer came — what a seek
// decides, which the dial's own lines do not show. None for any other
// request.
let asks = 0;
/**
 * @param {Request} request
 * @param {string} path
 * @returns {Said | null}
 */
function ranged(request, path) {
  const media = ['video', 'audio'].includes(request.destination);
  if (!request.headers.has('range') && !media) return null;
  asks += 1;
  const ask = asks;
  const t0 = Date.now();
  /** @type {Said} */
  const said = (what) => trace(`RANGE ${ask}: ${what}, ${Date.now() - t0} ms`);
  const range = request.headers.get('range') ?? 'whole';
  said(`asked ${path} ${range} for ${request.destination || 'a fetch'}`);
  request.signal.addEventListener('abort', () => said('the browser let go of it'));
  return said;
}

// `body`, its reading traced by `said`.
/**
 * @param {ReadableStream<Uint8Array>} body
 * @param {Said} said
 */
function counted(body, said) {
  const reader = body.getReader();
  let bytes = 0;
  let cancelled = false;
  return new ReadableStream({
    async pull(controller) {
      const { value, done } = await reader.read().catch((error) => {
        said(`the body failed after ${bytes} bytes: ${error}`);
        return Promise.reject(error);
      });
      if (done && !cancelled) {
        said(`read whole, ${bytes} bytes`);
        controller.close();
      }
      if (done) return;
      bytes += value.byteLength;
      controller.enqueue(value);
    },
    cancel: (why) => {
      cancelled = true;
      said(`cancelled by the browser after ${bytes} bytes: ${why}`);
      return reader.cancel(why);
    },
  });
}

// `blob`'s bytes, a SLICE at a time as the dial takes them, traced when it
// has taken them all. WebKit's `blob.stream()` gives a 64 MiB body
// as one chunk, which the runtime copies into its wasm memory, and wasm
// memory never shrinks: each upload left the worker's process 135-320 MB
// larger, and past 1 GB WebKit ended it ('Suspended WebProcess is exiting
// due to memory pressure'), sending the request it held to the network
// (demo/runs/u5/2026-10-06-worker-gone/).
const SLICE = 256 * 1024;
/**
 * @param {Blob} blob
 * @param {string} what
 */
function sent(blob, what) {
  const t0 = Date.now();
  let at = 0;
  trace(`RUNTIME: ${what} sends a body of ${blob.size} bytes`);
  return new ReadableStream({
    async pull(controller) {
      if (at >= blob.size) {
        trace(`RUNTIME: ${what} handed its body to the dial whole, ${Date.now() - t0} ms`);
        controller.close();
        return;
      }
      const slice = blob.slice(at, at + SLICE);
      at += SLICE;
      controller.enqueue(new Uint8Array(await slice.arrayBuffer()));
    },
  });
}

// The header an answer that set cookies carries to the shim (shim.js).
const PAGE_COOKIES = 'x-svrngs-page-cookies';

// svrngs's own answer in the lend's place, marked with its kind as the
// door marks its own (door.rs, `LendRefusal`), so a page tells it from the
// lend's.
const REFUSED = 'x-svrngs-refused';
// The acts a lent app writes as the person asking (`appended`).
const APPEND = 'x-svrngs-append';
/**
 * @param {number} status
 * @param {string} words
 * @param {Refusal} kind
 */
const say = (status, words, kind) =>
  new Response(words, { status, headers: { ...PLAIN, [REFUSED]: kind } });
// The two refusals a page meets at the pass and the lend, each in one set of
// words (ralph/REVIEW_FINDINGS.md #111): no pass held, and a lend the pass does not open.
const unheld = () => say(503, 'Open this again from your group’s page.', 'pass');
const closed = () => say(403, 'This is not open to this page.', 'closed');

// The shim (shim.js) for a lend's page, with the cookies its pages read,
// and the nonce of the page's script policy when it names one: under
// `script-src 'nonce-…' 'strict-dynamic'` (Nextcloud's) a script without
// it does not run.
/**
 * @param {string} label
 * @param {Cookie[]} jar
 * @param {string | null} policy
 */
function shim(label, jar, policy) {
  const nonce = /'nonce-([^']+)'/.exec(policy ?? '')?.[1];
  const nonced = nonce ? ` nonce="${nonce}"` : '';
  const cookies = encodeURIComponent(cookieFor(jar, '/', true));
  return `<script${nonced} src="${SITE}shim.js?v=__VERSION__&l=${label}&c=${cookies}"></script>`;
}

// A request served from a lend, as each step of `lent` reads it: the
// request, the lend `where` names and the pass holds, whether the request
// may name acts (`signs`), the lend's cookie jar, and the path at its door.
/**
 * @typedef {object} Asked
 * @property {Request} request
 * @property {Where} where
 * @property {Passed} lend
 * @property {boolean} signs
 * @property {Cookie[]} jar
 * @property {string} path `/lend/<label><path>`
 */

// Serve `request`, at `url`, from the lend `where` names.
/**
 * @param {Request} request
 * @param {URL} url
 * @param {Where} where
 * @returns {Promise<Response>}
 */
async function lent(request, url, where, signs = false) {
  const held = await holding();
  if (!held) return unheldAt(request, url);
  const lend = held.lends.find((one) => one.label === where.label);
  if (!lend) return closed();
  if (request.method === 'POST' && lend.group && where.path === '/svrngs/acts') {
    return actsPosted(request, lend, signs);
  }
  if (where.read && request.mode === 'navigate') return opening(held, where);
  return proxied(
    {
      request,
      where,
      lend,
      signs,
      jar: await jarOf(where.label),
      path: `/lend/${where.label}${where.path}`,
    },
    held,
  );
}

// No pass held: a page is sent to its start page, which reads it again;
// anything else is refused.
/**
 * @param {Request} request
 * @param {URL} url
 */
function unheldAt(request, url) {
  if (request.mode !== 'navigate') return unheld();
  trace(`RUNTIME: no pass for ${OWN}: its start page reads it again`);
  const back = encodeURIComponent(url.pathname + url.search);
  return Response.redirect(
    `${SCOPE.origin}${SITE}open.html?v=__VERSION__&for=${OWN}&back=${back}`,
    302,
  );
}

// A group's lend's acts, posted as JSON by a page that may name them, are
// written here (`writing`); any other post there is refused.
/**
 * @param {Request} request
 * @param {Passed} lend
 * @param {boolean} signs
 */
function actsPosted(request, lend, signs) {
  const type = request.headers.get('content-type')?.split(';')[0]?.trim();
  if (!signs || type !== 'application/json') return closed();
  return writing(request, lend);
}

// Whether `request` carries a body.
const posts = (/** @type {Request} */ request) => !['GET', 'HEAD'].includes(request.method);

// `asked` sent through the lend's door, and its answer served as the
// browser would take it from the lend itself.
/**
 * @param {Asked} asked
 * @param {Held} held
 * @returns {Promise<Response>}
 */
async function proxied(asked, held) {
  const { request, path } = asked;
  const headers = await headersFor(asked, held);
  const body = await bodyFor(asked, headers);
  const said = ranged(request, path);
  const res = await doorAnswer(asked, headers, body);
  if ('lost' in res) return away(request, asked.lend, res.lost);
  const range = res.headers.get('content-range') ?? '';
  const late = request.signal.aborted ? ', after the browser let go of it' : '';
  said?.(`answered ${res.status} ${range}${late}`);
  const set = res.headers.get('x-svrngs-set-cookies');
  if (set) await keepCookies(asked, set);
  const turned = (await refusedAppend(asked, res)) ?? redirected(asked, res);
  if (turned) return turned;
  return served(asked, res, said, set);
}

// The headers `asked` goes to the door with: the page's, but its cookies
// and length, with a range bounded, and what a browser would have sent
// that a worker is not shown.
/**
 * @param {Asked} asked
 * @param {Held} held
 * @returns {Promise<Header[]>}
 */
async function headersFor({ request, where, jar }, held) {
  const headers = bounded(
    [...request.headers].filter(
      ([name]) => !['cookie', 'content-length'].includes(name.toLowerCase()),
    ),
    request.destination,
    where.path,
  );
  // A browser never shows a worker the `Host` it asked, and a page's
  // `fetch` cannot name one. The door tells the lender's service the
  // browser's origin as a proxy does (`X-Forwarded-Host`),
  // so the worker names the address the browser asked here.
  const host = new URL(request.url).host;
  if (host) headers.push(['host', host]);
  const cookie = cookieFor(jar, where.path, false);
  if (cookie) headers.push(['cookie', cookie]);
  headers.push(...(await passed(held)));
  // A browser names the origin a post comes from; a worker is not shown
  // it (WebKit's never, Chromium's but for a form's), and Nextcloud
  // refuses a sign-in posted without one. Named here
  // for a post from this lend's own pages: a fetch, which only they can
  // make through this worker, or a page's form whose referrer is here; a
  // post another site sends here is left without, for the app to refuse.
  const named = headers.some(([name]) => name.toLowerCase() === 'origin');
  const from = request.referrer ? new URL(request.referrer).origin : null;
  if (posts(request) && !named && (request.mode !== 'navigate' || from === self.location.origin)) {
    headers.push(['origin', self.location.origin]);
  }
  // A browser never shows a worker the `Accept-Encoding` it sends, and
  // llama-server's chat page comes only in gzip, 415 to a request without
  // (demo/runs/start/2026-10-13-story/ 9): so gzip is asked for here, but
  // for a range, and the answer unpacked below (`unpacked`).
  if (!headers.some(([name]) => /^(range|accept-encoding)$/i.test(name))) {
    headers.push(['accept-encoding', 'gzip']);
  }
  return headers;
}

// A body is read whole, as a Blob, which the browser keeps on disk when
// it is large, so it goes with its length, named in `headers`, a slice at
// a time (`sent`): a worker never sees the page's Content-Length, and a
// parser that reads by it — Django's multipart, Paperless-ngx's upload —
// reads nothing from a stream sent without one.
/**
 * @param {Asked} asked
 * @param {Header[]} headers
 */
async function bodyFor({ request, path }, headers) {
  if (!posts(request)) return null;
  const blob = await request.blob();
  headers.push(['content-length', String(blob.size)]);
  return sent(blob, `${request.method} ${path}`);
}

// The door's answer to `asked`, or what was lost when the lender's computer
// could not be reached.
/**
 * @param {Asked} asked
 * @param {Header[]} headers
 * @param {ReadableStream<Uint8Array> | null} body
 * @returns {Promise<Response | { lost: unknown }>} the wasm's `fetch` answers the page's `Response`
 */
async function doorAnswer({ request, lend, path }, headers, body) {
  for (let tries = 0; ; tries++) {
    try {
      return await (await dialFor(lend.key)).fetch(request.method, path, headers, body);
    } catch (/** @type {any} */ error) {
      dials.delete(lend.key);
      trace(`RUNTIME: ${request.method} ${path} failed (try ${tries + 1}): ${error}`);
      // A dial that heard nothing from the computer within the cap, new or
      // held, is not tried again: a second would wait the cap again.
      if (body || tries || /asleep/.test(error)) return { lost: error };
    }
  }
}

// The cookies the door's answer `set` (`x-svrngs-set-cookies`), kept in
// the lend's jar.
/**
 * @param {Asked} asked
 * @param {string} set
 */
async function keepCookies({ where, jar }, set) {
  for (const line of JSON.parse(set)) setCookie(jar, line);
  await keepJar(where.label);
}

// The acts the answer names to be written (`APPEND`): refused unless the
// request may name them, else each `appended`. None when it names none, or
// when every one is written.
/**
 * @param {Asked} asked
 * @param {Response} res
 * @returns {Promise<Response | null>}
 */
async function refusedAppend({ lend, signs }, res) {
  const naming = res.headers.get(APPEND);
  if (!naming) return null;
  if (!signs) return closed();
  return appended(lend, naming);
}

// A page whose pass the door refused goes to its start page, which asks
// the group's page for another; a redirect to a path is read under the
// lend's own address. None for any other answer.
/**
 * @param {Asked} asked
 * @param {Response} res
 */
function redirected({ request, where, path }, res) {
  if (res.headers.get(REFUSED) === 'pass' && request.mode === 'navigate') {
    trace(
      `RUNTIME: the pass for ${where.label} is refused: its start page asks the group's page for another`,
    );
    return Response.redirect(`${SCOPE.origin}${SITE}open.html?v=__VERSION__&for=${OWN}&again`, 302);
  }
  const to = res.headers.get('location');
  if (res.status >= 300 && res.status < 400 && to) {
    const there = new URL(to.startsWith('/') ? `.${to}` : to, baseOf(where.label)).href;
    trace(`RUNTIME: ${path} redirects to ${to}, read as ${there}`);
    // `Response.redirect` takes these alone; any other 3xx goes as a 302.
    const status = [301, 302, 303, 307, 308].includes(res.status) ? res.status : 302;
    return Response.redirect(there, status);
  }
  return null;
}

// The door's answer `res` to `asked`, as the browser takes it: unpacked,
// with the cookies its pages may read when it `set` some, its reading
// traced by `said`; a page with the shim in it.
/**
 * @param {Asked} asked
 * @param {Response} res
 * @param {Said | null} said
 * @param {string | null} set
 */
function served(asked, res, said, set) {
  const { answer, headed } = unpacked(res);
  // What its pages may read now, for the shim to take in before the page
  // reads the answer: Immich reads its sign-in's cookie unreloaded.
  if (set) headed.set(PAGE_COOKIES, cookieFor(asked.jar, '/', true));
  const type = res.headers.get('content-type') ?? '';
  const html = asked.request.method === 'GET' && type.includes('text/html');
  if (html) return shimmed(asked, res, answer, headed);
  const body = said && answer ? counted(answer, said) : answer;
  return new Response(body, { status: res.status, statusText: res.statusText, headers: headed });
}

// `res`'s body unpacked from gzip as a browser unpacks what it asked for
// itself, and its headers without this worker's own or, once unpacked,
// the gzip's.
/** @param {Response} res */
function unpacked(res) {
  const gzip = /^gzip$/i.test(res.headers.get('content-encoding') ?? '');
  const answer = gzip ? res.body?.pipeThrough(new DecompressionStream('gzip')) : res.body;
  const headed = new Headers(res.headers);
  headed.delete('x-svrngs-set-cookies');
  headed.delete(APPEND);
  if (gzip) {
    headed.delete('content-encoding');
    headed.delete('content-length');
  }
  return { answer, headed };
}

// A page of the lend's, `answer`, with the shim in its head, under the
// app's own policy with the signer let in.
/**
 * @param {Asked} asked
 * @param {Response} res
 * @param {ReadableStream<Uint8Array> | null | undefined} answer
 * @param {Headers} headed
 */
async function shimmed({ where, jar, path }, res, answer, headed) {
  // The app's own policy, with the signer derived from this origin's host,
  // never an address supplied by app data, let in as a frame only where it
  // restricts frames (authority.js `framing`). The signer accepts scoped acts only.
  const { policy, framed } = framing(headed.get('content-security-policy'), SIGNER);
  if (policy == null) {
    trace(`RUNTIME: ${path}'s policy is none, so none is set`);
  } else {
    const by = framed.join(', ');
    trace(`RUNTIME: ${path}'s policy frames by ${by}; the signer is let in where they restrict`);
    headed.set('content-security-policy', policy);
  }
  headed.delete('content-length');
  const text = await new Response(answer).text();
  // A policy in the page's own `<meta>` this worker can't add the signer to
  // (DESIGN §3, the known gap): said, and its saves told why.
  if (/<meta[^>]+content-security-policy/i.test(text)) walled = path;
  if (walled === path) {
    trace(`RUNTIME: ${path} sets its own policy in a <meta>, which may keep the signer out`);
  }
  const head =
    /<head[^>]*>/i.exec(text) ?? /<html[^>]*>/i.exec(text) ?? /<!doctype[^>]*>/i.exec(text);
  const at = head ? head.index + head[0].length : 0;
  const shimTag = shim(where.label, jar, res.headers.get('content-security-policy'));
  const page = `${text.slice(0, at)}${shimTag}${text.slice(at)}`;
  return new Response(page, { status: res.status, statusText: res.statusText, headers: headed });
}

// An app's write is this phone's own (DESIGN §3): shaped
// as the door shapes one (svrngs-core `app_act`) and `signed`.
/**
 * @param {Request} request
 * @param {Passed} lend a lend of a group: `lent` writes for no other
 */
async function writing(request, lend) {
  await (loaded ??= init({ module_or_path: store.runtime() }));
  const act = app_act(lend.kind, new Uint8Array(await request.arrayBuffer()));
  if (!act) {
    return Response.json({ error: 'an act is JSON, {op} or {corrects, op}' }, { status: 422 });
  }
  return signed(lend, /** @type {string} */ (lend.group), act, handed.pass);
}

// The acts a lent app's answer names (`x-svrngs-append`), this phone's own
// as a carried app's are: each shaped under the lend's kind, in the group
// it was opened from, and `signed` under the pass that lets its holder
// write that kind's part (app.js `writes`); none of them if one is not.
// Nothing when all are counted, else what the person is told.
/**
 * @param {Passed} lend
 * @param {string} named
 * @returns {Promise<Response | null>}
 */
async function appended(lend, named) {
  await (loaded ??= init({ module_or_path: store.runtime() }));
  const what = `${lend.person}’s ${lend.name}`;
  // Each act shaped, or false or none where one is not, which
  // `every(Boolean)` refuses below.
  /** @type {(string | false | undefined)[] | null} */
  let acts = null;
  try {
    acts = JSON.parse(named).map((/** @type {{ group?: string }} */ act) => shaped(lend, act));
  } catch (error) {
    trace(`RUNTIME: ${what}'s acts are not a list: ${String(error)}`);
  }
  if (!handed.writes || !acts?.every(Boolean)) {
    trace(
      `RUNTIME: refused — ${what} named an act this phone does not write as its person: ${named}`,
    );
    const error = `${what} wrote outside its own part of the group you opened it from, so it was not saved.`;
    return Response.json({ error }, { status: 403 });
  }
  // Every act is shaped, and a pass that writes is a member's, whose open
  // names its group.
  for (const act of acts) {
    const res = await signed(
      lend,
      /** @type {string} */ (handed.group),
      /** @type {string} */ (act),
      handed.writes,
    );
    if (!res.ok) return res;
  }
  return null;
}

// An act a lent app's answer named, shaped under `lend`'s kind; false
// where it names a group other than the open's.
/**
 * @param {Passed} lend
 * @param {{ group?: string }} act
 */
function shaped(lend, act) {
  if (act.group !== handed.group) return false;
  return app_act(lend.kind, new TextEncoder().encode(JSON.stringify(act)));
}

// `act` signed in `group` by the pass's holder under `grant` as its chain's
// next line, kept beside the pass before it is sent, so no seq is signed
// twice, and carried with every line not yet taken to the lend's computer
// (`/carry`), which writes none of its own. A line not taken waits, the
// same bytes, for the next write.
/** @param {Parameters<typeof writeSigned>} args */
function signed(...args) {
  const next = signing.then(() => writeSigned(...args));
  signing = next.catch(() => {});
  return next;
}

// How long, in ms, the group's signer has to approve a line.
const APPROVING = 15_000;

// `line`, unsigned, approved by the group's signer, which a page of this
// lend's frames (shim.js): the pass that permits it, or why not, thrown.
/**
 * @param {string} group
 * @param {string} line
 */
async function approved(group, line) {
  const pages = await scope.clients.matchAll({ type: 'window' });
  const client = pages.find((one) => new URL(one.url).origin === SCOPE.origin);
  if (!client) throw new Error('no app page is open to ask the trusted signer');
  const channel = new MessageChannel();
  /** @type {Promise<string>} */
  const answer = new Promise((yes, no) => {
    const why = walled
      ? `This app’s page (${walled}) may keep out your group’s signer, so this wasn’t saved.`
      : 'the trusted signer did not answer';
    const timeout = setTimeout(() => no(new Error(why)), APPROVING);
    // The signer's answer (sign.js `answered`): the pass, or why not.
    channel.port1.onmessage = (
      /** @type {MessageEvent<{ pass?: string, error?: string }>} */ { data },
    ) => {
      clearTimeout(timeout);
      if (data.pass) yes(data.pass);
      else no(new Error(data.error));
    };
  });
  const authority = { ...JSON.parse(openMessage(handed)), proof: handed.proof };
  const approve = { group, reach: handed.pass, line, authority };
  client.postMessage({ approve, signer: SIGNER }, [channel.port2]);
  return answer;
}

/**
 * @param {Passed} lend
 * @param {string} group
 * @param {string} act
 * @param {string} grant
 * @returns {Promise<Response>}
 */
async function writeSigned(lend, group, act, grant) {
  const at = handed.line ?? { seq: -1, id: null };
  const stamp = Math.floor(Date.now() / 1000);
  const draft = unsigned_line(handed.holder, group, stamp, at.seq + 1, at.id, act, grant)[0];
  let permitted;
  try {
    permitted = await approved(group, draft);
  } catch (/** @type {any} */ error) {
    return Response.json({ error: error.message }, { status: 403 });
  }
  const [unsigned, message] = unsigned_line(
    handed.holder,
    group,
    stamp,
    at.seq + 1,
    at.id,
    act,
    permitted,
  );
  const sig = await crypto.subtle.sign('Ed25519', handed.key, message);
  const line = { ...JSON.parse(unsigned), sig: store.hex(new Uint8Array(sig)) };
  handed = {
    ...handed,
    line: { seq: line.seq, id: line.id },
    unsent: [...(handed.unsent ?? []), [group, line]],
  };
  await store.lendKeep(`pass ${OWN}`, handed);
  // `unsent` is set just above, and below each time `handed` is.
  const carrying = /** @type {Unsent[]} */ (handed.unsent).length;
  trace(`RUNTIME: an act of ${lend.kind} signed as line ${line.seq}, ${carrying} to carry`);
  let res;
  try {
    res = await carried(lend);
  } catch (error) {
    dials.delete(lend.key);
    trace(`RUNTIME: lines not carried: ${String(error)}`);
    const why = unreached(error, silent(`${lend.person}’s computer`));
    const words = `${why} It is kept on this phone and goes with your next entry`;
    return Response.json({ error: words }, { status: 502 });
  }
  return judged(lend, res);
}

// `text` posted as JSON to `lend`'s computer, beside `headers`.
/**
 * @param {Passed} lend
 * @param {string} path
 * @param {string} text
 * @param {Header[]} [headers]
 * @returns {Promise<Response>} the wasm's `fetch` answers the page's `Response`
 */
async function posted(lend, path, text, headers = []) {
  const body = new TextEncoder().encode(text);
  const dial = await dialFor(lend.key);
  const heads = [
    ...headers,
    ['content-type', 'application/json'],
    ['content-length', `${body.length}`],
  ];
  return dial.fetch('POST', path, heads, new Blob([body]).stream());
}

// Each line not yet carried (`unsent`) taken to the lend's computer in
// turn, and let go of once taken, until one is not: the answer to the
// last one sent.
/**
 * @param {Passed} lend
 * @returns {Promise<Response>}
 */
async function carried(lend) {
  /** @type {Response | undefined} */
  let res;
  for (const carrying of /** @type {Unsent[]} */ (handed.unsent)) {
    res = await posted(lend, '/carry', JSON.stringify(carrying));
    trace(
      `RUNTIME: line ${carrying[1].seq} carried to ${lend.person}'s computer, answered ${res.status}`,
    );
    if (!res.ok) break;
    handed = {
      ...handed,
      unsent: /** @type {Unsent[]} */ (handed.unsent).filter(
        (unsent) => unsent[1].id !== carrying[1].id,
      ),
    };
    await store.lendKeep(`pass ${OWN}`, handed);
  }
  // `writeSigned` called this with the line it signed in `unsent`, so one
  // was sent.
  return /** @type {Response} */ (res);
}

// The lend's computer's answer `res` to the last line carried, as the page
// is told it.
/**
 * @param {Passed} lend
 * @param {Response} res
 */
async function judged(lend, res) {
  // A computer on an older svrngs takes no signed line: kept, said.
  if (!res.ok && !links(lend.version)) {
    const words = `${lend.person}’s computer runs an older svrngs: what you write here reaches your group when a computer on the new svrngs is on.`;
    return Response.json({ error: words }, { status: 503 });
  }
  // Taken but not counted: the fold refused it (svrngs-core `Grant`), said
  // to the page, never as saved.
  /** @type {{ counted?: boolean } | null} */
  const said = res.ok ? await res.json() : null;
  if (said?.counted === false) {
    return Response.json({ error: 'your group does not count it' }, { status: 409 });
  }
  return said ? Response.json(said) : res;
}

// The lend's computer did not answer: a person's tab is shown its person's
// window, dark, naming the lend (away.html, which a computer's door shows
// too, scripts/build-web.sh copying it here); a page's own request, and a
// tab when that page cannot be fetched, svrngs's words.
/**
 * @param {Request} request
 * @param {Passed} lend
 * @param {unknown} error
 */
async function away(request, lend, error) {
  const words = unreached(error, QUIET);
  const dark = words === QUIET && request.mode === 'navigate';
  const page = dark ? await awayPage() : undefined;
  const as = page ? 'as its window, dark' : 'in svrngs’s words';
  trace(`RUNTIME: ${lend.label}'s computer was not reached, said ${as}: ${String(error)}`);
  if (!page) return say(502, words, 'away');
  const who = lend.person.replace(/</g, '&lt;');
  const what = `${who}’s ${lend.name.replace(/</g, '&lt;')}`;
  const filled = page.replaceAll('{who}', who).replaceAll('{what}', what);
  return new Response(filled, {
    status: 502,
    headers: { 'content-type': 'text/html; charset=utf-8', [REFUSED]: 'away' },
  });
}

// away.html's text, or none, traced, where it cannot be fetched.
async function awayPage() {
  try {
    const response = await saved(new Request(store.savedAt('away.html')));
    if (response.ok) return await response.text();
    trace(`RUNTIME: away.html not fetched: ${response.status}`);
  } catch (error) {
    trace(`RUNTIME: away.html not fetched: ${String(error)}`);
  }
  return undefined;
}

// What this lend's pages are told of the lends they read: none of the
// pass. A handoff that names no one unanswered is from a page from before
// the pages said who did not answer: asked for again, never read as none.
async function told() {
  const held = await holding();
  if (!held?.unanswered) return unheld();
  const lends = held.lends.filter((lend) => lend.label !== OWN);
  trace(
    `RUNTIME: told this lend's page the ${lends.length} lend(s) it reads, ${held.unanswered.length} unanswered`,
  );
  return Response.json({ lends: lends.map(toldOf), unanswered: held.unanswered });
}

// A lend this lend's pages read, as they are told it: with the words it
// asks with, and where to ask and stop, when it asks.
/** @param {Passed} lend */
function toldOf(lend) {
  const address = `${SCOPE.origin}${READS}/${lend.label}/`;
  const entry = { kind: lend.kind, name: lend.name, lender: lend.person, address };
  if (!lend.asks) return entry;
  return {
    ...entry,
    asks: lend.asks,
    ask: `${SCOPE.origin}${ASK}?lend=${lend.label}&app=${OWN}`,
    stop: `${SCOPE.origin}${READS}/${lend.label}/svrngs-stop`,
  };
}

// The group's page at `to`, then `then` and the fragment this page was
// opened with, which the browser keeps from a worker; so a page of this
// worker's own sends it on, `<` escaped.
/**
 * @param {string} to
 * @param {string} then
 */
function onward(to, then) {
  const there = JSON.stringify(to).replace(/</g, '\\u003c');
  const tail = JSON.stringify(then).replace(/</g, '\\u003c');
  const script = `location.replace(${there} + encodeURIComponent(${tail} + location.hash))`;
  return new Response(`<!doctype html><script>${script}</script>`, {
    headers: { 'content-type': 'text/html; charset=utf-8' },
  });
}

// A page navigated to `where`, a lend it reads: the group's page opens that
// lend at the path.
/**
 * @param {Held} held
 * @param {Where} where
 */
const opening = (held, where) => {
  trace(
    `RUNTIME: a page opens ${where.label}, which it reads, at ${where.path}: through the group's page`,
  );
  return onward(`${held.page}#open=${where.label}&at=`, where.path);
};

// A page sends its person to say yes to this lend using `lend`, one it
// reads: the group's page asks, then opens this lend again at `back`.
/** @param {URL} url */
async function asking(url) {
  const held = await holding();
  const lend = url.searchParams.get('lend');
  if (!held) return unheld();
  if (!held.lends.some((one) => one.label === lend && one.label !== OWN)) {
    trace(`RUNTIME: a page asks for ${lend}, which it does not read: refused`);
    return closed();
  }
  trace(`RUNTIME: a page asks its person to let ${OWN} use ${lend}: through the group's page`);
  return onward(`${held.page}#ask=${lend}&app=${OWN}&at=`, url.searchParams.get('back') ?? '/');
}

// A page stops this lend using `label`: sent to its door with the pass.
/** @param {string} label */
async function stopping(label) {
  const held = await holding();
  if (!held) return unheld();
  const lend = held.lends.find((one) => one.label === label);
  if (!lend) {
    trace(`RUNTIME: a page stops ${OWN} using ${label}, which it does not read: refused`);
    return closed();
  }
  const headers = await passed(held);
  trace(`RUNTIME: a page stops ${OWN} using ${label}`);
  try {
    return await posted(lend, `/choices/${label}`, '{"said":"stop"}', headers);
  } catch (error) {
    dials.delete(lend.key);
    trace(`RUNTIME: the stop for ${label} failed: ${String(error)}`);
    return say(502, unreached(error, QUIET), 'away');
  }
}

// ── the app's own worker ────────────────────────────────────────────────
//
// An app that registers a worker of its own would put it over this one,
// which serves its every request; the shim hands its script here instead,
// kept in this origin's store, and its listeners are asked first.

/**
 * @param {string} code
 * @returns {On}
 */
function appWorker(code) {
  /** @type {On} */
  const on = { fetch: [], install: [], activate: [], message: [] };
  const own = {
    // A type the app names that is none of `on`'s is let go.
    /**
     * @param {string} type
     * @param {Listener} listener
     */
    addEventListener: (type, listener) => {
      /** @type {Partial<On>} */ (on)[/** @type {keyof On} */ (type)]?.push(listener);
    },
    skipWaiting: async () => {},
    /**
     * @param {RequestInfo | URL} input
     * @param {RequestInit} [options]
     */
    fetch: (input, options) => {
      const request = new Request(
        input instanceof Request ? input : new URL(input, SCOPE),
        options,
      );
      const where = whereOf(new URL(request.url));
      return where ? lent(request, new URL(request.url), where, true) : fetch(request);
    },
  };
  // The app's `self`: what `own` names, else the worker's own, its
  // functions bound to it.
  const appScope = new Proxy(/** @type {Record<string | symbol, any>} */ (self), {
    get: (target, property) => {
      if (property in own) return own[/** @type {keyof typeof own} */ (property)];
      if (typeof target[property] === 'function') return target[property].bind(target);
      return target[property];
    },
  });
  // The app's own worker, run as its code: the one way a worker runs a script
  // it has as text, in the scope above.
  // oxlint-disable-next-line typescript/no-implied-eval
  new Function('self', 'addEventListener', 'skipWaiting', 'fetch', code)(
    appScope,
    own.addEventListener,
    own.skipWaiting,
    own.fetch,
  );
  for (const listener of [...on.install, ...on.activate]) listener({ waitUntil: () => {} });
  trace(`RUNTIME: the app's worker runs, ${on.fetch.length} fetch listener(s)`);
  return on;
}

async function appLoaded() {
  if (app === undefined) {
    const code = /** @type {string | undefined | null} */ (
      await store.lendKept(`worker ${OWN}`).catch(() => null)
    );
    app = code ? appWorker(code) : null;
  }
  return app;
}

// The app's listeners first: an answer one of them gives, else the lend's.
/**
 * @param {FetchEvent} fetchEvent
 * @param {URL} url
 * @param {Where} where
 */
async function answered(fetchEvent, url, where) {
  const allowed = await maySign(fetchEvent, scope.clients, SCOPE.origin);
  // App code must not turn an untrusted navigation into an authorized fetch.
  const on = allowed && where.label === OWN ? await appLoaded() : null;
  /** @type {Response | PromiseLike<Response> | null} */
  let given = null;
  if (on) {
    const event = {
      request: fetchEvent.request,
      clientId: fetchEvent.clientId,
      /** @param {Response | PromiseLike<Response>} promise */
      respondWith: (promise) => (given ??= promise),
      waitUntil: () => {},
    };
    for (const listener of on.fetch) listener.call(self, event);
  }
  // Set by a listener's `respondWith`, which the checker does not follow
  // into the listener, so it reads `given` as still `null` here.
  return (
    /** @type {Response | PromiseLike<Response> | null} */ (given) ??
    lent(fetchEvent.request, url, where, allowed)
  );
}

scope.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  const here = url.origin === self.location.origin;
  if (here && shellAt(url)) {
    event.respondWith(saved(event.request));
    return;
  }
  const own = here ? ownAnswer(event.request, url) : null;
  if (own) {
    event.respondWith(own);
    return;
  }
  const where = whereOf(url);
  if (where) event.respondWith(answered(event, url, where));
});

// Whether `url`, at this origin, is one of the page's files, any version
// of them (`SAVED`).
/** @param {URL} url */
const shellAt = (url) =>
  url.searchParams.has('v') &&
  url.pathname.startsWith(SITE) &&
  store.SAVED.includes(url.pathname.slice(SITE.length));

// One of the page's files: this build's from what is saved, its version
// matched and the rest of its address not, as the shim's carries its lend
// and an open its way back; any other version from the network, so a newer
// page starts here too. Neither there, the person is told what to do.
/** @param {Request} request */
async function saved(request) {
  const url = new URL(request.url);
  const file = url.pathname.slice(SITE.length);
  const here = url.searchParams.get('v') === '__VERSION__';
  const hit = here
    ? await caches.open(store.CACHE).then((cache) => cache.match(store.savedAt(file)))
    : null;
  if (hit) return hit;
  trace(`RUNTIME: ${file} ${url.searchParams.get('v')} is not saved here, so the site is asked`);
  return fetch(request).catch((error) => {
    trace(`RUNTIME: ${file} could not be fetched either: ${String(error)}`);
    return say(503, 'This phone has not saved this app. Open it once while online.', 'away');
  });
}

// The answer to `request` at one of this worker's own addresses on this
// origin: its trace, the lends its pages read, an ask, or a stop. None at
// any other.
/**
 * @param {Request} request
 * @param {URL} url
 * @returns {Promise<Response> | null}
 */
function ownAnswer(request, url) {
  if (url.pathname === TRACE) return traceShown();
  if (url.pathname === READS) return told();
  if (url.pathname === ASK) return asking(url);
  const stop = request.method === 'POST' ? STOP.exec(url.pathname) : null;
  // `STOP` names its label whenever it matches.
  if (stop) return stopping(/** @type {string} */ (stop[1]));
  return null;
}

async function traceShown() {
  const lines = await kept();
  return new Response(`svrngs lend worker __VERSION__ for ${OWN}\n${lines.join('\n')}`, {
    headers: PLAIN,
  });
}

// ── what a lend's page hands over ───────────────────────────────────────

// A message from a page of this origin: a window's `Client`, of which a
// worker's `source` is one kind.
scope.addEventListener('message', async (event) => {
  const reply = event.ports[0];
  if (!fromHere(/** @type {Client | null} */ (event.source))) {
    reply?.postMessage({ error: 'only this origin hands the worker authority' });
    return;
  }
  // A page's hand-over, each part read as its kind before it is taken: an
  // open (`handsPass`), a cookie or a worker by `typeof`, a socket as
  // shim.js asks one.
  /**
   * @type {{
   *   handed?: any,
   *   cookie?: unknown,
   *   worker?: unknown,
   *   socket?: { url: string, protocols?: string[] },
   * }}
   */
  const data = event.data ?? {};
  const { handed: given, cookie, worker, socket: asked } = data;
  if (handsPass(given)) {
    await holdPass(given, reply);
  } else if (typeof cookie === 'string') {
    event.waitUntil(cookieKept(cookie, reply));
  } else if (typeof worker === 'string') {
    event.waitUntil(workerRun(worker, reply));
  } else if (asked && reply) {
    event.waitUntil(socket(asked, reply));
  } else {
    trace(`RUNTIME: a message that hands over nothing, refused: ${Object.keys(data).join(',')}`);
  }
});

/** @param {Client | null} source */
const fromHere = (source) => !!source?.url && new URL(source.url).origin === SCOPE.origin;

// Whether `given` is an open of this lend's, with its pass.
/** @param {any} given */
const handsPass = (given) => given?.label === OWN && typeof given.pass === 'string';

// The open the page handed over, held in place of any before it.
/**
 * @param {any} given
 * @param {MessagePort | undefined} reply
 */
async function holdPass(given, reply) {
  // The holder's bytes, where its key was lost on the way (below):
  // imported here as a key no script can copy, held in memory alone.
  const bytes = given.key instanceof Uint8Array;
  if (bytes) {
    given.key = await crypto.subtle.importKey('pkcs8', given.key, 'Ed25519', false, ['sign']);
  }
  handed = given;
  dials.clear();
  endpoint = null;
  const lends = `${given.lends.length} lend(s)`;
  const imported = bytes ? ', and its holder’s bytes, imported here and kept nowhere' : '';
  trace(`RUNTIME: the page handed over the pass for ${OWN}, ${lends}${imported}`);
  reply?.postMessage('held');
}

// A cookie the shim set from this lend's page, kept in its jar.
/**
 * @param {string} cookie
 * @param {MessagePort | undefined} reply
 */
async function cookieKept(cookie, reply) {
  const jar = await jarOf(OWN);
  setCookie(jar, cookie);
  await keepJar(OWN);
  reply?.postMessage('kept');
}

// The app's own worker, its script read through the lend at `worker`,
// kept and run (`appWorker`).
/**
 * @param {string} worker
 * @param {MessagePort | undefined} reply
 */
async function workerRun(worker, reply) {
  const url = new URL(worker, SCOPE);
  const where = whereOf(url);
  try {
    const res = where && (await lent(new Request(url), url, where));
    if (!res?.ok) {
      reply?.postMessage({ error: `its worker answered ${res?.status}` });
      return;
    }
    const code = await res.text();
    await store.lendKeep(`worker ${OWN}`, code);
    app = appWorker(code);
    reply?.postMessage('running');
  } catch (error) {
    reply?.postMessage({ error: String(error) });
  }
}

// Safari on an iPhone loses a message carrying a CryptoKey, the holder's,
// and fires this in its place (2026-10-30, demo/runs/ios/2026-10-30-observe/
// keyprobe/phone.log): its pages are told, so the start page hands the
// key's bytes at once (open.js).
scope.addEventListener('messageerror', async () => {
  trace('RUNTIME: a message was lost on its way here: its pages are told');
  const clients = await scope.clients.matchAll({ includeUncontrolled: true });
  for (const client of clients) client.postMessage(store.LOST);
});

// ── a WebSocket, over a socket the door switches ────────────────────────

// A frame's first bytes: its opcode, final, and its length, masked, in
// the form its size takes (RFC 6455 §5.2).
/**
 * @param {number} op
 * @param {number} size
 */
function headOf(op, size) {
  if (size < 126) return [0x80 | op, 0x80 | size];
  if (size < 65536) return [0x80 | op, 0xfe, size >> 8, size & 255];
  return [
    0x80 | op,
    0xff,
    0,
    0,
    0,
    0,
    size >>> 24,
    (size >>> 16) & 255,
    (size >>> 8) & 255,
    size & 255,
  ];
}

// One frame from this end: masked, as a client's must be (RFC 6455 §5.3).
/**
 * @param {number} op
 * @param {string | ArrayBuffer | Uint8Array} data
 */
function frame(op, data) {
  const bytes = typeof data === 'string' ? new TextEncoder().encode(data) : new Uint8Array(data);
  const size = bytes.length;
  const head = headOf(op, size);
  const mask = crypto.getRandomValues(new Uint8Array(4));
  const out = new Uint8Array(head.length + 4 + size);
  out.set(head);
  out.set(mask, head.length);
  for (let i = 0; i < size; i++) {
    // `i` is under `size`, the bytes' length, and `i & 3` under the mask's 4.
    const byte = /** @type {number} */ (bytes[i]);
    out[head.length + 4 + i] = byte ^ /** @type {number} */ (mask[i & 3]);
  }
  return out;
}

// Carry the WebSocket the shim opened at `url` to `port`: open, each
// message both ways, ping answered, close.
/**
 * @param {{ url: string, protocols?: string[] }} asked
 * @param {MessagePort} port
 */
async function socket(asked, port) {
  const remote = await switched(asked, port);
  if (!remote) return;
  const writer = remote.writable.getWriter();
  const chosen =
    remote.headers.find(
      (/** @type {Header} */ [name]) => name.toLowerCase() === 'sec-websocket-protocol',
    )?.[1] ?? '';
  port.postMessage({ open: chosen });
  port.onmessage = ({ data }) => {
    // Opcodes (RFC 6455 §5.2): 1 text, 2 binary, 8 close.
    if ('send' in data) {
      void writer.write(frame(typeof data.send === 'string' ? 1 : 2, data.send));
    }
    if ('close' in data) {
      void writer.write(frame(8, new Uint8Array([data.close >> 8, data.close & 255])));
    }
  };
  return relayed(remote.readable.getReader(), writer, port);
}

// The wasm's `socket` answers (svrngs_web.d.ts `Dialled`): the raw bytes
// both ways once `status` is 101.
/**
 * @typedef {{
 *   status: number,
 *   headers: Header[],
 *   readable: ReadableStream<Uint8Array>,
 *   writable: WritableStream<Uint8Array>,
 * }} Switched
 */

// The socket the lend's door switches to for the WebSocket asked at `url`,
// or none, `port` told why.
/**
 * @param {{ url: string, protocols?: string[] }} asked
 * @param {MessagePort} port
 * @returns {Promise<Switched | null>}
 */
async function switched({ url, protocols }, port) {
  const http = new URL(url.replace(/^ws/, 'http'));
  const where = whereOf(http);
  const held = await holding();
  const lend = where && held?.lends.find((one) => one.label === where.label);
  if (!lend) {
    port.postMessage({ closed: 1006, reason: 'not open to this page' });
    return null;
  }
  const key = btoa(String.fromCharCode(...crypto.getRandomValues(new Uint8Array(16))));
  const headers = [
    ['connection', 'upgrade'],
    ['upgrade', 'websocket'],
    ['sec-websocket-version', '13'],
    ['sec-websocket-key', key],
    // A lend was found, so the open is held.
    ...(await passed(/** @type {Held} */ (held))),
  ];
  if (protocols && protocols.length > 0) {
    headers.push(['sec-websocket-protocol', protocols.join(', ')]);
  }
  const cookie = cookieFor(await jarOf(where.label), where.path, false);
  if (cookie) headers.push(['cookie', cookie]);
  /** @type {Switched} */
  let remote;
  try {
    const dial = await dialFor(lend.key);
    remote = await dial.socket(`/lend/${where.label}${where.path}`, headers);
  } catch (error) {
    // The dial let go, as `lent` does, so the next socket dials the
    // computer afresh once it answers again (demo/runs/doc/2026-10-20/found/).
    dials.delete(lend.key);
    trace(`RUNTIME: a socket at ${where.path} failed: ${String(error)}`);
    port.postMessage({ closed: 1006, reason: String(error) });
    return null;
  }
  if (remote.status !== 101) {
    port.postMessage({ closed: 1006, reason: `answered ${remote.status}` });
    return null;
  }
  return remote;
}

// A message's frames so far: its first's opcode, and each one's bytes.
/** @typedef {{ op: number, data: Uint8Array }} Part */
// One frame read from the door's socket: its opcode, whether it is a
// message's last, its bytes, and the bytes after it.
/** @typedef {{ op: number, fin: number, data: Uint8Array, rest: Uint8Array<ArrayBuffer> }} Arrived */

// Each frame the door's socket sends, read off `reader` until the socket
// ends or closes: a ping answered on `writer`, and each whole message and
// the close told to `port`.
/**
 * @param {ReadableStreamDefaultReader<Uint8Array>} reader
 * @param {WritableStreamDefaultWriter<Uint8Array>} writer
 * @param {MessagePort} port
 */
async function relayed(reader, writer, port) {
  let buf = new Uint8Array(0);
  /** @type {Part[]} */
  let parts = [];
  for (;;) {
    const { value, done } = await reader.read();
    if (done) {
      port.postMessage({ closed: 1006, reason: 'the stream ended' });
      return;
    }
    buf = Uint8Array.from([...buf, ...value]);
    for (let arrived = frameAt(buf); arrived; arrived = frameAt(buf)) {
      buf = arrived.rest;
      if (arrived.op === 9) void writer.write(frame(10, arrived.data));
      if (arrived.op === 8) return closedBy(arrived.data, writer, port);
      parts = gathered(parts, arrived, port);
    }
  }
}

// The first frame in `buf`, or none until all of it has come.
/**
 * @param {Uint8Array<ArrayBuffer>} buf
 * @returns {Arrived | null}
 */
function frameAt(buf) {
  if (buf.length < 2) return null;
  let length = /** @type {number} */ (buf[1]) & 127;
  // The head is 2 bytes, or 4 or 10 with a longer length, and is waited
  // for whole before that length is read: a byte not come yet reads
  // `undefined`, which `*` makes NaN, the check below lets NaN by, and
  // `slice(NaN)` keeps the whole buffer, so a 64 KiB frame whose head came
  // in two reads was read again forever (web/tests/lend-frames.test.mjs).
  // Past this check each byte the head names has come.
  let at = 2;
  if (length === 126) at = 4;
  if (length === 127) at = 10;
  if (buf.length < at) return null;
  // `else`: a 2-byte length may itself be 127, and is a length, not the
  // 8-byte form's mark. Past the check above, each byte read here has come.
  const byte = (/** @type {number} */ index) => /** @type {number} */ (buf[index]);
  if (length === 126) {
    length = (byte(2) << 8) | byte(3);
  } else if (length === 127) {
    length = byte(6) * 2 ** 24 + ((byte(7) << 16) | (byte(8) << 8) | byte(9));
  }
  if (buf.length < at + length) return null;
  return {
    op: byte(0) & 15,
    fin: byte(0) & 128,
    data: buf.slice(at, at + length),
    rest: buf.slice(at + length),
  };
}

// The close the door's socket sent, `data`, told to `port`, and the
// socket closed.
/**
 * @param {Uint8Array} data
 * @param {WritableStreamDefaultWriter<Uint8Array>} writer
 * @param {MessagePort} port
 */
function closedBy(data, writer, port) {
  // A close frame's first two bytes, when it has them, are its code
  // (RFC 6455 §5.5.1); 1005 is "none was given" (§7.4.1).
  let code = 1005;
  if (data.length >= 2) {
    code = (/** @type {number} */ (data[0]) << 8) | /** @type {number} */ (data[1]);
  }
  port.postMessage({ closed: code, reason: new TextDecoder().decode(data.slice(2)) });
  return writer.close().catch(() => {});
}

// A data frame (continuation, text or binary, RFC 6455 §5.2) added to the
// message's `parts`, and the message told to `port` once whole: the parts
// left to come after `arrived`.
/**
 * @param {Part[]} parts
 * @param {Arrived} arrived
 * @param {MessagePort} port
 * @returns {Part[]}
 */
function gathered(parts, { op, fin, data }, port) {
  if (![0, 1, 2].includes(op)) return parts;
  const message = [...parts, { op, data }];
  if (!fin) return message;
  const whole = Uint8Array.from(message.flatMap((part) => [...part.data]));
  // A part was added just above; the first's opcode is the message's, 1 text.
  const text = /** @type {Part} */ (message[0]).op === 1;
  port.postMessage({ message: text ? new TextDecoder().decode(whole) : whole.buffer });
  return [];
}
