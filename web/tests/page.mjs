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
// trailing slash, and the wasm by its type so the browser streams it.
const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
};
export function serve(dir) {
  const root = resolve(dir);
  const server = http.createServer(async (req, res) => {
    const path = new URL(req.url, 'http://page').pathname;
    const file = join(root, normalize(path.endsWith('/') ? `${path}index.html` : path));
    if (!file.startsWith(root)) return res.writeHead(403).end();
    try {
      const body = await readFile(file);
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
      res.end(body);
    } catch {
      res.writeHead(404).end();
    }
  });
  return new Promise((done) => server.listen(0, '127.0.0.1', () => done(server)));
}

// The engines every browser test runs in (u4-7): Chromium as a desktop
// opens a page, and WebKit as an iPhone does — Playwright's iPhone 15,
// its screen, touch and Safari's user agent — the engine iOS Safari is.
const ENGINES = { chromium: null, webkit: 'iPhone 15' };

// A browser of `engine` and a context in it, as a phone or a desktop
// opens the page, and how to close it. With `profile`, a directory, the
// context keeps what the page stores there, so a browser closed and
// launched again on it finds the page's store and worker as a phone's
// does when it opens the browser again.
export async function launch(engine, { profile } = {}) {
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
  const device = ENGINES[engine] ? pw.devices[ENGINES[engine]] : {};
  try {
    if (profile) {
      const context = await pw[engine].launchPersistentContext(profile, device);
      return { context, close: () => context.close() };
    }
    const browser = await pw[engine].launch();
    return { context: await browser.newContext(device), close: () => browser.close() };
  } catch (e) {
    console.error(`Playwright's ${engine} cannot start — run: ${INSTALL}\n${e}`);
    process.exit(3);
  }
}

// In the page: its store holding a group joined by `token`, as joining
// keeps one (web/store.js, u4-5): the person's key sealed, the group's
// name, and the person's — a phone that joined. Imported as app.js
// imports it.
export async function hold({ hex, token }) {
  const app = document.querySelector('script[type=module]').src;
  const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
  const key = Uint8Array.from(hex.match(/../g), (h) => parseInt(h, 16));
  await store.keep({ id: 'held', name: 'house', person: 'bo', token, key });
}
