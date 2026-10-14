// The browser half of v1ui-reach's test (crates/svrngs-node/tests/browser):
// the phone reaching whoever's awake. The page holds a joined group and
// opens its link while the link's computer answers; then it writes the file
// `args.ready` and waits for a line `go` on stdin, which the Rust half sends
// once it has stopped that computer; then it opens the group's page again,
// timed from the address given to its status shown, and opens the lend
// `args.open` from its list, timed to its page's title, `args.title`.
//
//   node web/tests/reach.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "ready": "<file>", "open": "<the lend's
//     name>", "title": "<its page's title>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { held, launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const go = new Promise((done) => createInterface({ input: process.stdin }).on('line', (l) => l.includes('go') && done()));
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
const shown = (page) =>
  page.evaluate(() => ({
    status: document.getElementById('status').textContent,
    said: document.getElementById('response').textContent,
    lent: [...document.querySelectorAll('#lent button')].map((b) => b.textContent),
    text: document.body.innerText,
  }));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(page, origin, args);
  const first = await shown(page);
  writeFileSync(args.ready, '');
  await go;
  await page.goto('about:blank');
  const t0 = Date.now();
  await page.goto(origin);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const listed = { ms: Date.now() - t0, ...(await shown(page)) };
  let opened = null;
  if (listed.lent.some((l) => l.includes(args.open))) {
    await page.locator('#lent button', { hasText: args.open }).click();
    await page.waitForFunction((t) => document.title === t, args.title, { timeout: 60_000 }).catch(() => {});
    opened = { ms: Date.now() - t0, url: page.url(), title: await page.title() };
  }
  console.log(JSON.stringify({ first, listed, opened, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
  process.stdin.destroy();
}
