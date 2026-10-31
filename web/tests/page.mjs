// What the browser halves share (drive.mjs, join.mjs, runtime.mjs): the
// built page served as an origin serves it, Playwright's headless Chromium
// or WebKit as an iPhone, and a joined group held in the page's store.
// Each exits 3 naming the install command when Playwright or its browser
// is absent — the browser step fails then, never skips.
import http from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize, resolve } from 'node:path';

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
// its `frame-ancestors` is the one guard a `<meta>` policy cannot be.
const HOST = JSON.parse(await readFile(new URL('../vercel.json', import.meta.url), 'utf8')).headers;
//
// The page's policy is given the tests' own allowance, and no other: their
// relay, plain HTTP on 127.0.0.1 (crates/svrngs-node/tests/browser/harness.rs),
// which the published page never holds (web/index.html). A page whose
// `connect-src` no longer reads as below stops the tests, said, rather
// than serving it without the allowance.
const CONNECT = "connect-src 'self' wss: https:;";
const LOOPBACK = 'ws://127.0.0.1:* http://127.0.0.1:*';
function tests(page) {
  if (!String(page).includes(CONNECT)) {
    console.error(`the page's policy holds no \`${CONNECT}\`, so page.mjs cannot add the tests' relay to it`);
    process.exit(1);
  }
  return String(page).replace(CONNECT, `${CONNECT.slice(0, -1)} ${LOOPBACK};`);
}
export function serve(dir, { worker, port = 0 } = {}) {
  const root = resolve(dir);
  const asked = [];
  const server = http.createServer(async (req, res) => {
    const path = new URL(req.url, 'http://page').pathname;
    res.on('finish', () => asked.push(`${new Date().toISOString()} ${req.method} ${req.headers.host}${req.url} ${res.statusCode}`));
    for (const { source, headers } of HOST) {
      if (new RegExp(`^${source}$`).test(path)) for (const { key, value } of headers) res.setHeader(key, value);
    }
    const file = join(root, normalize(path.endsWith('/') ? `${path}index.html` : path));
    if (!file.startsWith(root)) return res.writeHead(403).end();
    try {
      let body = await readFile(file);
      if (/\/(index|404)\.html$/.test(file)) body = tests(body);
      if (worker && path === '/lend.js') body = `${worker}\n${body}`;
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
      res.end(body);
    } catch {
      // A path with no file: the site's 404.html, as a static host serves
      // it (Vercel, Netlify, Cloudflare Pages, GitHub Pages), else nothing.
      const missing = await readFile(join(root, '404.html')).catch(() => null);
      res.writeHead(404, { 'content-type': TYPES['.html'] }).end(missing ? tests(missing) : '');
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
// redeemed by the context's own requests, whose cookies its pages share, so
// the context holds each host's session as a person's browser does
// (give-session). A code is single-use, so each context mints its own; one
// refused is said, and the host left unopened.
async function opened(context) {
  for (const { port, token, names } of JSON.parse(process.env.SVRNGS_OPENS ?? '[]')) {
    for (const name of names) {
      const body = JSON.stringify(name === 'svrngs' ? {} : { lend: name });
      const headers = { authorization: `Bearer ${token}`, 'content-type': 'application/json' };
      const asked = await fetch(`http://127.0.0.1:${port}/v1/open`, { method: 'POST', headers, body }).catch((e) => e);
      if (!asked.ok) {
        console.error(`svrngs at ${port} did not open ${name}: ${asked.status ?? asked}`);
        continue;
      }
      // Redeemed and not followed, so no lend's service is asked by it.
      const at = (await asked.json()).open;
      const redeemed = await context.request.get(at, { maxRedirects: 0 });
      if (!redeemed.headers()['set-cookie']) console.error(`svrngs at ${port} set no session at ${at}: ${redeemed.status()}`);
    }
  }
  return context;
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
// reached (give-session). With `zone`, an IANA time zone, the browser's
// clock reads in it, as a browser elsewhere does (Playwright's
// `timezoneId`).
export async function launch(engine, { profile, device: as, args, unopened, zone } = {}) {
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
  const device = { ...(named ? pw.devices[named] : {}), ...(zone ? { timezoneId: zone } : {}) };
  let made;
  try {
    if (profile) {
      const context = await pw[engine].launchPersistentContext(profile, device);
      made = { context, close: () => context.close() };
    } else {
      // The run's browser of `engine`, where its harness started one
      // (browsers.mjs), and a context of its own in it: a browser a test
      // crashed macOS's pasteboard daemon (harness.rs `browsers`). Closing
      // it then closes this context and lets the shared browser be.
      const shared = !args && JSON.parse(process.env.SVRNGS_BROWSERS ?? '{}')[engine];
      const browser = await (shared ? pw[engine].connect(shared) : pw[engine].launch({ args }));
      made = { context: await browser.newContext(device), close: () => browser.close() };
    }
  } catch (e) {
    console.error(`Playwright's ${engine} cannot start — run: ${INSTALL}\n${e}`);
    process.exit(3);
  }
  if (!unopened) await opened(made.context);
  return made;
}

// Every address off this computer refused, as with n0 unreachable
// (relay-default (b)): each request and socket the context opens elsewhere —
// n0's lookup, n0's relays, any host — aborted, and its URL kept in the
// list answered. Ours run on 127.0.0.1 and names under localhost.
// "What acts as you" opened by its heading, as a person opens it: on both
// pages it is folded under it (question.js `acts`). Once open, left so.
export async function unfold(page) {
  const acts = page.locator('#acts > details');
  if (!(await acts.evaluate((d) => d.open, null, { timeout: 90_000 }))) await acts.locator('> summary').click({ timeout: 10_000 });
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
    const TAP = 'a[href],button,input,label,select,textarea,summary,[role=button],[role=link],[onclick]';
    const [pieces, walk, range] = [new Map(), document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT), document.createRange()];
    const shown = (n, m) => (range.setStart(n, m.index), range.setEnd(n, m.index + m[0].length), ((r) => r.width > 0 && r.bottom > 0 && r.top < innerHeight && r.right > 0 && r.left < innerWidth)(range.getBoundingClientRect()));
    for (let n = walk.nextNode(); n; n = walk.nextNode()) {
      const el = n.parentElement;
      if (!el || ['SCRIPT', 'STYLE', 'NOSCRIPT'].includes(el.tagName) || !el.checkVisibility({ visibilityProperty: true, opacityProperty: true })) continue;
      const words = [...n.data.matchAll(/\S+/g)].filter((m) => shown(n, m)).map((m) => m[0]);
      const at = el.closest(TAP) ?? n;
      if (words.length) pieces.set(at, { el: el.closest(TAP) ?? el, words: [...(pieces.get(at)?.words ?? []), ...words] });
    }
    for (const box of document.querySelectorAll('input[type=checkbox]')) pieces.get(box.labels?.[0])?.words.push(box.checked ? '(ticked)' : '(not ticked)');
    const rank = ({ el }, s = getComputedStyle(el), px = parseFloat(s.fontSize)) => (el.matches(TAP) ? 0 : el.closest('h1,h2,h3,h4,h5,h6,[role=heading]') ? 1 : px >= 24 || (Number(s.fontWeight) >= 600 && px >= 18.66) ? 2 : 3);
    return [...pieces.values()].sort((a, b) => rank(a) - rank(b)).flatMap((p) => p.words).slice(0, room).join(' ');
  }, room);
}

export async function offline(context) {
  const away = [];
  const off = (url) => /^(https?|wss?):$/.test(url.protocol) && !/^(127\.0\.0\.1|\[::1\]|(.+\.)?localhost)$/.test(url.hostname);
  await context.route(off, (route) => (away.push(route.request().url()), route.abort()));
  await context.routeWebSocket(off, (ws) => (away.push(ws.url()), ws.close()));
  return away;
}

// In the page: its store holding a group joined by `token`, as joining
// keeps one (web/store.js, u4-5): the person's key sealed, the group's
// name, and the person's — a phone that joined; with `maker` and `since`,
// who made the group and when the phone joined it (v1ui-door (d)); with
// `id`, the group's own, as a phone that joined keeps it (v1ui-shelf (c)).
// Imported as app.js imports it.
export async function hold({ hex, token, person = 'bo', maker, since, id = 'held' }) {
  const app = document.querySelector('script[type=module]').src;
  const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
  const key = Uint8Array.from(hex.match(/../g), (h) => parseInt(h, 16));
  await store.keep({ id, name: 'house', person, token, key, maker, since });
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
// launched, in `args.zone` when given, each console line and page error kept, and `drive({ page,
// context, origin, logs })` run on a page holding the group `args` names; what it
// answers printed as the JSON line with the log, or the page's failure and
// its log, exit 1. Closed after, either way.
export async function driving(args, drive) {
  const server = await serve(args.site, { port: args.port });
  const origin = `http://localhost:${server.address().port}/`;
  const { context, close } = await launch(args.engine, { zone: args.zone });
  const logs = [];
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
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

// In the lend's page: play its `<video>`, or seek it to `at`, and answer
// how long until it plays on from there — `ms`, or `null` past `within` —
// with where it stood then, or what the page says when it holds no video.
export async function play({ at, within }) {
  const v = document.querySelector('video');
  if (!v) return { ms: null, error: `no <video> on the page, which says: ${document.body.innerText.slice(0, 200)}` };
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
        done({ ms: null, at: v.currentTime, seeking: v.seeking, paused: v.paused, error: v.error?.message ?? null });
      } else {
        setTimeout(check, 20);
      }
    };
    check();
  });
}
