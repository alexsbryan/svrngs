// The browser half of crates/svrngs-node/tests/browser/keep_going.rs
// (KEEP_GOING_AT_HOME 4, 6, 10, 29): a phone that opened the group, and with
// `args.open` an app of it, once while everything answered, its browser then
// fully closed and opened again on what it saved. Between the two it writes
// `args.ready`, a file, and waits for `go`, while the Rust half cuts what the
// scene cuts; with `args.siteDown` the page's public site is gone too, and
// with `args.missing` the named saved piece is deleted first, so a reopen
// that does not need every piece is seen to.
//
//   node web/tests/reopen.mjs '{"engine": ..., "site": ..., "key": ...,
//     "token": ..., "ready": "<file>", "open": "<the lend's name>",
//     "siteDown": true, "missing": "shim.js"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline';
import { held, inApp, launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = pageAt(server);
const profile = mkdtempSync(join(tmpdir(), 'svrngs-reopen-'));
let { context, close } = await launch(args.engine, { profile });
const logs = [];
const watch = (page) => {
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  return page;
};
// In the group's page: the group as its store keeps it.
const keptGroup = async () => {
  const app = document.querySelector('script[type=module]').src;
  const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
  return (await store.groups())[0];
};
// In a lend's page: what this origin's caches hold, by the file's name.
const pieces = async () => {
  const held = {};
  for (const name of await caches.keys())
    held[name] = (await (await caches.open(name)).keys()).map((r) => new URL(r.url).pathname);
  return held;
};
// In a lend's page: every piece named `piece`, gone from this origin's caches.
const lose = async (piece) => {
  let gone = 0;
  for (const name of await caches.keys()) {
    const cache = await caches.open(name);
    for (const kept of await cache.keys())
      if (new URL(kept.url).pathname === `/${piece}` && (await cache.delete(kept))) gone += 1;
  }
  return gone;
};
const status = (page) =>
  page.evaluate(() => {
    const said = document.getElementById('status');
    return { said: said?.textContent, kind: said?.className };
  });

try {
  let page = watch(await context.newPage());
  await held(page, origin, args);
  const first = { ...(await status(page)), group: await page.evaluate(keptGroup) };
  let app = null;
  let saved = null;
  let lost = null;
  if (args.open) {
    const button = page.locator('#lent button', { hasText: args.open });
    await Promise.all([page.waitForURL(inApp, { timeout: 90_000 }), button.click()]);
    await page.waitForFunction(() => document.body.innerText.includes('hi'), null, {
      timeout: 60_000,
    });
    app = page.url();
    saved = await page.evaluate(pieces);
    if (args.missing) lost = await page.evaluate(lose, args.missing);
  }
  await close();
  writeFileSync(args.ready, 'ready');
  const input = createInterface({ input: process.stdin });
  await new Promise((done) => input.once('line', done));
  input.close();
  if (args.siteDown) {
    server.closeAllConnections();
    await new Promise((done) => server.close(done));
  }

  ({ context, close } = await launch(args.engine, { profile }));
  page = watch(context.pages()[0] ?? (await context.newPage()));
  await page.goto(origin);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const again = await status(page);
  let reopened = null;
  if (app) {
    const failed = (e) => ({ failed: String(e) });
    reopened = await page.goto(app).catch(failed);
    if (!reopened?.failed) {
      reopened = await page
        .waitForFunction(() => document.body.innerText.includes('hi'), null, { timeout: 30_000 })
        .then(
          () => ({ text: 'hi' }),
          (e) => failed(e),
        );
    }
    reopened.body = await page.evaluate(() => document.body?.innerText).catch(() => null);
  }
  console.log(JSON.stringify({ first, again, app, saved, lost, reopened, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.closeAllConnections();
  server.close();
  rmSync(profile, { recursive: true, force: true });
}
