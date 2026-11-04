// How long an idle page's service worker keeps what it holds in memory:
// a worker draws a run id at start; the page asks it at 0, 35 and 70 s.
import { launch } from '../../../../web/tests/page.mjs';
import http from 'node:http';
const sw = `const run = Math.random().toString(36).slice(2);
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (e) => e.waitUntil(self.clients.claim()));
self.addEventListener('fetch', (e) => { if (new URL(e.request.url).pathname === '/run') e.respondWith(new Response(run)); });`;
const html = `<!doctype html><script>navigator.serviceWorker.register('/sw.js')</script>`;
const server = http.createServer((q, r) => { r.setHeader('content-type', q.url === '/sw.js' ? 'text/javascript' : 'text/html'); r.end(q.url === '/sw.js' ? sw : html); }).listen(0);
await new Promise((d) => server.on('listening', d));
const engines = process.argv.slice(2);
await Promise.all(engines.map(async (engine) => {
  const { context, close } = await launch(engine);
  const p = await context.newPage();
  await p.goto(`http://localhost:${server.address().port}/`);
  await p.waitForFunction(() => navigator.serviceWorker.controller, null, { timeout: 30000 }).catch(() => p.reload());
  await p.waitForFunction(() => navigator.serviceWorker.controller, null, { timeout: 30000 });
  const runs = [];
  for (const wait of [0, 35, 70]) {
    await p.waitForTimeout(wait ? 35000 : 0);
    runs.push(await p.evaluate(() => fetch('/run').then((r) => r.text())));
  }
  console.log(engine, JSON.stringify(runs), new Set(runs).size === 1 ? 'one run: memory kept' : 'restarted: memory lost');
  await close();
}));
server.close();
