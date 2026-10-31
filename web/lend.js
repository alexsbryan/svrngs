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
import init, { actor, app_act, holder_endpoint, link, links, unsigned_line } from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';

const SCOPE = new URL(self.registration.scope);
// Where the page's files are on this origin, the shell this worker leaves
// to the network: any version of them, so a newer page starts here too.
const SITE = new URL('./', self.location).pathname;
const SHELL = /^(open\.html|open\.js|lend\.js|shim\.js|store\.js|wasm\/svrngs_web(_bg\.wasm|\.js))$/;
// This worker's lend: the first name of its address.
const OWN = store.labelOf(SCOPE.hostname);
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
const traced = [];
// The last runs' lines, read once; this run's are kept after them, a
// second at most after each is written.
const before = store.lendKept(`trace ${OWN}`).then((l) => l ?? [], () => []);
let keeping = null;
const kept = async () => [...(await before), ...traced].slice(-LINES);
self.__trace = (l) => {
  console.debug(l);
  traced.push(`${new Date().toISOString()} ${l}`);
  if (traced.length > LINES) traced.shift();
  keeping ??= setTimeout(async () => {
    keeping = null;
    await store.lendKeep(`trace ${OWN}`, await kept()).catch((e) => console.debug(`RUNTIME: the trace was not kept: ${e}`));
  }, 1000);
};
const trace = self.__trace;
trace(`RUNTIME: this run of the worker begins`);

self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (e) => e.waitUntil(self.clients.claim()));

// What this run of the worker holds, all of it lost when the browser stops
// it: what the page handed it, the wasm, its endpoint, a dial per lender,
// each lend's cookies, the app's worker.
let handed = null;
let loaded = null;
let endpoint = null;
let linked = null;
const dials = new Map();
const jars = new Map();
let app;

// Where `label`'s pages are: this worker's lend under its scope; another
// at an address of its own beside this one.
const baseOf = (label) => (label === OWN ? SCOPE.href : `${store.lendOrigin(SCOPE.protocol, label, SCOPE.host.slice(OWN.length + 1))}/`);

// The lend `url` is at, and its path there; or none. Under `READS`, a
// lend this page reads, by its label.
function at(url) {
  const [, read, path] = (url.origin === SCOPE.origin && url.pathname.match(new RegExp(`^${READS}/([^/]+)(/.*)?$`))) || [];
  if (read) return { label: read, path: `${path ?? '/'}${url.search}`, read: true };
  for (const label of [OWN, ...(handed?.lends ?? []).map((l) => l.label)]) {
    const base = new URL(baseOf(label));
    if (url.origin === base.origin && url.pathname.startsWith(base.pathname)) {
      return { label, path: `/${url.pathname.slice(base.pathname.length)}${url.search}` };
    }
  }
  return null;
}

async function holding() {
  if (handed) return handed;
  try {
    handed = (await store.lendKept(`pass ${OWN}`)) ?? null;
    trace(`RUNTIME: the pass for ${OWN} ${handed ? 'read from this origin’s store' : 'is not kept here'}`);
  } catch (e) {
    trace(`RUNTIME: this origin's store cannot be read here: ${e}`);
  }
  return handed;
}

function dialFor(lender) {
  if (!dials.has(lender)) {
    const opening = (async () => {
      endpoint ??= opened();
      trace(`RUNTIME: dialling ${lender} for the pass's holder`);
      // Through the relay its group names for the lender (relay-in-record).
      return (await endpoint).dial_lend(lender, handed.lends.find((l) => l.key === lender)?.relay ?? null);
    })();
    dials.set(lender, opening);
    opening.catch(() => dials.delete(lender));
  }
  return dials.get(lender);
}

// This run's endpoint (capability-signed (H); DESIGN §3): a key of this
// run's alone, which the holder, a key no page can copy, names in a link
// the door checks beside the pass (`linked`); or, when a lend of the open
// is at a door taking no link, the holder's own bytes, as before (F2).
async function opened() {
  await (loaded ??= init({ module_or_path: store.runtime() }));
  const own = !handed.seed;
  const bytes = own ? crypto.getRandomValues(new Uint8Array(32)) : Uint8Array.from(handed.seed.match(/../g), (h) => parseInt(h, 16));
  linked = null;
  if (own) {
    const to = actor(bytes);
    const sig = new Uint8Array(await crypto.subtle.sign('Ed25519', handed.key, link(handed.pass, to)));
    linked = new TextDecoder().decode(link(handed.pass, to, sig));
  }
  trace(`RUNTIME: opening the endpoint ${own ? 'of this run, named by the holder’s link' : 'as the pass’s holder, for an older door'}`);
  return holder_endpoint(bytes, handed.relay);
}

// The pass a request carries, and the link naming this run's endpoint
// when the holder signed one (`opened`).
const passed = async (h) => (await (endpoint ??= opened()), [['x-svrngs-pass', h.pass], ...(linked ? [['x-svrngs-link', linked]] : [])]);

// ── cookies, a jar per lend ─────────────────────────────────────────────

const jarOf = async (label) => (jars.has(label) || jars.set(label, (await store.lendKept(`cookies ${label}`).catch(() => null)) ?? []), jars.get(label));

const live = (c) => c.expires === null || c.expires > Date.now();

// Keep a `Set-Cookie` value in `jar`: its name, value, path, lapse and
// whether pages may read it. Any other attribute is the browser's, and
// there is no other site here to keep it from.
function setCookie(jar, line) {
  const [pair, ...attrs] = line.split(';');
  const eq = pair.indexOf('=');
  if (eq < 1) return;
  const c = { name: pair.slice(0, eq).trim(), value: pair.slice(eq + 1).trim(), path: '/', expires: null, page: true };
  for (const attr of attrs) {
    const [k, ...v] = attr.split('=');
    const [key, value] = [k.trim().toLowerCase(), v.join('=').trim()];
    if (key === 'path' && value.startsWith('/')) c.path = value;
    if (key === 'max-age') c.expires = Date.now() + Number(value) * 1000;
    if (key === 'expires' && c.expires === null) c.expires = Date.parse(value);
    if (key === 'httponly') c.page = false;
  }
  const kept = jar.filter((o) => o.name !== c.name || o.path !== c.path);
  jar.splice(0, jar.length, ...kept, ...(live(c) ? [c] : []));
}

const cookieFor = (jar, path, page) =>
  jar.filter((c) => live(c) && path.startsWith(c.path) && (c.page || !page)).map((c) => `${c.name}=${c.value}`).join('; ');

async function keepJar(label) {
  await store.lendKeep(`cookies ${label}`, await jarOf(label)).catch((e) => trace(`RUNTIME: cookies not kept: ${e}`));
}

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
const ends = new Map();

function bounded(headers, destination, path) {
  return headers.map(([name, value]) => {
    const range = name.toLowerCase() === 'range' && /^bytes=(\d+)-(\d*)$/.exec(value.trim());
    if (!range) return [name, value];
    const from = Number(range[1]);
    const media = destination === 'video' || destination === 'audio';
    const size = range[2] !== '' ? PART : Math.min(PART, 2 * (ends.get(`${path} ${from}`) ?? FIRST / 2));
    if (range[2] !== '' && !(media && Number(range[2]) - from + 1 > size)) return [name, value];
    ends.delete(`${path} ${from}`);
    ends.set(`${path} ${from + size}`, size);
    if (ends.size > 64) ends.delete(ends.keys().next().value);
    trace(`RUNTIME: ${value} for ${destination || 'a fetch'} runs on: asked as a part of ${size} bytes`);
    return [name, `bytes=${from}-${from + size - 1}`];
  });
}

// A range, or a media element's ask, traced from asked to its end
// (u5-11): answered, read whole, cancelled by the browser after so many
// bytes, or let go of by the browser before its answer came — what a seek
// decides, which the dial's own lines do not show. None for any other
// request.
let asks = 0;
function ranged(request, path) {
  if (!request.headers.has('range') && !['video', 'audio'].includes(request.destination)) return null;
  const [n, t0] = [++asks, Date.now()];
  const said = (what) => trace(`RANGE ${n}: ${what}, ${Date.now() - t0} ms`);
  said(`asked ${path} ${request.headers.get('range') ?? 'whole'} for ${request.destination || 'a fetch'}`);
  request.signal?.addEventListener('abort', () => said('the browser let go of it'));
  return said;
}

// `body`, its reading traced by `said`.
function counted(body, said) {
  const reader = body.getReader();
  let [bytes, cancelled] = [0, false];
  return new ReadableStream({
    async pull(c) {
      const { value, done } = await reader.read().catch((e) => (said(`the body failed after ${bytes} bytes: ${e}`), Promise.reject(e)));
      if (done && !cancelled) { said(`read whole, ${bytes} bytes`); c.close(); }
      if (done) return;
      bytes += value.byteLength;
      c.enqueue(value);
    },
    cancel: (why) => ((cancelled = true), said(`cancelled by the browser after ${bytes} bytes: ${why}`), reader.cancel(why)),
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
function sent(blob, what) {
  const t0 = Date.now();
  let at = 0;
  trace(`RUNTIME: ${what} sends a body of ${blob.size} bytes`);
  return new ReadableStream({
    async pull(c) {
      if (at >= blob.size) return trace(`RUNTIME: ${what} handed its body to the dial whole, ${Date.now() - t0} ms`), c.close();
      c.enqueue(new Uint8Array(await blob.slice(at, (at += SLICE)).arrayBuffer()));
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
const say = (status, words, kind) => new Response(words, { status, headers: { ...PLAIN, [REFUSED]: kind } });
// The two refusals a page meets at the pass and the lend, each in one set of
// words (#111): no pass held, and a lend the pass does not open.
const unheld = () => say(503, 'Open this again from your group’s page.', 'pass');
const closed = () => say(403, 'This is not open to this page.', 'closed');

// The shim (shim.js) for a lend's page, with the cookies its pages read,
// and the nonce of the page's script policy when it names one: under
// `script-src 'nonce-…' 'strict-dynamic'` (Nextcloud's) a script without
// it does not run (u5-catalogue-dry).
const shim = (label, jar, policy, nonce = /'nonce-([^']+)'/.exec(policy ?? '')?.[1]) => `<script${nonce ? ` nonce="${nonce}"` : ''} src="${SITE}shim.js?v=__VERSION__&l=${label}&c=${encodeURIComponent(cookieFor(jar, '/', true))}"></script>`;

// Serve `request`, at `url`, from the lend `where` names.
async function lent(request, url, where) {
  const h = await holding();
  if (!h) {
    if (request.mode !== 'navigate') return unheld();
    trace(`RUNTIME: no pass for ${OWN}: its start page reads it again`);
    return Response.redirect(`${SCOPE.origin}${SITE}open.html?v=__VERSION__&for=${OWN}&back=${encodeURIComponent(url.pathname + url.search)}`, 302);
  }
  const lend = h.lends.find((l) => l.label === where.label);
  if (!lend) return closed();
  if (request.method === 'POST' && lend.group && where.path === '/svrngs/acts') return writing(request, lend);
  if (where.read && request.mode === 'navigate') return opening(h, where);
  const jar = await jarOf(where.label);
  const headers = bounded(
    [...request.headers].filter(([n]) => !['cookie', 'content-length'].includes(n.toLowerCase())),
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
  headers.push(...(await passed(h)));
  // A browser names the origin a post comes from; a worker is not shown
  // it (WebKit's never, Chromium's but for a form's), and Nextcloud
  // refuses a sign-in posted without one (u5-catalogue-dry). Named here
  // for a post from this lend's own pages: a fetch, which only they can
  // make through this worker, or a page's form whose referrer is here; a
  // post another site sends here is left without, for the app to refuse.
  const posted = !['GET', 'HEAD'].includes(request.method);
  const named = headers.some(([n]) => n.toLowerCase() === 'origin');
  const from = request.referrer ? new URL(request.referrer).origin : null;
  if (posted && !named && (request.mode !== 'navigate' || from === self.location.origin)) {
    headers.push(['origin', self.location.origin]);
  }
  // A browser never shows a worker the `Accept-Encoding` it sends, and
  // llama-server's chat page comes only in gzip, 415 to a request without
  // (demo/runs/start/2026-10-13-story/ 9): so gzip is asked for here, but
  // for a range, and the answer unpacked below.
  if (!headers.some(([n]) => /^(range|accept-encoding)$/i.test(n))) headers.push(['accept-encoding', 'gzip']);
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
    } catch (e) {
      dials.delete(lend.key);
      trace(`RUNTIME: ${request.method} ${path} failed (try ${tries + 1}): ${e}`);
      // A dial that heard nothing from the computer within the cap, new or
      // held, is not tried again: a second would wait the cap again.
      if (body || tries || /asleep/.test(e)) return away(request, lend);
    }
  }
  said?.(`answered ${res.status} ${res.headers.get('content-range') ?? ''}${request.signal?.aborted ? ', after the browser let go of it' : ''}`);
  const set = res.headers.get('x-svrngs-set-cookies');
  if (set) {
    for (const line of JSON.parse(set)) setCookie(jar, line);
    await keepJar(where.label);
  }
  const naming = res.headers.get(APPEND);
  const refused = naming && (await appended(lend, naming));
  if (refused) return refused;
  if (res.headers.get(REFUSED) === 'pass' && request.mode === 'navigate') {
    trace(`RUNTIME: the pass for ${where.label} is refused: its start page asks the group's page for another`);
    return Response.redirect(`${SCOPE.origin}${SITE}open.html?v=__VERSION__&for=${OWN}&again`, 302);
  }
  const to = res.headers.get('location');
  if (res.status >= 300 && res.status < 400 && to) {
    const there = new URL(to.startsWith('/') ? `.${to}` : to, baseOf(where.label)).href;
    trace(`RUNTIME: ${path} redirects to ${to}, read as ${there}`);
    return Response.redirect(there, [301, 302, 303, 307, 308].includes(res.status) ? res.status : 302);
  }
  // Unpacked from gzip as a browser unpacks what it asked for itself.
  const gzip = /^gzip$/i.test(res.headers.get('content-encoding') ?? '');
  const answer = gzip ? res.body?.pipeThrough(new DecompressionStream('gzip')) : res.body;
  const headed = new Headers(res.headers);
  ['x-svrngs-set-cookies', APPEND, ...(gzip ? ['content-encoding', 'content-length'] : [])].forEach((h) => headed.delete(h));
  // What its pages may read now, for the shim to take in before the page
  // reads the answer: Immich reads its sign-in's cookie unreloaded.
  if (set) headed.set(PAGE_COOKIES, cookieFor(jar, '/', true));
  const html = request.method === 'GET' && (res.headers.get('content-type') ?? '').includes('text/html');
  if (!html) return new Response(said && answer ? counted(answer, said) : answer, { status: res.status, statusText: res.statusText, headers: headed });
  headed.delete('content-length');
  const text = await new Response(answer).text();
  const head = /<head[^>]*>/i.exec(text) ?? /<html[^>]*>/i.exec(text) ?? /<!doctype[^>]*>/i.exec(text);
  const at = head ? head.index + head[0].length : 0;
  const page = `${text.slice(0, at)}${shim(where.label, jar, res.headers.get('content-security-policy'))}${text.slice(at)}`;
  return new Response(page, { status: res.status, statusText: res.statusText, headers: headed });
}

// An app's write is this phone's own (capability-signed; DESIGN §3): shaped
// as the door shapes one (svrngs-core `app_act`) and `signed`.
async function writing(request, lend) {
  await (loaded ??= init({ module_or_path: store.runtime() }));
  const act = app_act(lend.kind, new Uint8Array(await request.arrayBuffer()));
  if (!act) return Response.json({ error: 'an act is JSON, {op} or {corrects, op}' }, { status: 422 });
  return signed(lend, lend.group, act, handed.pass);
}

// The acts a lent app's answer names (`x-svrngs-append`), this phone's own
// as a carried app's are: each shaped under the lend's kind, in the group
// it was opened from, and `signed` under the pass that lets its holder
// write that kind's part (app.js `writes`); none of them if one is not.
// Nothing when all are counted, else what the person is told.
async function appended(lend, named) {
  await (loaded ??= init({ module_or_path: store.runtime() }));
  const what = `${lend.person}’s ${lend.name}`;
  let acts = null;
  try {
    acts = JSON.parse(named).map((a) => a.group === handed.group && app_act(lend.kind, new TextEncoder().encode(JSON.stringify(a))));
  } catch (e) {
    trace(`RUNTIME: ${what}'s acts are not a list: ${e}`);
  }
  if (!handed.writes || !acts?.every(Boolean)) {
    trace(`RUNTIME: refused — ${what} named an act this phone does not write as its person: ${named}`);
    return Response.json({ error: `${what} wrote outside its own part of the group you opened it from, so it was not saved.` }, { status: 403 });
  }
  for (const act of acts) {
    const res = await signed(lend, handed.group, act, handed.writes);
    if (!res.ok) return res;
  }
  return null;
}

// `act` signed in `group` by the pass's holder under `grant` as its chain's
// next line, kept beside the pass before it is sent, so no seq is signed
// twice, and carried with every line not yet taken to the lend's computer
// (`/carry`), which writes none of its own. A line not taken waits, the
// same bytes, for the next write.
async function signed(lend, group, act, grant) {
  const at = handed.line ?? { seq: -1, id: null };
  const [unsigned, message] = unsigned_line(handed.holder, group, Math.floor(Date.now() / 1000), at.seq + 1, at.id, act, grant);
  const line = { ...JSON.parse(unsigned), sig: store.hex(new Uint8Array(await crypto.subtle.sign('Ed25519', handed.key, message))) };
  handed = { ...handed, line: { seq: line.seq, id: line.id }, unsent: [...(handed.unsent ?? []), [group, line]] };
  await store.lendKeep(`pass ${OWN}`, handed);
  trace(`RUNTIME: an act of ${lend.kind} signed as line ${line.seq}, ${handed.unsent.length} to carry`);
  let res;
  try {
    for (const sent of handed.unsent) {
      const body = new TextEncoder().encode(JSON.stringify(sent));
      const headers = [['content-type', 'application/json'], ['content-length', String(body.length)]];
      res = await (await dialFor(lend.key)).fetch('POST', '/carry', headers, new Blob([body]).stream());
      trace(`RUNTIME: line ${sent[1].seq} carried to ${lend.person}'s computer, answered ${res.status}`);
      if (!res.ok) break;
      handed = { ...handed, unsent: handed.unsent.filter((u) => u[1].id !== sent[1].id) };
      await store.lendKeep(`pass ${OWN}`, handed);
    }
  } catch (e) {
    dials.delete(lend.key);
    trace(`RUNTIME: lines not carried: ${e}`);
    return Response.json({ error: `${lend.person}’s computer did not answer. It is kept on this phone and goes with your next entry` }, { status: 502 });
  }
  // A computer on an older svrngs takes no signed line (F2): kept, said.
  if (!res.ok && !links(lend.version)) {
    const words = `${lend.person}’s computer runs an older svrngs: what you write here reaches your group when a computer on the new svrngs is on.`;
    return Response.json({ error: words }, { status: 503 });
  }
  // Taken but not counted: the fold refused it (svrngs-core `Grant`), said
  // to the page, never as saved.
  const said = res.ok ? await res.json() : null;
  if (said?.counted === false) return Response.json({ error: 'your group does not count it' }, { status: 409 });
  return said ? Response.json(said) : res;
}

// The lend's computer did not answer: a person's tab is shown its person's
// window, dark, naming the lend (away.html, which a computer's door shows
// too, scripts/build-web.sh copying it here); a page's own request, and a
// tab when that page cannot be fetched, svrngs's words.
async function away(request, lend) {
  const who = lend.person.replace(/</g, '&lt;');
  const page = request.mode === 'navigate' && (await fetch(`${SITE}away.html?v=__VERSION__`).then((r) => (r.ok ? r.text() : Promise.reject(r.status))).catch((e) => trace(`RUNTIME: away.html not fetched: ${e}`)));
  trace(`RUNTIME: ${lend.label}'s computer did not answer, said ${page ? 'as its window, dark' : 'in svrngs’s words'}`);
  if (!page) return say(502, 'The computer lending this did not answer. Try again later.', 'away');
  return new Response(page.replaceAll('{who}', who).replaceAll('{what}', `${who}’s ${lend.name.replace(/</g, '&lt;')}`), { status: 502, headers: { 'content-type': 'text/html; charset=utf-8', [REFUSED]: 'away' } });
}

// What this lend's pages are told of the lends they read: none of the
// pass. A handoff that names no one unanswered is from a page before
// mnp-2, which said nothing of them: asked for again, never read as none.
async function told() {
  const h = await holding();
  if (!h?.unanswered) return unheld();
  const lends = h.lends.filter((l) => l.label !== OWN);
  trace(`RUNTIME: told this lend's page the ${lends.length} lend(s) it reads, ${h.unanswered.length} unanswered`);
  return Response.json({
    lends: lends.map((l) => ({
      kind: l.kind,
      name: l.name,
      lender: l.person,
      address: `${SCOPE.origin}${READS}/${l.label}/`,
      ...(l.asks ? { asks: l.asks, ask: `${SCOPE.origin}${ASK}?lend=${l.label}&app=${OWN}`, stop: `${SCOPE.origin}${READS}/${l.label}/svrngs-stop` } : {}),
    })),
    unanswered: h.unanswered,
  });
}

// The group's page at `to`, then `then` and the fragment this page was
// opened with, which the browser keeps from a worker; so a page of this
// worker's own sends it on, `<` escaped.
function onward(to, then) {
  const [there, tail] = [to, then].map((x) => JSON.stringify(x).replace(/</g, '\\u003c'));
  const page = `<!doctype html><script>location.replace(${there} + encodeURIComponent(${tail} + location.hash))</script>`;
  return new Response(page, { headers: { 'content-type': 'text/html; charset=utf-8' } });
}

// A page navigated to `where`, a lend it reads: the group's page opens that
// lend at the path.
const opening = (h, where) => (trace(`RUNTIME: a page opens ${where.label}, which it reads, at ${where.path}: through the group's page`), onward(`${h.page}#open=${where.label}&at=`, where.path));

// A page sends its person to say yes to this lend using `lend`, one it
// reads: the group's page asks, then opens this lend again at `back`.
async function asking(url) {
  const h = await holding();
  const lend = url.searchParams.get('lend');
  if (!h) return unheld();
  if (!h.lends.some((l) => l.label === lend && l.label !== OWN)) {
    trace(`RUNTIME: a page asks for ${lend}, which it does not read: refused`);
    return closed();
  }
  trace(`RUNTIME: a page asks its person to let ${OWN} use ${lend}: through the group's page`);
  return onward(`${h.page}#ask=${lend}&app=${OWN}&at=`, url.searchParams.get('back') ?? '/');
}

// A page stops this lend using `label`: sent to its door with the pass.
async function stopping(label) {
  const h = await holding();
  if (!h) return unheld();
  const lend = h.lends.find((l) => l.label === label);
  if (!lend) {
    trace(`RUNTIME: a page stops ${OWN} using ${label}, which it does not read: refused`);
    return closed();
  }
  const body = new TextEncoder().encode('{"said":"stop"}');
  const headers = [...(await passed(h)), ['content-type', 'application/json'], ['content-length', String(body.length)]];
  trace(`RUNTIME: a page stops ${OWN} using ${label}`);
  try {
    return await (await dialFor(lend.key)).fetch('POST', `/choices/${label}`, headers, new Blob([body]).stream());
  } catch (e) {
    dials.delete(lend.key);
    trace(`RUNTIME: the stop for ${label} failed: ${e}`);
    return say(502, 'The computer lending this did not answer. Try again later.', 'away');
  }
}

// ── the app's own worker ────────────────────────────────────────────────
//
// An app that registers a worker of its own would put it over this one,
// which serves its every request; the shim hands its script here instead,
// kept in this origin's store, and its listeners are asked first.

function appWorker(code) {
  const on = { fetch: [], install: [], activate: [], message: [] };
  const own = {
    addEventListener: (type, f) => on[type]?.push(f),
    skipWaiting: async () => {},
    fetch: (input, init) => {
      const request = new Request(new URL(input?.url ?? input, SCOPE), init);
      const where = at(new URL(request.url));
      return where ? lent(request, new URL(request.url), where) : fetch(request);
    },
  };
  const scope = new Proxy(self, {
    get: (o, k) => (k in own ? own[k] : typeof o[k] === 'function' ? o[k].bind(o) : o[k]),
  });
  new Function('self', 'addEventListener', 'skipWaiting', 'fetch', code)(scope, own.addEventListener, own.skipWaiting, own.fetch);
  for (const f of [...on.install, ...on.activate]) f({ waitUntil: () => {} });
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
async function answered(e, url, where) {
  const on = where.label === OWN && (await appLoaded());
  let given = null;
  if (on) {
    const event = { request: e.request, clientId: e.clientId, respondWith: (p) => (given ??= p), waitUntil: () => {} };
    for (const f of on.fetch) f.call(self, event);
  }
  return given ? given : lent(e.request, url, where);
}

self.addEventListener('fetch', (e) => {
  const url = new URL(e.request.url);
  if (url.origin === self.location.origin && url.pathname === TRACE) {
    return e.respondWith(kept().then((lines) => new Response(`svrngs lend worker __VERSION__ for ${OWN}\n${lines.join('\n')}`, { headers: PLAIN })));
  }
  const shell = url.origin === self.location.origin && url.searchParams.has('v') && url.pathname.startsWith(SITE);
  if (shell && SHELL.test(url.pathname.slice(SITE.length))) return;
  if (url.origin === self.location.origin && url.pathname === READS) return e.respondWith(told());
  if (url.origin === self.location.origin && url.pathname === ASK) return e.respondWith(asking(url));
  const stop = url.origin === self.location.origin && e.request.method === 'POST' && STOP.exec(url.pathname);
  if (stop) return e.respondWith(stopping(stop[1]));
  const where = at(url);
  if (!where) return;
  e.respondWith(answered(e, url, where));
});

// ── what a lend's page hands over ───────────────────────────────────────

self.addEventListener('message', async (e) => {
  const reply = e.ports[0];
  const { handed: given, cookie, worker, socket: asked } = e.data ?? {};
  if (given?.label === OWN && typeof given.pass === 'string') {
    // The holder's bytes, where its key was lost on the way (below):
    // imported here as a key no script can copy, held in memory alone.
    const bytes = given.key instanceof Uint8Array;
    if (bytes) given.key = await crypto.subtle.importKey('pkcs8', given.key, 'Ed25519', false, ['sign']);
    handed = given;
    dials.clear();
    endpoint = null;
    trace(`RUNTIME: the page handed over the pass for ${OWN}, ${given.lends.length} lend(s)${bytes ? ', and its holder’s bytes, imported here and kept nowhere' : ''}`);
    return reply?.postMessage('held');
  }
  if (typeof cookie === 'string') {
    return e.waitUntil(jarOf(OWN).then((jar) => (setCookie(jar, cookie), keepJar(OWN))).then(() => reply?.postMessage('kept')));
  }
  if (typeof worker === 'string') {
    const url = new URL(worker, SCOPE);
    const where = at(url);
    return e.waitUntil(
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
  if (asked && reply) return e.waitUntil(socket(asked, reply));
  trace(`RUNTIME: a message that hands over nothing, refused: ${Object.keys(e.data ?? {})}`);
});

// Safari on an iPhone loses a message carrying a CryptoKey, the holder's,
// and fires this in its place (2026-10-30, demo/runs/ios/2026-10-30-observe/
// keyprobe/phone.log): its pages are told, so the start page hands the
// key's bytes at once (open.js).
self.addEventListener('messageerror', async () => {
  trace('RUNTIME: a message was lost on its way here: its pages are told');
  for (const c of await self.clients.matchAll({ includeUncontrolled: true })) c.postMessage(store.LOST);
});

// ── a WebSocket, over a socket the door switches ────────────────────────

// One frame from this end: masked, as a client's must be (RFC 6455 §5.3).
function frame(op, data) {
  const bytes = typeof data === 'string' ? new TextEncoder().encode(data) : new Uint8Array(data);
  const n = bytes.length;
  const head = n < 126 ? [0x80 | op, 0x80 | n] : n < 65536 ? [0x80 | op, 0xfe, n >> 8, n & 255] : [0x80 | op, 0xff, 0, 0, 0, 0, n >>> 24, (n >>> 16) & 255, (n >>> 8) & 255, n & 255];
  const mask = crypto.getRandomValues(new Uint8Array(4));
  const out = new Uint8Array(head.length + 4 + n);
  out.set(head);
  out.set(mask, head.length);
  for (let i = 0; i < n; i++) out[head.length + 4 + i] = bytes[i] ^ mask[i & 3];
  return out;
}

// Carry the WebSocket the shim opened at `url` to `port`: open, each
// message both ways, ping answered, close.
async function socket({ url, protocols }, port) {
  const http = new URL(url.replace(/^ws/, 'http'));
  const where = at(http);
  const h = await holding();
  const lend = where && h?.lends.find((l) => l.label === where.label);
  if (!lend) return port.postMessage({ closed: 1006, reason: 'not open to this page' });
  const key = btoa(String.fromCharCode(...crypto.getRandomValues(new Uint8Array(16))));
  const headers = [['connection', 'upgrade'], ['upgrade', 'websocket'], ['sec-websocket-version', '13'], ['sec-websocket-key', key], ...(await passed(h))];
  if (protocols?.length) headers.push(['sec-websocket-protocol', protocols.join(', ')]);
  const cookie = cookieFor(await jarOf(where.label), where.path, false);
  if (cookie) headers.push(['cookie', cookie]);
  let s;
  try {
    s = await (await dialFor(lend.key)).socket(`/lend/${where.label}${where.path}`, headers);
  } catch (e) {
    // The dial let go, as `lent` does, so the next socket dials the
    // computer afresh once it answers again (demo/runs/doc/2026-10-20/found/).
    dials.delete(lend.key);
    trace(`RUNTIME: a socket at ${where.path} failed: ${e}`);
    return port.postMessage({ closed: 1006, reason: String(e) });
  }
  if (s.status !== 101) return port.postMessage({ closed: 1006, reason: `answered ${s.status}` });
  const writer = s.writable.getWriter();
  const chosen = s.headers.find(([n]) => n.toLowerCase() === 'sec-websocket-protocol')?.[1] ?? '';
  port.postMessage({ open: chosen });
  port.onmessage = ({ data }) => {
    if ('send' in data) writer.write(frame(typeof data.send === 'string' ? 1 : 2, data.send));
    if ('close' in data) writer.write(frame(8, new Uint8Array([data.close >> 8, data.close & 255])));
  };
  const reader = s.readable.getReader();
  let buf = new Uint8Array(0);
  let parts = [];
  for (;;) {
    const { value, done } = await reader.read();
    if (done) return port.postMessage({ closed: 1006, reason: 'the stream ended' });
    buf = Uint8Array.from([...buf, ...value]);
    while (buf.length >= 2) {
      let n = buf[1] & 127;
      let at = 2;
      if (n === 126) [n, at] = [(buf[2] << 8) | buf[3], 4];
      if (n === 127) [n, at] = [buf[6] * 2 ** 24 + ((buf[7] << 16) | (buf[8] << 8) | buf[9]), 10];
      if (buf.length < at + n) break;
      const [op, fin, data] = [buf[0] & 15, buf[0] & 128, buf.slice(at, at + n)];
      buf = buf.slice(at + n);
      if (op === 9) writer.write(frame(10, data));
      if (op === 8) {
        port.postMessage({ closed: data.length >= 2 ? (data[0] << 8) | data[1] : 1005, reason: new TextDecoder().decode(data.slice(2)) });
        return writer.close().catch(() => {});
      }
      if (op === 0 || op === 1 || op === 2) parts.push({ op, data });
      if (fin && (op === 0 || op === 1 || op === 2)) {
        const whole = Uint8Array.from(parts.flatMap((p) => [...p.data]));
        port.postMessage({ message: parts[0].op === 1 ? new TextDecoder().decode(whole) : whole.buffer });
        parts = [];
      }
    }
  }
}
