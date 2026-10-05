// The browser half of crates/svrngs-node/tests/browser (DESIGN §8, U4): it
// serves the built page on 127.0.0.1, opens it in Playwright's headless
// Chromium, and prints what the page did as one JSON line. The Rust half
// owns the relay and the node, so every byte stays on this computer.
//
//   node web/tests/drive.mjs '{"site": "<dir>", "key": "<64 hex>", "token": "<token>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its Chromium is absent — the browser step fails then, never skips; 1 on
// anything else.
import http from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize, resolve } from 'node:path';

const INSTALL = 'npm ci --prefix web && npx --prefix web playwright install chromium';

const args = JSON.parse(process.argv[2]);

let chromium;
try {
  ({ chromium } = await import('playwright'));
} catch (e) {
  console.error(`Playwright is not installed — run: ${INSTALL}\n${e}`);
  process.exit(3);
}

// The page as an origin serves it: its directory at the root, with the
// trailing slash, and the wasm by its type so the browser streams it.
const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
};
function serve(dir) {
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

// The key into the page's store as app.js reads it (IndexedDB `svrngs`,
// object store `person`, entry `key`): the page holds the person's key.
function hold(hex) {
  return new Promise((done, fail) => {
    const open = indexedDB.open('svrngs', 1);
    open.onupgradeneeded = () => open.result.createObjectStore('person');
    open.onerror = () => fail(open.error);
    open.onsuccess = () => {
      const tx = open.result.transaction('person', 'readwrite');
      tx.objectStore('person').put(Uint8Array.from(hex.match(/../g), (h) => parseInt(h, 16)), 'key');
      tx.oncomplete = () => done();
      tx.onerror = () => fail(tx.error);
    };
  });
}

const server = await serve(args.site);
const origin = `http://127.0.0.1:${server.address().port}/`;
let browser;
try {
  browser = await chromium.launch();
} catch (e) {
  console.error(`Playwright's Chromium cannot start — run: ${INSTALL}\n${e}`);
  process.exit(3);
}
const logs = [];
try {
  const page = await browser.newPage();
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await page.goto(origin);
  await page.evaluate(hold, args.key);
  await page.goto('about:blank');
  await page.goto(`${origin}#${args.token}`);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const result = await page.evaluate(() => ({
    status: document.getElementById('status').textContent,
    answer: document.getElementById('response').textContent,
    lines: window.__lines,
  }));
  console.log(JSON.stringify({ ...result, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await browser.close();
  server.close();
}
