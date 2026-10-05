// What the browser halves share (drive.mjs, join.mjs): the built page
// served as an origin serves it, and Playwright's headless Chromium. Each
// exits 3 naming the install command when Playwright or its Chromium is
// absent — the browser step fails then, never skips.
import http from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize, resolve } from 'node:path';

export const INSTALL = 'npm ci --prefix web && npx --prefix web playwright install chromium';

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

export async function launch() {
  let chromium;
  try {
    ({ chromium } = await import('playwright'));
  } catch (e) {
    console.error(`Playwright is not installed — run: ${INSTALL}\n${e}`);
    process.exit(3);
  }
  try {
    return await chromium.launch();
  } catch (e) {
    console.error(`Playwright's Chromium cannot start — run: ${INSTALL}\n${e}`);
    process.exit(3);
  }
}
