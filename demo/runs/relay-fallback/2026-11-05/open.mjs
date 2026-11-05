// The fresh browser opening a group's link under the page's production
// policy (relay fallbacks; the incident this queue corrects): the built
// page served as production serves it — its own meta CSP, `connect-src
// 'self' wss: https:`, no loopback allowance (web/index.html:51) — and a
// link whose token carries an HTTP-only relay, as the Pi's invitation did
// (http://10.42.242.105:3340/). The policy refuses the relay before the
// dial leaves, so the page never asks a name and says the computer did
// not answer. Prints what it saw as one JSON line.
//
// With `allow`, the control alone: the policy's `connect-src` is given
// the tests' loopback allowance before the page is served, so the same
// page reaches the same plain relay — showing the failure above is the
// policy, not the relay. It cannot prove the correction (CHARTER.md).
//
//   node open.mjs '{"engine":"chromium","site":"target/web/site","token":"<token>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when
// Playwright or its browser is absent; 1 on anything else.
import http from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize, resolve } from 'node:path';
import { launch, pageAt } from '../../../../web/tests/page.mjs';

const args = JSON.parse(process.argv[2]);
const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
};
const root = resolve(args.site);
const server = await new Promise((done) => {
  const s = http.createServer(async (req, res) => {
    const path = new URL(req.url, 'http://page').pathname;
    const file = join(root, normalize(path.endsWith('/') ? `${path}index.html` : path));
    if (!file.startsWith(root)) return res.writeHead(403).end();
    try {
      let body = await readFile(file);
      if (args.allow && /\/(index|404)\.html$/.test(file))
        body = String(body).replace(
          "connect-src 'self' wss: https:;",
          "connect-src 'self' wss: https: ws://127.0.0.1:* http://127.0.0.1:*;",
        );
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
      res.end(body);
    } catch {
      res.writeHead(404, { 'content-type': TYPES['.html'] }).end();
    }
  });
  s.listen(0, '127.0.0.1', () => done(s));
});
const origin = pageAt(server);
const { context, close } = await launch(args.engine, { unopened: true });
const logs = [];
try {
  await context.addInitScript(() => {
    window.__said = [];
    new MutationObserver(() => {
      const s = document.getElementById('status')?.textContent;
      if (s && window.__said.at(-1) !== s) window.__said.push(s);
    }).observe(document, { subtree: true, childList: true, characterData: true });
  });
  const page = await context.newPage();
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  page.on('requestfailed', (r) =>
    logs.push(`requestfailed: ${r.url()} ${r.failure()?.errorText}`),
  );
  await page.goto(`${origin}#${args.token}`);
  await page.waitForFunction(
    () =>
      /good|bad/.test(document.getElementById('status')?.className ?? '') ||
      !document.getElementById('join').hidden,
    null,
    { timeout: 90_000 },
  );
  console.log(
    JSON.stringify({
      status: await page.evaluate(() => document.getElementById('status').textContent),
      nameForm: await page.evaluate(() => !document.getElementById('join').hidden),
      said: await page.evaluate(() => window.__said),
      logs,
    }),
  );
} catch (e) {
  console.error(`the page did not settle: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
