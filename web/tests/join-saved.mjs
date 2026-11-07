// The browser half of crates/svrngs-node/tests/browser/keep_going.rs's join
// on a saved page (KEEP_GOING_AT_HOME 3): someone in no group opens the
// group's page once, so it is saved, and the browser closes. It writes
// `args.ready` and waits for `go` while the Rust half cuts the internet. With
// the page's site gone it opens again and tries `args.old`, a link naming
// only the public relay, then `args.link`, which names both, typing a name
// when the page asks.
//
//   node web/tests/join-saved.mjs '{"engine": ..., "site": ..., "link": ...,
//     "old": ..., "ready": "<file>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline';
import { launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const profile = mkdtempSync(join(tmpdir(), 'svrngs-join-saved-'));
const server = await serve(args.site);
const origin = pageAt(server);
let { context, close } = await launch(args.engine, { profile });
const logs = [];
const watch = (page) => {
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  return page;
};
// What the page said once it has asked, or decided, and how it said it.
const said = (page) =>
  page.evaluate(() => {
    const status = document.getElementById('status');
    return { said: status?.textContent, kind: status?.className };
  });
const settled = (page) =>
  page.waitForFunction(
    () =>
      !document.getElementById('join').hidden ||
      /good|bad/.test(document.getElementById('status').className),
    null,
    { timeout: 90_000 },
  );
try {
  let page = watch(await context.newPage());
  await page.goto(origin);
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready;
    while ((await caches.keys()).length === 0) await new Promise((done) => setTimeout(done, 100));
  });
  await page.close();
  await close();
  writeFileSync(args.ready, 'ready');
  const input = createInterface({ input: process.stdin });
  await new Promise((done) => input.once('line', done));
  input.close();
  server.closeAllConnections();
  await new Promise((done) => server.close(done));
  ({ context, close } = await launch(args.engine, { profile }));
  page = watch(context.pages()[0] ?? (await context.newPage()));
  await page.goto('about:blank');
  await page.goto(`${origin}#${args.old}`);
  await settled(page);
  const old = await said(page);
  await page.goto('about:blank');
  await page.goto(`${origin}#${args.link}`);
  await settled(page);
  if (await page.evaluate(() => !document.getElementById('join').hidden)) {
    await page.fill('#name', 'Pia');
    await page.click('#join button');
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  }
  console.log(JSON.stringify({ old, joined: await said(page), logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.closeAllConnections();
  server.close();
  rmSync(profile, { recursive: true, force: true });
}
