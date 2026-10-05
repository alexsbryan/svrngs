# `*.localhost` in WebKit and Safari — 2026-10-05

u5-2 asks first whether WebKit resolves `*.localhost`, so a computer can
serve each lend at an origin of its own on loopback,
`http://<id>.localhost:<port>/` (DESIGN §3, "Lent apps", call 1), or needs a
loopback port per lend. It does, in every engine measured, as a secure
context with each host's cookies, storage and worker its own. No port per
lend is needed. (A port per lend would not keep cookies apart: a cookie is
scoped by host, never by port.)

**How it ran.** One HTTP server on 127.0.0.1, any port, answering every host:
a page with COOP and COEP (`same-origin`, `require-corp`) that sets a
host-only cookie `who=<its host>` and registers a worker at `/sw.js`.

- Playwright 1.59.1's Chromium (desktop) and WebKit (as an iPhone 15),
  headless, `measure.mjs` below: each loads `a.localhost`, `b.localhost` and
  `localhost` on that port and reads `isSecureContext`,
  `crossOriginIsolated`, `typeof SharedArrayBuffer`, `document.cookie`, the
  worker's registration and `crypto.subtle`.
- Safari 26.5.2 on this Mac (macOS 26.5.2): `safaridriver` refused ("You
  must enable 'Allow remote automation'"), so `open -g -a Safari
  http://a.localhost:<port>/`, whose page posts the same readings back and
  goes on to `b.localhost`.

**What it found.** Every engine, every host:
`{"secure":true,"coi":true,"sab":"function","sw":"registered","subtle":true}`,
and the cookie each read was its own host's alone: `who=a.localhost` on
`a.localhost`, `who=b.localhost` on `b.localhost`, `who=localhost` on
`localhost`. The server saw each worker's script fetched carrying only its
host's cookie (`a.localhost:<port> /sw.js cookie=who=a.localhost`). Safari's
user agent: `Version/26.5.2 Safari/605.1.15`. macOS's own resolver answers
`abc.localhost` with `::1` and `127.0.0.1` (`getaddrinfo`), which the
command's check for a wildcard leans on (u5-2).

```js
// measure.mjs — node measure.mjs, with web/'s Playwright installed
import http from 'node:http';
import { createRequire } from 'node:module';
const pw = createRequire(new URL('../../../../web/', import.meta.url))('playwright');
const server = http.createServer((req, res) => {
  const host = req.headers.host.split(':')[0];
  if (req.url === '/sw.js') return res.writeHead(200, { 'content-type': 'text/javascript' }).end("self.onfetch=()=>{}");
  res.writeHead(200, { 'content-type': 'text/html', 'set-cookie': `who=${host}; Path=/`,
    'cross-origin-opener-policy': 'same-origin', 'cross-origin-embedder-policy': 'require-corp' });
  res.end('<!doctype html><title>t</title>hi');
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
for (const engine of ['chromium', 'webkit']) {
  const browser = await pw[engine].launch();
  const page = await (await browser.newContext(engine === 'webkit' ? pw.devices['iPhone 15'] : {})).newPage();
  for (const host of ['a.localhost', 'b.localhost', 'localhost']) {
    await page.goto(`http://${host}:${server.address().port}/`);
    console.log(engine, host, JSON.stringify(await page.evaluate(async () => {
      let sw;
      try { await navigator.serviceWorker.register('/sw.js'); sw = 'registered'; } catch (e) { sw = String(e); }
      return { secure: isSecureContext, coi: self.crossOriginIsolated, sab: typeof SharedArrayBuffer,
        cookie: document.cookie, sw, subtle: !!crypto.subtle };
    })));
  }
  await browser.close();
}
server.close();
```
