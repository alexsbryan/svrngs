// V1.1.3's smoke, the phone's half, on the Mac: the release's page served
// here as `localhost`, whose every name this computer answers, and one
// phone in Playwright's `engine` as an iPhone. It joins House by its link,
// opens lee's llama, then cy's shelf; then, at each line on stdin, goes on:
// `frozen` (cy's computer stopped) opens the shelf's address again;
// `ada` (the link's computer stopped too) opens the group's page again.
// Each step's page is kept as a screenshot under `shots`, and the whole as
// one JSON line on stdout; `ready <step>` lines on stderr say where it waits.
//
//   node phone.mjs '{"engine": ..., "site": ..., "token": ..., "name": ..., "shots": ...}'
import http from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize, resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { chromium, webkit, devices } from '/Users/alexsbryan/dev/svrngs/web/node_modules/playwright/index.mjs';

const args = JSON.parse(process.argv[2]);
const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.wasm': 'application/wasm', '.css': 'text/css; charset=utf-8' };
const root = resolve(args.site);
const server = http.createServer(async (req, res) => {
  const path = new URL(req.url, 'http://page').pathname;
  const file = join(root, normalize(path.endsWith('/') ? `${path}index.html` : path));
  if (!file.startsWith(root)) return res.writeHead(403).end();
  try {
    const body = await readFile(file);
    res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream', 'cache-control': 'no-cache' }).end(body);
  } catch {
    res.writeHead(404).end();
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const origin = `http://localhost:${server.address().port}/`;
const lines = createInterface({ input: process.stdin });
const waiting = [];
lines.on('line', (l) => waiting.shift()?.(l.trim()));
// In the lend's page: a socket at `path`, opened or not within 10 s.
function opens(path) {
  return new Promise((done) => {
    const s = new WebSocket(`${location.origin.replace(/^http/, 'ws')}${path}`);
    const t = setTimeout(() => (s.close(), done('no answer in 10 s')), 10_000);
    s.onopen = () => (clearTimeout(t), s.close(), done('open'));
    s.onclose = (e) => (clearTimeout(t), done(`closed ${e.code}`));
  });
}
const next = (step) => (console.error(`ready ${step}`), new Promise((r) => waiting.push(r)));

const browser = await { chromium, webkit }[args.engine].launch();
const out = { engine: args.engine, origin };
// The link opened on a computer first: what the page offers there.
{
  const desk = await browser.newContext(devices[args.engine === 'webkit' ? 'Desktop Safari' : 'Desktop Chrome']);
  const p = await desk.newPage();
  await p.goto(`${origin}#${args.token}`);
  await p.waitForSelector('#join:not([hidden]), #status.good, #status.bad', { timeout: 60_000 }).catch(() => {});
  const seen = {};
  for (const id of ['computer', 'join', 'ask', 'status']) seen[id] = await p.locator(`#${id}`).isVisible();
  out.computer = { seen, status: await p.locator('#status').textContent(), text: (await p.evaluate(() => document.body.innerText)).slice(0, 800) };
  await p.screenshot({ path: `${args.shots}/${args.engine}-0-link-on-a-computer.png`, fullPage: true });
  await desk.close();
}
const context = await browser.newContext(devices['iPhone 15']);
const page = await context.newPage();
const logs = [];
page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
const shot = (n) => page.screenshot({ path: `${args.shots}/${args.engine}-${n}.png` }).catch(() => {});
const shown = async () => ({ url: page.url(), title: await page.title().catch(() => null), text: (await page.evaluate(() => document.body?.innerText ?? '').catch(() => '')).slice(0, 600) });
const status = () => page.evaluate(() => ({ status: document.getElementById('status')?.textContent, said: document.getElementById('response')?.textContent, lent: [...document.querySelectorAll('#lent button')].map((b) => b.textContent) }));
try {
  let t0 = Date.now();
  await page.goto(`${origin}#${args.token}`);
  await page.waitForSelector('#join:not([hidden])', { timeout: 60_000 });
  await page.fill('#name', args.name);
  await page.click('#join button');
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  await page.waitForFunction(() => document.querySelectorAll('#lent button').length >= 2, null, { timeout: 30_000 }).catch(() => {});
  out.joined = { ms: Date.now() - t0, ...(await status()) };
  await shot('1-joined');

  t0 = Date.now();
  const doc = page.waitForResponse((r) => r.request().isNavigationRequest() && r.url().includes('.localhost'), { timeout: 60_000 }).catch(() => null);
  await page.locator('#lent button', { hasText: 'llama' }).click();
  const r = await doc;
  await page.waitForSelector('textarea', { timeout: 60_000 }).catch(() => {});
  out.llama = { ms: Date.now() - t0, status: r?.status() ?? null, textarea: await page.locator('textarea').count(), ...(await shown()) };
  await shot('2-llama');

  await page.goto(origin);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  await page.locator('#lent button', { hasText: 'shelf' }).click();
  await page.waitForFunction(() => document.title === "cy's shelf", null, { timeout: 60_000 }).catch(() => {});
  out.shelf = await shown();
  out.socket = { first: await page.evaluate(opens, '/sync') };
  await shot('3-shelf');

  // The lid: cy's computer stopped, a socket asked for; started again, a
  // socket asked for each second, the page not opened again.
  await next('shelf');
  out.socket.stopped = await page.evaluate(opens, '/sync');
  await next('thaw');
  t0 = Date.now();
  for (let tries = 1; Date.now() - t0 < 60_000; tries++) {
    out.socket.back = { last: await page.evaluate(opens, '/sync'), tries, ms: Date.now() - t0 };
    if (out.socket.back.last === 'open') break;
    await new Promise((r) => setTimeout(r, 1000));
  }

  await next('shelf2');
  t0 = Date.now();
  const res = await page.goto(out.shelf.url, { timeout: 90_000 });
  out.away = { ms: Date.now() - t0, status: res?.status() ?? null, refused: (await res?.headerValue('x-svrngs-refused')) ?? null, ...(await shown()) };
  await shot('4-away');

  await next('away');
  t0 = Date.now();
  await page.goto(origin);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  await page.waitForFunction(() => document.querySelectorAll('#lent button').length >= 1, null, { timeout: 30_000 }).catch(() => {});
  out.failover = { ms: Date.now() - t0, ...(await status()) };
  await shot('5-failover');
} catch (e) {
  out.error = String(e);
  await shot('error');
}
out.logs = logs.slice(-80);
console.log(JSON.stringify(out));
await browser.close();
server.close();
process.exit(0);
