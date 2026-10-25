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
      if (file.endsWith('/index.html')) body = tests(body);
      if (worker && path === '/lend.js') body = `${worker}\n${body}`;
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
      res.end(body);
    } catch {
      res.writeHead(404).end();
    }
  });
  server.asked = asked;
  return new Promise((done) => server.listen(port, '127.0.0.1', () => done(server)));
}

// The engines every browser test runs in (u4-7): Chromium as a desktop
// opens a page, and WebKit as an iPhone does — Playwright's iPhone 15,
// its screen, touch and Safari's user agent — the engine iOS Safari is.
const ENGINES = { chromium: null, webkit: 'iPhone 15' };

// A browser of `engine` and a context in it, as a phone or a desktop
// opens the page, and how to close it. With `profile`, a directory, the
// context keeps what the page stores there, so a browser closed and
// launched again on it finds the page's store and worker as a phone's
// does when it opens the browser again. With `device`, one of
// Playwright's by name, the engine is that device in place of its own
// (Demo B's Android phones: Chromium as a Pixel 7). With `args`, the
// browser's own flags (arch-peer (a): Chromium told a page is public).
export async function launch(engine, { profile, device: as, args } = {}) {
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
  try {
    if (profile) {
      const context = await pw[engine].launchPersistentContext(profile, device);
      return { context, close: () => context.close() };
    }
    // The run's browser of `engine`, where its harness started one
    // (browsers.mjs), and a context of its own in it: a browser a test
    // crashed macOS's pasteboard daemon (harness.rs `browsers`). Closing
    // it then closes this context and lets the shared browser be.
    const shared = !args && JSON.parse(process.env.SVRNGS_BROWSERS ?? '{}')[engine];
    const browser = await (shared ? pw[engine].connect(shared) : pw[engine].launch({ args }));
    return { context: await browser.newContext(device), close: () => browser.close() };
  } catch (e) {
    console.error(`Playwright's ${engine} cannot start — run: ${INSTALL}\n${e}`);
    process.exit(3);
  }
}

// Every address off this computer refused, as with n0 unreachable
// (relay-default (b)): each request and socket the context opens elsewhere —
// n0's lookup, n0's relays, any host — aborted, and its URL kept in the
// list answered. Ours run on 127.0.0.1 and names under localhost.
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
// launched, each console line and page error kept, and `drive({ page,
// context, origin, logs })` run on a page holding the group `args` names; what it
// answers printed as the JSON line with the log, or the page's failure and
// its log, exit 1. Closed after, either way.
export async function driving(args, drive) {
  const server = await serve(args.site, { port: args.port });
  const origin = `http://localhost:${server.address().port}/`;
  const { context, close } = await launch(args.engine);
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
