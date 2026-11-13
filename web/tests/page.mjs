// What the browser halves share (drive.mjs, join.mjs, runtime.mjs): the
// built page served as an origin serves it, Playwright's headless Chromium
// or WebKit as an iPhone, and a joined group held in the page's store.
// Each exits 3 naming the install command when Playwright or its browser
// is absent — the browser step fails then, never skips.
import http from 'node:http';
import net from 'node:net';
import tls from 'node:tls';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const INSTALL = 'npm ci --prefix web && npx --prefix web playwright install chromium webkit';

// The page as an origin serves it: its directory at the root, with the
// trailing slash, and the wasm by its type so the browser streams it,
// on `port` when given, as a page a group keeps is at one address.
// With `worker`, script run in a lend's worker before its own: the
// browser alone fetches lend.js, as it registers and updates the worker,
// so the pages and what they import never see it (u4-10's plant, u5-2's
// worker).
const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
};
//
// What it was asked is in `server.asked`, each `<time> <method> <host><path>
// <status>`: a lend's request that reaches the page's host is one the
// lend's worker did not answer (u5-12).
//
// Each answer carries the headers the page host is told to send
// (web/vercel.json, sup-3), each `source` read as a pattern of the path:
// its `frame-ancestors` is the one guard a `<meta>` policy cannot be. All
// but HSTS: under it the tests' certificate, which no browser trusts, is
// refused past any telling, and Playwright's WebKit keeps it for every
// profile of this user (~/Library/Caches/org.webkit.Playwright/WebKit/HSTS,
// 2026-11-02: an app's name refused "A TLS error" ever after one visit).
const HSTS = 'Strict-Transport-Security';
const HOST = JSON.parse(await readFile(new URL('../vercel.json', import.meta.url), 'utf8')).headers;
//
// The page's policy is given the tests' own allowance, and no other: their
// relay, plain HTTP on 127.0.0.1 (crates/svrngs-node/tests/browser/harness.rs),
// which the published page never holds (web/index.html), asked over TLS:
// WebKit refuses a TLS page a plain request to 127.0.0.1, as mixed content,
// so the page and the lend's worker upgrade theirs
// (`upgrade-insecure-requests`), and the relay's front ends the TLS
// (harness.rs `relay`); Chromium asks 127.0.0.1 in the clear, as before.
// Only at the page's TLS name (`PAGE_HOST`): a page a test opens on
// 127.0.0.1 in the clear upgrades nothing, or its own files would be asked
// over TLS of a server that speaks none.
// A page whose `connect-src` no longer reads as below stops the tests,
// said, rather than serving it without the allowance.
const CONNECT = "connect-src 'self' wss: https:;";
const LOOPBACK = 'ws://127.0.0.1:* http://127.0.0.1:*';
function tests(page, upgraded) {
  if (!String(page).includes(CONNECT)) {
    console.error(
      `the page's policy holds no \`${CONNECT}\`, so page.mjs cannot add the tests' relay to it`,
    );
    process.exit(1);
  }
  return String(page).replace(
    CONNECT,
    `${CONNECT.slice(0, -1)} ${LOOPBACK};${upgraded ? ' upgrade-insecure-requests;' : ''}`,
  );
}
const UPGRADED = { 'content-security-policy': 'upgrade-insecure-requests' };

// The page at the shape production's has, over TLS at a name with each
// app at a name of its own under it (DESIGN §3, "the page's host must serve
// the page at every name under it"): `https://group.svrnme.test:<port>/`,
// an app `https://<label>.group.svrnme.test:<port>/`. On `localhost` and
// `<label>.localhost` WebKit gives the group's signer, framed under an app,
// none of the page's cookies, and the signer's handoff is one
// (HOLDER_APPROVAL §3.D; demo/runs/security/2026-11-02-webkit-partition/,
// holder-design/run-localhost*.txt). No name resolves: each context reaches
// them through `proxied`, which ends their TLS and hands each to the page's
// server, plain HTTP on 127.0.0.1 at the name's port.
export const PAGE_HOST = 'group.svrnme.test';
export const pageAt = (server) => `https://${PAGE_HOST}:${server.address().port}/`;
export const appAt = (label, port) => `https://${label}.${PAGE_HOST}:${port}`;
export const inApp = (url) => url.hostname.endsWith(`.${PAGE_HOST}`);
const TLS = new URL('../../target/tls/', import.meta.url).pathname;

// The certificate the proxy ends TLS with, made once a checkout by openssl,
// which macOS and every Linux carry, in a directory of its own put in
// place whole, so two drivers making it at once each read one pair: the
// browsers take it as they are told to (`launch`), so it need name nothing
// they check. Exported for dev.mjs, which names it to Chrome by its key.
export function certified() {
  try {
    return { key: readFileSync(`${TLS}page/key.pem`), cert: readFileSync(`${TLS}page/cert.pem`) };
  } catch {
    mkdirSync(TLS, { recursive: true });
    const made = mkdtempSync(`${TLS}making-`);
    try {
      execFileSync(
        'openssl',
        [
          'req',
          '-x509',
          '-newkey',
          'ec',
          '-pkeyopt',
          'ec_paramgen_curve:P-256',
          '-nodes',
          '-days',
          '3650',
          '-subj',
          `/CN=${PAGE_HOST}`,
          '-keyout',
          `${made}/key.pem`,
          '-out',
          `${made}/cert.pem`,
          '-addext',
          `subjectAltName=DNS:${PAGE_HOST},DNS:*.${PAGE_HOST}`,
        ],
        { stdio: 'pipe' },
      );
    } catch (e) {
      console.error(
        `openssl could not make the tests' page certificate, so no page can be tested over TLS: ${e}`,
      );
      process.exit(3);
    }
    try {
      renameSync(made, `${TLS}page`);
    } catch {
      rmSync(made, { recursive: true, force: true });
    }
    return certified();
  }
}

let secure;
const secured = () => (secure ??= tls.createSecureContext(certified()));

// The proxy each context reaches the page's names through: the run's, which
// browsers.mjs starts beside its browsers, else one of this process's own;
// one a run, since WebKit opened contexts at once under proxies of several
// processes and some found no proxy (2026-11-02, the pages step). A CONNECT
// to a name under PAGE_HOST is answered here, its TLS ended, with one
// context made once, and its bytes handed to 127.0.0.1 at the name's port.
// (Ending it in each page's server instead, after reading a byte to tell
// TLS from plain, timed out every page load, 2026-11-02.) Anything else goes where it was
// asked, as with no proxy: a TV at the room's address on this WiFi (the
// night scene); what a test keeps off this computer, `offline` stops in
// the browser before it gets here.
let proxying;
export const proxied = () =>
  (proxying ??= new Promise((done) => {
    const piped = (a, b) => (
      a.pipe(b).pipe(a),
      [a, b].forEach((s) => s.on('error', () => (a.destroy(), b.destroy())))
    );
    const proxy = http.createServer((req, res) => {
      const to = new URL(req.url);
      const sent = http.request(
        {
          host: to.hostname,
          port: to.port || 80,
          path: `${to.pathname}${to.search}`,
          method: req.method,
          headers: req.headers,
        },
        (r) => (res.writeHead(r.statusCode, r.headers), r.pipe(res)),
      );
      sent.on('error', () => res.writeHead(502).end());
      req.pipe(sent);
    });
    proxy.on('connect', (req, sock, head) => {
      const [, name, port] = /^(.+):(\d+)$/.exec(req.url) ?? [];
      const ours = name === PAGE_HOST || name?.endsWith(`.${PAGE_HOST}`);
      sock.write('HTTP/1.1 200 Connection Established\r\n\r\n');
      if (head?.length) sock.unshift(head);
      if (!ours) return piped(sock, net.connect(+port, name));
      sock.on('error', () => sock.destroy());
      piped(
        new tls.TLSSocket(sock, { isServer: true, secureContext: secured() }),
        net.connect(+port, '127.0.0.1'),
      );
    });
    proxy.listen(
      0,
      '127.0.0.1',
      () => (proxy.unref(), done(`http://127.0.0.1:${proxy.address().port}`)),
    );
  }));
// What every context is opened with: the page's names through the proxy,
// its certificate taken, loopback asked directly. Chromium takes it for a
// worker's script only when told at launch (`IGNORING`).
const shared = () => JSON.parse(process.env.SVRNGS_BROWSERS ?? '{}').proxy;
export const reaching = async () => ({
  ignoreHTTPSErrors: true,
  proxy: { server: shared() ?? (await proxied()), bypass: 'localhost,*.localhost,127.0.0.1' },
});
export const IGNORING = ['--ignore-certificate-errors'];
// How Playwright starts a browser of `type` here, for browsers.mjs and
// `launch`. WebKit on macOS starts through webkit.sh, with AppKit's window
// animations off: each page's animation kept one of 64 threads while the
// display was off, and from the 64th page on, the browser loaded nothing
// (demo/runs/webkit-wall/2026-11-03). Anything else starts as Playwright
// starts it.
export function starting(type) {
  if (type.name() !== 'webkit' || process.platform !== 'darwin') return {};
  return {
    executablePath: fileURLToPath(new URL('./webkit.sh', import.meta.url)),
    env: { ...process.env, SVRNGS_WEBKIT: type.executablePath() },
  };
}
export function serve(dir, { worker, port = 0, policy } = {}) {
  const root = resolve(dir);
  const asked = [];
  const server = http.createServer(async (req, res) => {
    const path = new URL(req.url, 'http://page').pathname;
    res.on('finish', () =>
      asked.push(
        `${new Date().toISOString()} ${req.method} ${req.headers.host}${req.url} ${res.statusCode}`,
      ),
    );
    for (const { source, headers } of HOST) {
      if (new RegExp(`^${source}$`).test(path))
        for (const { key, value } of headers) if (key !== HSTS) res.setHeader(key, value);
    }
    const file = join(root, normalize(path.endsWith('/') ? `${path}index.html` : path));
    if (!file.startsWith(root)) return res.writeHead(403).end();
    // A production-policy page keeps the built meta as `scripts/build-web.sh`
    // writes it, so the tests can prove what the published page does with an
    // HTTP route: `tests()`' allowance is not added, and nothing upgrades.
    const upgraded =
      policy !== 'production' && new URL(`http://${req.headers.host}`).hostname.endsWith(PAGE_HOST);
    try {
      let body = await readFile(file);
      if (/\/(index|404)\.html$/.test(file) && policy !== 'production')
        body = tests(body, upgraded);
      if (worker && path === '/lend.js') body = `${worker}\n${body}`;
      res.writeHead(200, {
        'content-type': TYPES[extname(file)] ?? 'application/octet-stream',
        ...(upgraded && path === '/lend.js' ? UPGRADED : {}),
      });
      res.end(body);
    } catch {
      // A path with no file: the site's 404.html, as a static host serves
      // it (Vercel, Netlify, Cloudflare Pages, GitHub Pages), else nothing.
      const missing = await readFile(join(root, '404.html')).catch(() => null);
      res
        .writeHead(404, { 'content-type': TYPES['.html'] })
        .end(missing && policy !== 'production' ? tests(missing, upgraded) : missing || '');
    }
  });
  server.asked = asked;
  return new Promise((done) => server.listen(port, '127.0.0.1', () => done(server)));
}

// The engines every browser test runs in (u4-7): Chromium as a desktop
// opens a page, and WebKit as an iPhone does — Playwright's iPhone 15,
// its screen, touch and Safari's user agent — the engine iOS Safari is.
const ENGINES = { chromium: null, webkit: 'iPhone 15' };

// The hosts svrngs opened for this half (drive.rs `opens`): each
// computer's home page and each lend its args name, opened in `context` by
// a code from `/v1/open`, asked with the token as `svrngs open` asks it, and
// redeemed by a real bootstrap page, which imports the origin key and waits
// for its authentication worker to control it. No cookie models authority.
// A code is single-use, so each context mints its own; one
// refused is said, and the host left unopened.
async function opened(context) {
  for (const { port, token, names } of JSON.parse(process.env.SVRNGS_OPENS ?? '[]')) {
    for (const name of names) {
      const body = JSON.stringify({
        ...(name === 'svrngs' ? {} : { lend: name }),
        at: '/svrngs-session-ready',
      });
      const headers = { authorization: `Bearer ${token}`, 'content-type': 'application/json' };
      const asked = await fetch(`http://127.0.0.1:${port}/v1/open`, {
        method: 'POST',
        headers,
        body,
      }).catch((e) => e);
      if (!asked.ok) {
        console.error(`svrngs at ${port} did not open ${name}: ${asked.status ?? asked}`);
        continue;
      }
      // The ready endpoint is public bootstrap infrastructure, not the service.
      const at = (await asked.json()).open;
      const enrollment = await context.newPage();
      await enrollment.goto(at);
      await enrollment.waitForURL('**/svrngs-session-ready', { timeout: 30000 });
      await enrollment.close();
    }
  }
  return context;
}

// Layout A's seam, gone with A (home-reading.js `first`): each load of the
// home page, a redirect or a reload too, at `/?ui=a` before its scripts read
// the address, so a driver written for one long page stays on it.
function longPage() {
  const query = new URLSearchParams(location.search);
  if (location.hostname !== 'svrngs.localhost' || location.pathname !== '/' || query.has('ui'))
    return;
  query.set('ui', 'a');
  history.replaceState(null, '', `/?${query}${location.hash}`);
}

// A browser of `engine` and a context in it, as a phone or a desktop
// opens the page, and how to close it. With `profile`, a directory, the
// context keeps what the page stores there, so a browser closed and
// launched again on it finds the page's store and worker as a phone's
// does when it opens the browser again. With `device`, one of
// Playwright's by name, the engine is that device in place of its own
// (Demo B's Android phones: Chromium as a Pixel 7). With `args`, the
// browser's own flags (arch-peer (a): Chromium told a page is public).
// `unopened`: a context svrngs opened no host in, as a browser no open
// reached (give-session). With `b`, the home page as a person now has it; without
// it, a driver that reads one long page: `longPage`, the sunset's worklist.
export async function launch(engine, { profile, device: as, args, unopened, b } = {}) {
  if (!(engine in ENGINES)) {
    console.error(`no engine ${JSON.stringify(engine)}: one of ${Object.keys(ENGINES).join(', ')}`);
    process.exit(1);
  }
  let pw;
  try {
    pw = await import('playwright');
  } catch (e) {
    console.error(`Playwright is not installed — run: ${INSTALL}\n${e}`);
    process.exit(3);
  }
  const named = as ?? ENGINES[engine];
  const device = named ? pw.devices[named] : {};
  let made;
  try {
    const flags = engine === 'chromium' ? [...IGNORING, ...(args ?? [])] : args;
    if (profile) {
      const context = await pw[engine].launchPersistentContext(profile, {
        ...device,
        ...(await reaching()),
        ...starting(pw[engine]),
        args: flags,
      });
      made = { context, close: () => context.close() };
    } else {
      // The run's browser of `engine`, where its harness started one
      // (browsers.mjs), and a context of its own in it: a browser a test
      // crashed macOS's pasteboard daemon (harness.rs `browsers`). Closing
      // it then closes this context and lets the shared browser be.
      const shared = !args && JSON.parse(process.env.SVRNGS_BROWSERS ?? '{}')[engine];
      const browser = await (shared
        ? pw[engine].connect(shared)
        : pw[engine].launch({ ...starting(pw[engine]), args: flags }));
      made = {
        context: await browser.newContext({ ...device, ...(await reaching()) }),
        close: () => browser.close(),
      };
    }
  } catch (e) {
    console.error(`Playwright's ${engine} cannot start — run: ${INSTALL}\n${e}`);
    process.exit(3);
  }
  made.context.pages().forEach(landing);
  made.context.on('page', landing);
  if (!b) await made.context.addInitScript(longPage);
  if (!unopened) await opened(made.context);
  return made;
}

// Where `page` lands once an open it passed through has done its work: an
// open's address (`/svrngs-open?code=`, from `svrngs open`, the home page's
// `/to`, or a lend's bookmark sent there) answers only the bootstrap page,
// whose script redeems the code, keeps the origin's key, waits for its
// worker and only then goes on to where the open leads (home/session.js),
// as a person's browser goes on a moment after it loads. Loaded there, or
// at once when it never was at an open.
// Answered: the response its last whole-tab navigation got, the page it
// arrived at's own, where `page.goto`'s is the bootstrap's (each page of
// a context `launch` made, `landing`).
export async function arrived(page, timeout = 30_000) {
  await page.waitForURL((u) => u.pathname !== '/svrngs-open', { timeout });
  return landed.get(page) ?? null;
}
// Each page's last response to a navigation of its own tab, a redirect's
// included, kept as it comes.
const landed = new WeakMap();
function landing(page) {
  page.on(
    'response',
    (r) =>
      r.request().isNavigationRequest() && r.frame() === page.mainFrame() && landed.set(page, r),
  );
}

// `page`'s origin's session lapsed, as one does: the key svrngs's open
// enrolled there (home/session.js, IndexedDB `svrngs-browser`) dropped, so
// its worker proves no request after and each is refused as a browser
// with no session is (session.rs `refused`). What clearing the cookie was
// before the key moved out of one.
export async function lapse(page) {
  await page.evaluate(
    () =>
      new Promise((done, failed) => {
        const asked = indexedDB.deleteDatabase('svrngs-browser');
        asked.onsuccess = done;
        asked.onerror = () => failed(asked.error);
        asked.onblocked = () => failed(new Error('the session key is held open'));
      }),
  );
}

// Every address off this computer refused, as with n0 unreachable
// (relay-default (b)): each request and socket the context opens elsewhere —
// n0's lookup, n0's relays, any host — aborted, and its URL kept in the
// list answered. Ours run on 127.0.0.1, names under localhost and the page's
// (`PAGE_HOST`).
// "What acts as you" opened by its heading, as a person opens it: on both
// pages it is folded under it (question.js `acts`). Once open, left so.
export async function unfold(page) {
  const acts = page.locator('#acts > details');
  if (!(await acts.evaluate((d) => d.open, null, { timeout: 90_000 })))
    await acts.locator('> summary').click({ timeout: 10_000 });
}

// What a stranger takes in at one look of the screen as it stands, up to
// `room` words, in the order xenia's stage reads it: things to tap, then
// headings, then large text, then the rest, each in the document's order,
// a box's label with whether it is ticked (xenia src/stage.ts `SCAN` and
// src/core/world.ts `reading`, at 4f12ba4; contrast not judged). Each
// person's room is their budget, demo/xenia/people/*.md: the words a
// look read before acting, which the fifth set counted past each budget
// (demo/xenia/findings/wordy-*.md), checked here with no model run.
export function firstLook(page, room) {
  return page.evaluate((room) => {
    const TAP =
      'a[href],button,input,label,select,textarea,summary,[role=button],[role=link],[onclick]';
    const [pieces, walk, range] = [
      new Map(),
      document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT),
      document.createRange(),
    ];
    const shown = (n, m) => (
      range.setStart(n, m.index),
      range.setEnd(n, m.index + m[0].length),
      ((r) =>
        r.width > 0 && r.bottom > 0 && r.top < innerHeight && r.right > 0 && r.left < innerWidth)(
        range.getBoundingClientRect(),
      )
    );
    for (let n = walk.nextNode(); n; n = walk.nextNode()) {
      const el = n.parentElement;
      if (
        !el ||
        ['SCRIPT', 'STYLE', 'NOSCRIPT'].includes(el.tagName) ||
        !el.checkVisibility({ visibilityProperty: true, opacityProperty: true })
      )
        continue;
      const words = [...n.data.matchAll(/\S+/g)].filter((m) => shown(n, m)).map((m) => m[0]);
      const at = el.closest(TAP) ?? n;
      if (words.length)
        pieces.set(at, {
          el: el.closest(TAP) ?? el,
          words: [...(pieces.get(at)?.words ?? []), ...words],
        });
    }
    for (const box of document.querySelectorAll('input[type=checkbox]'))
      pieces.get(box.labels?.[0])?.words.push(box.checked ? '(ticked)' : '(not ticked)');
    const rank = ({ el }, s = getComputedStyle(el), px = parseFloat(s.fontSize)) =>
      el.matches(TAP)
        ? 0
        : el.closest('h1,h2,h3,h4,h5,h6,[role=heading]')
          ? 1
          : px >= 24 || (Number(s.fontWeight) >= 600 && px >= 18.66)
            ? 2
            : 3;
    return [...pieces.values()]
      .sort((a, b) => rank(a) - rank(b))
      .flatMap((p) => p.words)
      .slice(0, room)
      .join(' ');
  }, room);
}

export async function offline(context) {
  const away = [];
  const off = (url) =>
    /^(https?|wss?):$/.test(url.protocol) &&
    !/^(127\.0\.0\.1|\[::1\]|(.+\.)?localhost|(.+\.)?group\.svrnme\.test)$/.test(url.hostname);
  await context.route(off, (route) => (away.push(route.request().url()), route.abort()));
  await context.routeWebSocket(off, (ws) => (away.push(ws.url()), ws.close()));
  return away;
}

// In the page: its store holding a group joined by `token`, as joining
// keeps one (web/store.js, u4-5): the person's key sealed, the group's
// name, and the person's — a phone that joined; with `maker` and `since`,
// who made the group and when the phone joined it (v1ui-door (d)); with
// `id`, the group's own, as a phone that joined keeps it (v1ui-shelf (c));
// `name` the group's, house unless given. Imported as app.js imports it.
export async function hold({
  hex,
  token,
  person = 'bo',
  maker,
  since,
  id = 'house',
  name = 'house',
}) {
  const app = document.querySelector('script[type=module]').src;
  const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
  const key = Uint8Array.from(hex.match(/../g), (h) => parseInt(h, 16));
  await store.keep({ id, name, person, token, key, maker, since });
}

// The group's page at `origin`, opened by the person `args.key` names in
// the group `args.token` joins, their store holding it (`hold`); its list
// said, or its refusal.
export async function held(page, origin, args) {
  await page.goto(origin);
  await page.evaluate(hold, { hex: args.key, token: args.token, id: args.id });
  await page.goto('about:blank');
  await page.goto(`${origin}#${args.token}`);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
}

// A driver of the group's page, its preamble written once (#168): the
// page served from `args.site` (at `args.port` when given), `args.engine`
// launched, each console line and page error kept, and `drive({ page,
// context, origin, logs })` run on a page holding the group `args` names; what it
// answers printed as the JSON line with the log, or the page's failure and
// its log, exit 1. Closed after, either way.
export async function driving(args, drive) {
  const server = await serve(args.site, { port: args.port });
  const origin = pageAt(server);
  const { context, close } = await launch(args.engine);
  const logs = [];
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    page.on('requestfailed', (r) =>
      logs.push(`requestfailed: ${r.url()} ${r.failure()?.errorText}`),
    );
    await held(page, origin, args);
    console.log(JSON.stringify({ ...(await drive({ page, context, origin, logs })), logs }));
  } catch (e) {
    console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
    process.exitCode = 1;
  } finally {
    await close();
    server.close();
  }
}

// In the lend's page: `range` of `path` at the page's own origin, which
// the lend's worker serves, read whole — its status, type, content-range,
// length and SHA-256 — and the `<video>`'s source as the page resolved it.
// The bytes a lent film's page is answered, which every test but the
// playback step's asserts in place of the browser decoding them
// (crates/svrngs-node/tests/browser/playback.rs; demo/runs/flakes/2026-11-12).
export async function fetched({ path, range }) {
  const src = document.querySelector('video')?.currentSrc || document.querySelector('video')?.src;
  try {
    // A bound on a worker that never answers, so it is said, not hung on.
    const signal = AbortSignal.timeout(60_000);
    const res = await fetch(new URL(path, location.href), { headers: { range }, signal });
    const bytes = new Uint8Array(await res.arrayBuffer());
    const sha = [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))]
      .map((x) => x.toString(16).padStart(2, '0'))
      .join('');
    return {
      src: src ?? null,
      status: res.status,
      type: res.headers.get('content-type'),
      contentRange: res.headers.get('content-range'),
      bytes: bytes.length,
      sha,
    };
  } catch (e) {
    return { src: src ?? null, error: String(e) };
  }
}

// In the lend's page: play its `<video>`, or seek it to `at`, and answer
// how long until it plays on from there — `ms`, or `null` past `within` —
// with where it stood then, or what the page says when it holds no video.
export async function play({ at, within }) {
  const v = document.querySelector('video');
  if (!v)
    return {
      ms: null,
      error: `no <video> on the page, which says: ${document.body.innerText.slice(0, 200)}`,
    };
  const started = performance.now();
  if (at === null) {
    v.muted = true;
    v.play().catch(() => {});
  } else {
    v.currentTime = at;
  }
  const from = at ?? 0;
  return new Promise((done) => {
    const check = () => {
      const t = performance.now() - started;
      if (!v.seeking && !v.paused && v.currentTime > from + 0.25) {
        done({ ms: Math.round(t), at: v.currentTime });
      } else if (t > within) {
        done({
          ms: null,
          at: v.currentTime,
          seeking: v.seeking,
          paused: v.paused,
          error: v.error?.message ?? null,
        });
      } else {
        setTimeout(check, 20);
      }
    };
    check();
  });
}
