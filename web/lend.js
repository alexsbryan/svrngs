// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's worker: the runtime, at the lend's own origin (u5-2; DESIGN §3,
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
// (mnp-2; DESIGN §3, "Lent apps", call 6): each one's kind, name, lender
// and an address under `READS` to ask it at, and whose computer did not
// answer — what the group's page handed, never the pass or its key. A
// request at such an address is that lend's; a page navigated there opens
// that lend at the path asked, through the group's page, which signs its
// pass, and its start page (open.js), which goes there.
//
// A lend that asks (sup-3; DESIGN §3, "What an app gets") is told with
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
// the last runs' (u5-12: a worker WebKit ended took its trace with it).
import init, {
  actor,
  app_act,
  holder_endpoint,
  link,
  links,
  unsigned_line,
} from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';
import { maySign, signerFor, openMessage, framing } from './authority.js?v=__VERSION__';
/** @import { Handoff, Passed } from './authority.js?v=__VERSION__' */
/** @import { Dialled, Endpoint } from './wasm/svrngs_web.js?v=__VERSION__' */

// `self` as a service worker's: the worker's lib names it a
// `WorkerGlobalScope`, which every kind of worker shares
// (jsconfig.workers.json). Its own first, so `addEventListener` gives each
// listener its event's type.
/** @typedef {ServiceWorkerGlobalScope & typeof self} Scope */
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
/** @typedef {{ name: string, value: string, path: string, expires: number | null, page: boolean }} Cookie */
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

const SCOPE = new URL(/** @type {Scope} */ (self).registration.scope);
// Where the page's files are on this origin, the shell this worker leaves
// to the network: any version of them, so a newer page starts here too.
// `self.location` is a `WorkerLocation`, which `URL` reads as its `href`.
const SITE = new URL('./', /** @type {string} */ (/** @type {unknown} */ (self.location))).pathname;
const SHELL =
  /^(open\.html|open\.js|lend\.js|shim\.js|store\.js|authority\.js|wasm\/svrngs_web(_bg\.wasm|\.js))$/;
// This worker's lend: the first name of its address.
const OWN = store.labelOf(SCOPE.hostname);
const SIGNER = signerFor(SCOPE, SITE);
const PLAIN = { 'content-type': 'text/plain; charset=utf-8' };
const TRACE = `${SCOPE.pathname}svrngs-trace`;
// Where its pages ask the lends they read, as a computer's origin answers
// them too (crates/svrngs-node/src/surface.rs, `READS_PATH`).
const READS = `${SCOPE.pathname}svrngs-reads`;
const ASK = `${SCOPE.pathname}svrngs-ask`;
const STOP = new RegExp(`^${READS}/([^/]+)/svrngs-stop$`);

// Enough for a film's seeks, each range's lines with them (u5-11), over
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

/** @type {Scope} */ (self).addEventListener('install', () =>
  /** @type {Scope} */ (self).skipWaiting(),
);
/** @type {Scope} */ (self).addEventListener('activate', (event) =>
  event.waitUntil(/** @type {Scope} */ (self).clients.claim()),
);

// What this run of the worker holds, all of it lost when the browser stops
// it: what the page handed it, the wasm, its endpoint, a dial per lender,
// each lend's cookies, the app's worker.
//
// `handed` is none until a page hands it over or `holding` reads it back,
// and is read as held: every reader but `holding` runs after `holding`
// found it, or after a page handed it, as `lent` and `socket` do.
/** @type {Held} */
let handed = /** @type {Held} */ (/** @type {unknown} */ (null));
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
  const [, read, path] =
    (url.origin === SCOPE.origin && url.pathname.match(new RegExp(`^${READS}/([^/]+)(/.*)?$`))) ||
    [];
  if (read) return { label: read, path: `${path ?? '/'}${url.search}`, read: true };
  for (const label of [OWN, ...(handed?.lends ?? []).map((lend) => lend.label)]) {
    const base = new URL(baseOf(label));
    if (url.origin === base.origin && url.pathname.startsWith(base.pathname)) {
      return { label, path: `/${url.pathname.slice(base.pathname.length)}${url.search}` };
    }
  }
  return null;
}

/** @returns {Promise<Held | null>} */
async function holding() {
  if (handed) return handed;
  try {
    handed = (await store.lendKept(`pass ${OWN}`)) ?? null;
    trace(
      `RUNTIME: the pass for ${OWN} ${handed ? 'read from this origin’s store' : 'is not kept here'}`,
    );
  } catch (error) {
    trace(`RUNTIME: this origin's store cannot be read here: ${error}`);
  }
  return handed;
}

/**
 * @param {string} lender
 * @returns {Promise<Dialled>}
 */
function dialFor(lender) {
  if (!dials.has(lender)) {
    const opening = (async () => {
      endpoint ??= opened();
      trace(`RUNTIME: dialling ${lender} for the pass's holder`);
      // Through the relay its group names for the lender (relay-in-record).
      return (await endpoint).dial_lend(
        lender,
        handed.lends.find((lend) => lend.key === lender)?.relay ?? null,
      );
    })();
    dials.set(lender, opening);
    opening.catch(() => dials.delete(lender));
  }
  // Set just above when it was not.
  return /** @type {Promise<Dialled>} */ (dials.get(lender));
}

// This run's endpoint (capability-signed (H); DESIGN §3): a key of this
// run's alone, which the holder, a key no page can copy, names in a link
// the door checks beside the pass (`linked`); or, when a lend of the open
// is at a door taking no link, the holder's own bytes, as before (F2).
async function opened() {
  await (loaded ??= init({ module_or_path: store.runtime() }));
  const own = !handed.seed;
  const bytes = own
    ? crypto.getRandomValues(new Uint8Array(32))
    : // Not `own`, so the open names a seed.
      Uint8Array.from(
        /** @type {RegExpMatchArray} */ (/** @type {string} */ (handed.seed).match(/../g)),
        (pair) => parseInt(pair, 16),
      );
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
  trace(
    `RUNTIME: opening the endpoint ${own ? 'of this run, named by the holder’s link' : 'as the pass’s holder, for an older door'}`,
  );
  return holder_endpoint(bytes, handed.relay);
}

// The pass a request carries, and the link naming this run's endpoint
// when the holder signed one (`opened`).
/**
 * @param {Held} handoff
 * @returns {Promise<Header[]>}
 */
const passed = async (handoff) => (
  await (endpoint ??= opened()),
  [
    ['x-svrngs-pass', handoff.pass],
    ...(linked ? [/** @type {Header} */ (['x-svrngs-link', linked])] : []),
  ]
);

// ── cookies, a jar per lend ─────────────────────────────────────────────

/**
 * @param {string} label
 * @returns {Promise<Cookie[]>}
 */
const jarOf = async (label) => (
  jars.has(label) ||
    jars.set(label, (await store.lendKept(`cookies ${label}`).catch(() => null)) ?? []),
  // Set just above when it was not.
  /** @type {Cookie[]} */ (jars.get(label))
);

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
  const eq = pair.indexOf('=');
  if (eq < 1) return;
  /** @type {Cookie} */
  const cookie = {
    name: pair.slice(0, eq).trim(),
    value: pair.slice(eq + 1).trim(),
    path: '/',
    expires: null,
    page: true,
  };
  for (const attr of attrs) {
    const [name, ...rest] = /** @type {[string, ...string[]]} */ (attr.split('='));
    const [key, value] = [name.trim().toLowerCase(), rest.join('=').trim()];
    if (key === 'path' && value.startsWith('/')) cookie.path = value;
    if (key === 'max-age') cookie.expires = Date.now() + Number(value) * 1000;
    if (key === 'expires' && cookie.expires === null) cookie.expires = Date.parse(value);
    if (key === 'httponly') cookie.page = false;
  }
  const others = jar.filter((one) => one.name !== cookie.name || one.path !== cookie.path);
  jar.splice(0, jar.length, ...others, ...(live(cookie) ? [cookie] : []));
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
const keepJar = async (label) =>
  void (await store
    .lendKeep(`cookies ${label}`, await jarOf(label))
    .catch((error) => trace(`RUNTIME: cookies not kept: ${error}`)));

// ── a request, through the door ─────────────────────────────────────────

// A range that runs on is asked of the door as a part of at most PART (u4-9;
// moved from sw.js): an open-ended `bytes=N-`, and a `<video>`'s closed
// range longer than a part, WebKit's `bytes=N-<end>`. A page's own fetch
// of a closed range gets what it asked.
//
// An open-ended range's part is FIRST where no part of the file ended just
// there — a seek, or a first look — and twice the last each time the
// reading runs on from where the last ended, up to PART (u5-11). Chromium
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

/**
 * @param {Header[]} headers
 * @param {RequestDestination} destination
 * @param {string} path
 * @returns {Header[]}
 */
function bounded(headers, destination, path) {
  return headers.map(
    /** @returns {Header} */ ([name, value]) => {
      const range = name.toLowerCase() === 'range' && /^bytes=(\d+)-(\d*)$/.exec(value.trim());
      if (!range) return [name, value];
      const from = Number(range[1]);
      const media = destination === 'video' || destination === 'audio';
      const size =
        range[2] !== '' ? PART : Math.min(PART, 2 * (ends.get(`${path} ${from}`) ?? FIRST / 2));
      if (range[2] !== '' && !(media && Number(range[2]) - from + 1 > size)) return [name, value];
      ends.delete(`${path} ${from}`);
      ends.set(`${path} ${from + size}`, size);
      if (ends.size > 64) ends.delete(/** @type {string} */ (ends.keys().next().value));
      trace(
        `RUNTIME: ${value} for ${destination || 'a fetch'} runs on: asked as a part of ${size} bytes`,
      );
      return [name, `bytes=${from}-${from + size - 1}`];
    },
  );
}

// A range, or a media element's ask, traced from asked to its end
// (u5-11): answered, read whole, cancelled by the browser after so many
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
  if (!request.headers.has('range') && !['video', 'audio'].includes(request.destination))
    return null;
  const [ask, t0] = [++asks, Date.now()];
  /** @type {Said} */
  const said = (what) => trace(`RANGE ${ask}: ${what}, ${Date.now() - t0} ms`);
  said(
    `asked ${path} ${request.headers.get('range') ?? 'whole'} for ${request.destination || 'a fetch'}`,
  );
  request.signal?.addEventListener('abort', () => said('the browser let go of it'));
  return said;
}

// `body`, its reading traced by `said`.
/**
 * @param {ReadableStream<Uint8Array>} body
 * @param {Said} said
 */
function counted(body, said) {
  const reader = body.getReader();
  let [bytes, cancelled] = [0, false];
  return new ReadableStream({
    async pull(controller) {
      const { value, done } = await reader
        .read()
        .catch(
          (error) => (
            said(`the body failed after ${bytes} bytes: ${error}`),
            Promise.reject(error)
          ),
        );
      if (done && !cancelled) {
        said(`read whole, ${bytes} bytes`);
        controller.close();
      }
      if (done) return;
      bytes += value.byteLength;
      controller.enqueue(value);
    },
    cancel: (why) => (
      (cancelled = true),
      said(`cancelled by the browser after ${bytes} bytes: ${why}`),
      reader.cancel(why)
    ),
  });
}

// `blob`'s bytes, a SLICE at a time as the dial takes them, traced when it
// has taken them all (u5-12). WebKit's `blob.stream()` gives a 64 MiB body
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
      if (at >= blob.size)
        return (
          trace(`RUNTIME: ${what} handed its body to the dial whole, ${Date.now() - t0} ms`),
          controller.close()
        );
      controller.enqueue(new Uint8Array(await blob.slice(at, (at += SLICE)).arrayBuffer()));
    },
  });
}

// The header an answer that set cookies carries to the shim (shim.js).
const PAGE_COOKIES = 'x-svrngs-page-cookies';

// svrngs's own answer in the lend's place, marked with its kind as the
// door marks its own (door.rs, `LendRefusal`), so a page tells it from the
// lend's (mnp-1).
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
// words (#111): no pass held, and a lend the pass does not open.
const unheld = () => say(503, 'Open this again from your group’s page.', 'pass');
const closed = () => say(403, 'This is not open to this page.', 'closed');

// The shim (shim.js) for a lend's page, with the cookies its pages read,
// and the nonce of the page's script policy when it names one: under
// `script-src 'nonce-…' 'strict-dynamic'` (Nextcloud's) a script without
// it does not run (u5-catalogue-dry).
/**
 * @param {string} label
 * @param {Cookie[]} jar
 * @param {string | null} policy
 */
const shim = (label, jar, policy, nonce = /'nonce-([^']+)'/.exec(policy ?? '')?.[1]) =>
  `<script${nonce ? ` nonce="${nonce}"` : ''} src="${SITE}shim.js?v=__VERSION__&l=${label}&c=${encodeURIComponent(cookieFor(jar, '/', true))}"></script>`;

// Serve `request`, at `url`, from the lend `where` names.
/**
 * @param {Request} request
 * @param {URL} url
 * @param {Where} where
 * @returns {Promise<Response>}
 */
async function lent(request, url, where, signs = false) {
  const held = await holding();
  if (!held) {
    if (request.mode !== 'navigate') return unheld();
    trace(`RUNTIME: no pass for ${OWN}: its start page reads it again`);
    return Response.redirect(
      `${SCOPE.origin}${SITE}open.html?v=__VERSION__&for=${OWN}&back=${encodeURIComponent(url.pathname + url.search)}`,
      302,
    );
  }
  const lend = held.lends.find((one) => one.label === where.label);
  if (!lend) return closed();
  if (request.method === 'POST' && lend.group && where.path === '/svrngs/acts') {
    if (
      !signs ||
      // `split` gives at least one part; the cast's parens end the chain,
      // and `?.` takes it on as the chain went.
      /** @type {string | undefined} */ (
        request.headers.get('content-type')?.split(';')[0]
      )?.trim() !== 'application/json'
    )
      return closed();
    return writing(request, lend);
  }
  if (where.read && request.mode === 'navigate') return opening(held, where);
  const jar = await jarOf(where.label);
  const headers = bounded(
    [...request.headers].filter(
      ([name]) => !['cookie', 'content-length'].includes(name.toLowerCase()),
    ),
    request.destination,
    where.path,
  );
  // A browser never shows a worker the `Host` it asked, and a page's
  // `fetch` cannot name one. The door tells the lender's service the
  // browser's origin as a proxy does (`X-Forwarded-Host`, oidc-jellyfin),
  // so the worker names the address the browser asked here.
  const host = new URL(request.url).host;
  if (host) headers.push(['host', host]);
  const cookie = cookieFor(jar, where.path, false);
  if (cookie) headers.push(['cookie', cookie]);
  headers.push(...(await passed(held)));
  // A browser names the origin a post comes from; a worker is not shown
  // it (WebKit's never, Chromium's but for a form's), and Nextcloud
  // refuses a sign-in posted without one (u5-catalogue-dry). Named here
  // for a post from this lend's own pages: a fetch, which only they can
  // make through this worker, or a page's form whose referrer is here; a
  // post another site sends here is left without, for the app to refuse.
  const posted = !['GET', 'HEAD'].includes(request.method);
  const named = headers.some(([name]) => name.toLowerCase() === 'origin');
  const from = request.referrer ? new URL(request.referrer).origin : null;
  if (posted && !named && (request.mode !== 'navigate' || from === self.location.origin)) {
    headers.push(['origin', self.location.origin]);
  }
  // A browser never shows a worker the `Accept-Encoding` it sends, and
  // llama-server's chat page comes only in gzip, 415 to a request without
  // (demo/runs/start/2026-10-13-story/ 9): so gzip is asked for here, but
  // for a range, and the answer unpacked below.
  if (!headers.some(([name]) => /^(range|accept-encoding)$/i.test(name)))
    headers.push(['accept-encoding', 'gzip']);
  // A body is read whole, as a Blob, which the browser keeps on disk when
  // it is large, so it goes with its length, a slice at a time (`sent`): a
  // worker never sees the page's Content-Length, and a parser that reads by
  // it — Django's multipart, Paperless-ngx's upload (u5-catalogue-dry) —
  // reads nothing from a stream sent without one.
  const path = `/lend/${where.label}${where.path}`;
  const blob = posted ? await request.blob() : null;
  if (blob) headers.push(['content-length', String(blob.size)]);
  const body = blob ? sent(blob, `${request.method} ${path}`) : null;
  const said = ranged(request, path);
  let res;
  for (let tries = 0; !res; tries++) {
    try {
      res = await (await dialFor(lend.key)).fetch(request.method, path, headers, body);
    } catch (/** @type {any} */ error) {
      dials.delete(lend.key);
      trace(`RUNTIME: ${request.method} ${path} failed (try ${tries + 1}): ${error}`);
      // A dial that heard nothing from the computer within the cap, new or
      // held, is not tried again: a second would wait the cap again.
      if (body || tries || /asleep/.test(error)) return away(request, lend);
    }
  }
  said?.(
    `answered ${res.status} ${res.headers.get('content-range') ?? ''}${request.signal?.aborted ? ', after the browser let go of it' : ''}`,
  );
  const set = res.headers.get('x-svrngs-set-cookies');
  if (set) {
    for (const line of JSON.parse(set)) setCookie(jar, line);
    await keepJar(where.label);
  }
  const naming = res.headers.get(APPEND);
  if (naming && !signs) return closed();
  const refused = naming && (await appended(lend, naming));
  if (refused) return refused;
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
    return Response.redirect(
      there,
      [301, 302, 303, 307, 308].includes(res.status) ? res.status : 302,
    );
  }
  // Unpacked from gzip as a browser unpacks what it asked for itself.
  const gzip = /^gzip$/i.test(res.headers.get('content-encoding') ?? '');
  const answer = gzip ? res.body?.pipeThrough(new DecompressionStream('gzip')) : res.body;
  const headed = new Headers(res.headers);
  ['x-svrngs-set-cookies', APPEND, ...(gzip ? ['content-encoding', 'content-length'] : [])].forEach(
    (header) => headed.delete(header),
  );
  // What its pages may read now, for the shim to take in before the page
  // reads the answer: Immich reads its sign-in's cookie unreloaded.
  if (set) headed.set(PAGE_COOKIES, cookieFor(jar, '/', true));
  const html =
    request.method === 'GET' && (res.headers.get('content-type') ?? '').includes('text/html');
  if (!html)
    return new Response(said && answer ? counted(answer, said) : answer, {
      status: res.status,
      statusText: res.statusText,
      headers: headed,
    });
  // The app's own policy, with the signer derived from this origin's host,
  // never an address supplied by app data, let in as a frame only where it
  // restricts frames (authority.js `framing`). The signer accepts scoped acts only.
  const { policy, framed } = framing(headed.get('content-security-policy'), SIGNER);
  trace(
    `RUNTIME: ${path}'s policy ${policy == null ? 'is none, so none is set' : `frames by ${framed.join(', ')}; the signer is let in where they restrict`}`,
  );
  if (policy != null) headed.set('content-security-policy', policy);
  headed.delete('content-length');
  const text = await new Response(answer).text();
  // A policy in the page's own `<meta>` this worker can't add the signer to
  // (DESIGN §3, the known gap): said, and its saves told why.
  walled = /<meta[^>]+content-security-policy/i.test(text) ? path : walled;
  if (walled === path)
    trace(`RUNTIME: ${path} sets its own policy in a <meta>, which may keep the signer out`);
  const head =
    /<head[^>]*>/i.exec(text) ?? /<html[^>]*>/i.exec(text) ?? /<!doctype[^>]*>/i.exec(text);
  const at = head ? head.index + head[0].length : 0;
  const page = `${text.slice(0, at)}${shim(where.label, jar, res.headers.get('content-security-policy'))}${text.slice(at)}`;
  return new Response(page, { status: res.status, statusText: res.statusText, headers: headed });
}

// An app's write is this phone's own (capability-signed; DESIGN §3): shaped
// as the door shapes one (svrngs-core `app_act`) and `signed`.
/**
 * @param {Request} request
 * @param {Passed} lend a lend of a group: `lent` writes for no other
 */
async function writing(request, lend) {
  await (loaded ??= init({ module_or_path: store.runtime() }));
  const act = app_act(lend.kind, new Uint8Array(await request.arrayBuffer()));
  if (!act)
    return Response.json({ error: 'an act is JSON, {op} or {corrects, op}' }, { status: 422 });
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
    acts = JSON.parse(named).map(
      (/** @type {{ group?: string }} */ act) =>
        act.group === handed.group &&
        app_act(lend.kind, new TextEncoder().encode(JSON.stringify(act))),
    );
  } catch (error) {
    trace(`RUNTIME: ${what}'s acts are not a list: ${error}`);
  }
  if (!handed.writes || !acts?.every(Boolean)) {
    trace(
      `RUNTIME: refused — ${what} named an act this phone does not write as its person: ${named}`,
    );
    return Response.json(
      {
        error: `${what} wrote outside its own part of the group you opened it from, so it was not saved.`,
      },
      { status: 403 },
    );
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

/**
 * @param {string} group
 * @param {string} line
 */
async function approved(group, line) {
  const pages = await /** @type {Scope} */ (self).clients.matchAll({ type: 'window' });
  const client = pages.find((one) => new URL(one.url).origin === SCOPE.origin);
  if (!client) throw new Error('no app page is open to ask the trusted signer');
  const channel = new MessageChannel();
  /** @type {Promise<string>} */
  const answer = new Promise((yes, no) => {
    const why = walled
      ? `This app’s page (${walled}) may keep out your group’s signer, so this wasn’t saved.`
      : 'the trusted signer did not answer';
    const timeout = setTimeout(() => no(new Error(why)), 15000);
    channel.port1.onmessage = ({ data }) => {
      clearTimeout(timeout);
      if (data.pass) yes(data.pass);
      else no(new Error(data.error));
    };
  });
  const authority = { ...JSON.parse(openMessage(handed)), proof: handed.proof };
  client.postMessage({ approve: { group, reach: handed.pass, line, authority }, signer: SIGNER }, [
    channel.port2,
  ]);
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
  const line = {
    ...JSON.parse(unsigned),
    sig: store.hex(new Uint8Array(await crypto.subtle.sign('Ed25519', handed.key, message))),
  };
  handed = {
    ...handed,
    line: { seq: line.seq, id: line.id },
    unsent: [...(handed.unsent ?? []), [group, line]],
  };
  await store.lendKeep(`pass ${OWN}`, handed);
  // `unsent` is set just above, and below each time `handed` is.
  trace(
    `RUNTIME: an act of ${lend.kind} signed as line ${line.seq}, ${/** @type {Unsent[]} */ (handed.unsent).length} to carry`,
  );
  let res;
  try {
    for (const carrying of /** @type {Unsent[]} */ (handed.unsent)) {
      const body = new TextEncoder().encode(JSON.stringify(carrying));
      const headers = [
        ['content-type', 'application/json'],
        ['content-length', String(body.length)],
      ];
      res = await (
        await dialFor(lend.key)
      ).fetch('POST', '/carry', headers, new Blob([body]).stream());
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
  } catch (error) {
    dials.delete(lend.key);
    trace(`RUNTIME: lines not carried: ${error}`);
    return Response.json(
      {
        error: `${lend.person}’s computer did not answer. It is kept on this phone and goes with your next entry`,
      },
      { status: 502 },
    );
  }
  // A computer on an older svrngs takes no signed line (F2): kept, said.
  if (!res.ok && !links(lend.version)) {
    const words = `${lend.person}’s computer runs an older svrngs: what you write here reaches your group when a computer on the new svrngs is on.`;
    return Response.json({ error: words }, { status: 503 });
  }
  // Taken but not counted: the fold refused it (svrngs-core `Grant`), said
  // to the page, never as saved.
  const said = res.ok ? await res.json() : null;
  if (said?.counted === false)
    return Response.json({ error: 'your group does not count it' }, { status: 409 });
  return said ? Response.json(said) : res;
}

// The lend's computer did not answer: a person's tab is shown its person's
// window, dark, naming the lend (away.html, which a computer's door shows
// too, scripts/build-web.sh copying it here); a page's own request, and a
// tab when that page cannot be fetched, svrngs's words.
/**
 * @param {Request} request
 * @param {Passed} lend
 */
async function away(request, lend) {
  const who = lend.person.replace(/</g, '&lt;');
  const page =
    request.mode === 'navigate' &&
    (await fetch(`${SITE}away.html?v=__VERSION__`)
      .then((response) => (response.ok ? response.text() : Promise.reject(response.status)))
      .catch((error) => trace(`RUNTIME: away.html not fetched: ${error}`)));
  trace(
    `RUNTIME: ${lend.label}'s computer did not answer, said ${page ? 'as its window, dark' : 'in svrngs’s words'}`,
  );
  if (!page) return say(502, 'The computer lending this did not answer. Try again later.', 'away');
  return new Response(
    page
      .replaceAll('{who}', who)
      .replaceAll('{what}', `${who}’s ${lend.name.replace(/</g, '&lt;')}`),
    { status: 502, headers: { 'content-type': 'text/html; charset=utf-8', [REFUSED]: 'away' } },
  );
}

// What this lend's pages are told of the lends they read: none of the
// pass. A handoff that names no one unanswered is from a page before
// mnp-2, which said nothing of them: asked for again, never read as none.
async function told() {
  const held = await holding();
  if (!held?.unanswered) return unheld();
  const lends = held.lends.filter((lend) => lend.label !== OWN);
  trace(
    `RUNTIME: told this lend's page the ${lends.length} lend(s) it reads, ${held.unanswered.length} unanswered`,
  );
  return Response.json({
    lends: lends.map((lend) => ({
      kind: lend.kind,
      name: lend.name,
      lender: lend.person,
      address: `${SCOPE.origin}${READS}/${lend.label}/`,
      ...(lend.asks
        ? {
            asks: lend.asks,
            ask: `${SCOPE.origin}${ASK}?lend=${lend.label}&app=${OWN}`,
            stop: `${SCOPE.origin}${READS}/${lend.label}/svrngs-stop`,
          }
        : {}),
    })),
    unanswered: held.unanswered,
  });
}

// The group's page at `to`, then `then` and the fragment this page was
// opened with, which the browser keeps from a worker; so a page of this
// worker's own sends it on, `<` escaped.
/**
 * @param {string} to
 * @param {string} then
 */
const onward = (
  to,
  then,
  [there, tail] = [to, then].map((value) => JSON.stringify(value).replace(/</g, '\\u003c')),
) =>
  new Response(
    `<!doctype html><script>location.replace(${there} + encodeURIComponent(${tail} + location.hash))</script>`,
    { headers: { 'content-type': 'text/html; charset=utf-8' } },
  );

// A page navigated to `where`, a lend it reads: the group's page opens that
// lend at the path.
/**
 * @param {Held} held
 * @param {Where} where
 */
const opening = (held, where) => (
  trace(
    `RUNTIME: a page opens ${where.label}, which it reads, at ${where.path}: through the group's page`,
  ),
  onward(`${held.page}#open=${where.label}&at=`, where.path)
);

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
  const body = new TextEncoder().encode('{"said":"stop"}');
  const headers = [
    ...(await passed(held)),
    ['content-type', 'application/json'],
    ['content-length', String(body.length)],
  ];
  trace(`RUNTIME: a page stops ${OWN} using ${label}`);
  try {
    return await (
      await dialFor(lend.key)
    ).fetch('POST', `/choices/${label}`, headers, new Blob([body]).stream());
  } catch (error) {
    dials.delete(lend.key);
    trace(`RUNTIME: the stop for ${label} failed: ${error}`);
    return say(502, 'The computer lending this did not answer. Try again later.', 'away');
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
    addEventListener: (type, listener) =>
      /** @type {Partial<On>} */ (on)[/** @type {keyof On} */ (type)]?.push(listener),
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
  const scope = new Proxy(/** @type {Record<string | symbol, any>} */ (self), {
    get: (target, property) =>
      property in own
        ? own[/** @type {keyof typeof own} */ (property)]
        : typeof target[property] === 'function'
          ? target[property].bind(target)
          : target[property],
  });
  new Function('self', 'addEventListener', 'skipWaiting', 'fetch', code)(
    scope,
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
    const code = await store.lendKept(`worker ${OWN}`).catch(() => null);
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
  const allowed = await maySign(fetchEvent, /** @type {Scope} */ (self).clients, SCOPE.origin);
  // App code must not turn an untrusted navigation into an authorized fetch.
  const on = allowed && where.label === OWN && (await appLoaded());
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
  return given ? given : lent(fetchEvent.request, url, where, allowed);
}

/** @type {Scope} */ (self).addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (url.origin === self.location.origin && url.pathname === TRACE) {
    return event.respondWith(
      kept().then(
        (lines) =>
          new Response(`svrngs lend worker __VERSION__ for ${OWN}\n${lines.join('\n')}`, {
            headers: PLAIN,
          }),
      ),
    );
  }
  const shell =
    url.origin === self.location.origin &&
    url.searchParams.has('v') &&
    url.pathname.startsWith(SITE);
  if (shell && SHELL.test(url.pathname.slice(SITE.length))) return;
  if (url.origin === self.location.origin && url.pathname === READS)
    return event.respondWith(told());
  if (url.origin === self.location.origin && url.pathname === ASK)
    return event.respondWith(asking(url));
  const stop =
    url.origin === self.location.origin &&
    event.request.method === 'POST' &&
    STOP.exec(url.pathname);
  // `STOP` names its label whenever it matches.
  if (stop) return event.respondWith(stopping(/** @type {string} */ (stop[1])));
  const where = whereOf(url);
  if (!where) return;
  event.respondWith(answered(event, url, where));
});

// ── what a lend's page hands over ───────────────────────────────────────

// A message from a page of this origin: a window's `Client`, of which a
// worker's `source` is one kind.
/** @type {Scope} */ (self).addEventListener('message', async (event) => {
  const reply = event.ports[0];
  if (
    !(/** @type {Client | null} */ (event.source)?.url) ||
    new URL(/** @type {Client} */ (event.source).url).origin !== SCOPE.origin
  )
    return reply?.postMessage({ error: 'only this origin hands the worker authority' });
  const { handed: given, cookie, worker, socket: asked } = event.data ?? {};
  if (given?.label === OWN && typeof given.pass === 'string') {
    // The holder's bytes, where its key was lost on the way (below):
    // imported here as a key no script can copy, held in memory alone.
    const bytes = given.key instanceof Uint8Array;
    if (bytes)
      given.key = await crypto.subtle.importKey('pkcs8', given.key, 'Ed25519', false, ['sign']);
    handed = given;
    dials.clear();
    endpoint = null;
    trace(
      `RUNTIME: the page handed over the pass for ${OWN}, ${given.lends.length} lend(s)${bytes ? ', and its holder’s bytes, imported here and kept nowhere' : ''}`,
    );
    return reply?.postMessage('held');
  }
  if (typeof cookie === 'string') {
    return event.waitUntil(
      jarOf(OWN)
        .then((jar) => (setCookie(jar, cookie), keepJar(OWN)))
        .then(() => reply?.postMessage('kept')),
    );
  }
  if (typeof worker === 'string') {
    const url = new URL(worker, SCOPE);
    const where = whereOf(url);
    return event.waitUntil(
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
  if (asked && reply) return event.waitUntil(socket(asked, reply));
  trace(`RUNTIME: a message that hands over nothing, refused: ${Object.keys(event.data ?? {})}`);
});

// Safari on an iPhone loses a message carrying a CryptoKey, the holder's,
// and fires this in its place (2026-10-30, demo/runs/ios/2026-10-30-observe/
// keyprobe/phone.log): its pages are told, so the start page hands the
// key's bytes at once (open.js).
/** @type {Scope} */ (self).addEventListener('messageerror', async () => {
  trace('RUNTIME: a message was lost on its way here: its pages are told');
  for (const client of await /** @type {Scope} */ (self).clients.matchAll({
    includeUncontrolled: true,
  }))
    client.postMessage(store.LOST);
});

// ── a WebSocket, over a socket the door switches ────────────────────────

// One frame from this end: masked, as a client's must be (RFC 6455 §5.3).
/**
 * @param {number} op
 * @param {string | ArrayBuffer | Uint8Array} data
 */
function frame(op, data) {
  const bytes = typeof data === 'string' ? new TextEncoder().encode(data) : new Uint8Array(data);
  const size = bytes.length;
  const head =
    size < 126
      ? [0x80 | op, 0x80 | size]
      : size < 65536
        ? [0x80 | op, 0xfe, size >> 8, size & 255]
        : [
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
  const mask = crypto.getRandomValues(new Uint8Array(4));
  const out = new Uint8Array(head.length + 4 + size);
  out.set(head);
  out.set(mask, head.length);
  // `i` is under `size`, the bytes' length, and `i & 3` under the mask's 4.
  for (let i = 0; i < size; i++)
    out[head.length + 4 + i] =
      /** @type {number} */ (bytes[i]) ^ /** @type {number} */ (mask[i & 3]);
  return out;
}

// Carry the WebSocket the shim opened at `url` to `port`: open, each
// message both ways, ping answered, close.
/**
 * @param {{ url: string, protocols?: string[] }} asked
 * @param {MessagePort} port
 */
async function socket({ url, protocols }, port) {
  const http = new URL(url.replace(/^ws/, 'http'));
  const where = whereOf(http);
  const held = await holding();
  const lend = where && held?.lends.find((one) => one.label === where.label);
  if (!lend) return port.postMessage({ closed: 1006, reason: 'not open to this page' });
  const key = btoa(String.fromCharCode(...crypto.getRandomValues(new Uint8Array(16))));
  const headers = [
    ['connection', 'upgrade'],
    ['upgrade', 'websocket'],
    ['sec-websocket-version', '13'],
    ['sec-websocket-key', key],
    // A lend was found, so the open is held.
    ...(await passed(/** @type {Held} */ (held))),
  ];
  if (protocols?.length) headers.push(['sec-websocket-protocol', protocols.join(', ')]);
  const cookie = cookieFor(await jarOf(where.label), where.path, false);
  if (cookie) headers.push(['cookie', cookie]);
  let remote;
  try {
    remote = await (await dialFor(lend.key)).socket(`/lend/${where.label}${where.path}`, headers);
  } catch (error) {
    // The dial let go, as `lent` does, so the next socket dials the
    // computer afresh once it answers again (demo/runs/doc/2026-10-20/found/).
    dials.delete(lend.key);
    trace(`RUNTIME: a socket at ${where.path} failed: ${error}`);
    return port.postMessage({ closed: 1006, reason: String(error) });
  }
  if (remote.status !== 101)
    return port.postMessage({ closed: 1006, reason: `answered ${remote.status}` });
  const writer = remote.writable.getWriter();
  const chosen =
    remote.headers.find(
      (/** @type {Header} */ [name]) => name.toLowerCase() === 'sec-websocket-protocol',
    )?.[1] ?? '';
  port.postMessage({ open: chosen });
  port.onmessage = ({ data }) => {
    if ('send' in data) writer.write(frame(typeof data.send === 'string' ? 1 : 2, data.send));
    if ('close' in data)
      writer.write(frame(8, new Uint8Array([data.close >> 8, data.close & 255])));
  };
  const reader = remote.readable.getReader();
  let buf = new Uint8Array(0);
  // A message's frames so far: its first's opcode, and each one's bytes.
  /** @typedef {{ op: number, data: Uint8Array }} Part */
  /** @type {Part[]} */
  let parts = [];
  for (;;) {
    const { value, done } = await reader.read();
    if (done) return port.postMessage({ closed: 1006, reason: 'the stream ended' });
    buf = Uint8Array.from([...buf, ...value]);
    while (buf.length >= 2) {
      // The head's bytes past its first two are read before `at + n` is
      // checked, so one not come yet is `undefined`, cast here as a byte. To
      // `<<` and `|` it is 0, and the check waits for it; to `*` it is NaN,
      // which the check lets by, and `slice(NaN)` keeps the whole buffer, so
      // a 64 KiB frame whose head comes split never ends this loop. Found by
      // `types`, and left as it runs.
      let length = /** @type {number} */ (buf[1]) & 127;
      let at = 2;
      if (length === 126)
        [length, at] = [(/** @type {number} */ (buf[2]) << 8) | /** @type {number} */ (buf[3]), 4];
      if (length === 127)
        [length, at] = [
          /** @type {number} */ (buf[6]) * 2 ** 24 +
            ((/** @type {number} */ (buf[7]) << 16) |
              (/** @type {number} */ (buf[8]) << 8) |
              /** @type {number} */ (buf[9])),
          10,
        ];
      if (buf.length < at + length) break;
      const [op, fin, data] = [
        /** @type {number} */ (buf[0]) & 15,
        /** @type {number} */ (buf[0]) & 128,
        buf.slice(at, at + length),
      ];
      buf = buf.slice(at + length);
      if (op === 9) writer.write(frame(10, data));
      if (op === 8) {
        port.postMessage({
          closed:
            data.length >= 2
              ? (/** @type {number} */ (data[0]) << 8) | /** @type {number} */ (data[1])
              : 1005,
          reason: new TextDecoder().decode(data.slice(2)),
        });
        return writer.close().catch(() => {});
      }
      if (op === 0 || op === 1 || op === 2) parts.push({ op, data });
      if (fin && (op === 0 || op === 1 || op === 2)) {
        const whole = Uint8Array.from(parts.flatMap((part) => [...part.data]));
        port.postMessage({
          // A part was pushed just above.
          message:
            /** @type {Part} */ (parts[0]).op === 1
              ? new TextDecoder().decode(whole)
              : whole.buffer,
        });
        parts = [];
      }
    }
  }
}
