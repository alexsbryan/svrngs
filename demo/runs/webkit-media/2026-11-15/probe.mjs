// WebKit, a persistent profile: play a film, close, open again on the
// same profile (or a fresh one), play again. No svrngs: a plain origin
// answering ranges.
import http from 'node:http';
import { mkdtempSync, readFileSync, statSync } from 'node:fs';
// Run in the gate image with the checkout at /work (scripts/gate/README.md).
const pw = await import('/work/web/node_modules/playwright/index.mjs');
const [film, mode] = process.argv.slice(2);
const size = statSync(film).size;
const bytes = readFileSync(film);
const server = http.createServer((req, res) => {
  if (!req.url.startsWith('/title.mp4')) {
    res.writeHead(200, { 'content-type': 'text/html' });
    return res.end(`<!doctype html><video src="title.mp4" playsinline preload="${process.env.PRELOAD ?? 'auto'}"></video>`);
  }
  const m = /bytes=(\d+)-(\d*)/.exec(req.headers.range ?? '');
  const from = m ? Number(m[1]) : 0, to = m && m[2] ? Number(m[2]) : size - 1;
  res.writeHead(m ? 206 : 200, { 'content-type': 'video/mp4', 'accept-ranges': 'bytes', 'content-length': to - from + 1, ...(m ? { 'content-range': `bytes ${from}-${to}/${size}` } : {}) });
  res.end(bytes.subarray(from, to + 1));
}).listen(0, '127.0.0.1');
await new Promise((r) => server.once('listening', r));
const url = `http://127.0.0.1:${server.address().port}/`;
const play = async (page, at) => page.evaluate(async ({ at }) => {
  const v = document.querySelector('video');
  const t0 = performance.now();
  if (at === null) { v.muted = true; v.play().catch(() => {}); } else v.currentTime = at;
  const from = at ?? 0;
  for (;;) {
    if (!v.seeking && !v.paused && v.currentTime > from + 0.25) return { ms: Math.round(performance.now() - t0), at: v.currentTime };
    if (performance.now() - t0 > 15000) return { ms: null, at: v.currentTime, ready: v.readyState, network: v.networkState, paused: v.paused };
    await new Promise((r) => setTimeout(r, 20));
  }
}, { at });
const device = process.env.DESKTOP ? {} : pw.devices['iPhone 15'];
const profile = mkdtempSync('/tmp/probe-');
const open = async (p) => {
  let context;
  if (mode === 'ephemeral') {
    const browser = await pw.webkit.launch();
    context = await browser.newContext(device);
    context.on('close', () => browser.close());
  } else context = await pw.webkit.launchPersistentContext(p, device);
  const page = process.env.NEWPAGE ? await context.newPage() : (context.pages()[0] ?? (await context.newPage()));
  await page.goto(url);
  if (process.env.CLICK) await page.mouse.click(5, 5);
  return { context, page };
};
let { context, page } = await open(profile);
const first = await play(page, null);
const seek = await play(page, 40);
await context.close();
({ context, page } = await open(mode === 'fresh' ? mkdtempSync('/tmp/probe-') : profile));
const again = await play(page, null);
const seek2 = await play(page, 52);
await context.close();
console.log(JSON.stringify({ desktop: !!process.env.DESKTOP, click: !!process.env.CLICK, mode: mode ?? 'same profile', first, seek, again, seek2 }));
server.close();
process.exit(0);
