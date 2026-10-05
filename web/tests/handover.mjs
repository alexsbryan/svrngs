// The browser half of u4-10's test (crates/svrngs-node/tests/browser):
// Safari's worker on an iPhone, which read no sealing key where its page
// had kept one (2026-10-05). The worker here is planted so (page.mjs
// `serve`'s `worker`, PLANT): its every read of the `seal` store comes back
// empty, while the page's reads it. The page holds a joined group, opens
// the link and then the lend `args.open` from its list: the worker serves
// it with the key the page handed it, and its `<video>` plays. Then the
// browser stops the worker — Chromium through its DevTools protocol, WebKit
// by closing the browser and opening it again — and the lend's address is
// opened directly: the worker holds no key and cannot unseal, so it
// answers with the hand-over page, which hands the key over and opens the
// address again, and the `<video>` plays. Last, the page's own unseal must
// still open the person's key. The worker's trace is read at `/trace`
// (sw.js), as a phone's is. Prints what it saw as one JSON line.
//
//   node web/tests/handover.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the lend's name>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { hold, launch, play, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);

// Safari's worker: a read of the `seal` store asks for a key never put.
const PLANT = `{
  const get = IDBObjectStore.prototype.get;
  IDBObjectStore.prototype.get = function (k) {
    return get.call(this, this.name === 'seal' ? 'planted: Safari reads no sealing key here' : k);
  };
}`;

// In the page: the person's key in the group the store kept, unsealed by
// the page, as hex, or why it did not open.
async function unsealed() {
  const app = document.querySelector('script[type=module]').src;
  const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
  const [group] = await store.groups();
  try {
    return [...(await store.unseal(group))].map((b) => b.toString(16).padStart(2, '0')).join('');
  } catch (e) {
    return `failed: ${e}`;
  }
}

// In a page the worker controls: what the worker traced (sw.js `TRACE`).
// The hand-over page's policy lets it fetch nothing, so when it is the one
// still showing, that is said instead.
const traced = (page) =>
  page.evaluate(() => fetch('/trace').then((r) => r.text())).catch((e) => `the trace was not read: ${e}`);

const server = await serve(args.site, { worker: PLANT });
const origin = `http://127.0.0.1:${server.address().port}/`;
const profile = args.engine === 'webkit' ? mkdtempSync(join(tmpdir(), 'svrngs-webkit-')) : undefined;
let { context, close } = await launch(args.engine, { profile });
const logs = [];
let page;
const watch = (context) => {
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  context.on('page', (p) => p.on('pageerror', (e) => logs.push(`pageerror: ${e}`)));
};
try {
  watch(context);
  page = await context.newPage();
  await page.goto(origin);
  await page.evaluate(hold, { hex: args.key, token: args.token });
  await page.goto('about:blank');
  await page.goto(`${origin}#${args.token}`);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const listed = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
  const button = page.locator('#lent button', { hasText: args.open });
  await Promise.all([page.waitForURL(/\/l\//, { timeout: 90_000 }), button.click()]);
  await page.waitForLoadState('load');
  const opened = new URL(page.url()).pathname;
  const first = await page.evaluate(play, { at: null, within: 15_000 });
  const handedTrace = await traced(page);

  // The browser stops the worker, and with it the key it was handed.
  if (profile) {
    await close();
    ({ context, close } = await launch(args.engine, { profile }));
    watch(context);
    page = context.pages()[0] ?? (await context.newPage());
  } else {
    const cdp = await context.newCDPSession(page);
    const runs = [];
    cdp.on('ServiceWorker.workerVersionUpdated', ({ versions }) => {
      for (const v of versions) runs.push(v.runningStatus);
    });
    await cdp.send('ServiceWorker.enable');
    await cdp.send('ServiceWorker.stopAllWorkers');
    const stopped = Date.now();
    while (!runs.includes('stopped') && Date.now() - stopped < 10_000) await new Promise((r) => setTimeout(r, 50));
    if (!runs.includes('stopped')) throw new Error(`the worker did not stop: ${runs}`);
  }
  await page.goto(new URL(opened, origin).href);
  await page.waitForSelector('video', { timeout: 60_000 }).catch(() => {});
  await page.waitForLoadState('load');
  const reopened = new URL(page.url()).pathname;
  const replayed = await page.evaluate(play, { at: null, within: 15_000 });
  const stoppedTrace = await traced(page);

  await page.goto(origin);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const unsealedAfter = await page.evaluate(unsealed);
  console.log(
    JSON.stringify({ listed, opened, first, handedTrace, reopened, replayed, stoppedTrace, unsealedAfter, logs }),
  );
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
  if (profile) rmSync(profile, { recursive: true, force: true });
}
