// The browser half of u4-10's test (crates/svrngs-node/tests/browser):
// Safari's worker on an iPhone, which read nothing its page had kept
// (2026-10-05). Since u5-2 the worker that dials is a lend's, at the lend's
// own origin (lend.js), and what it reads is its origin's store; here it is
// planted so (page.mjs `serve`'s `worker`, PLANT): its every read of that
// store comes back empty, while its pages read it. The page holds a joined
// group, opens the link and then the lend `args.open` from its list: the
// lend's start page (open.js) hands its worker the pass in memory, and the
// `<video>` plays. Then the browser stops the worker — Chromium through its
// DevTools protocol, WebKit by closing the browser and opening it again —
// and a page it serves fetches: the worker holds no pass and reads none,
// so it answers itself, marked as svrngs's (mnp-1). Then the lend's
// address is opened directly: the worker holds no pass and
// reads none, so it sends the page to its start page, which reads the
// pass, hands it over and opens the address again, and the `<video>`
// plays. Last, the group's page's own unseal must still open the person's
// key. The worker's trace is read at `/svrngs-trace` (lend.js), as a
// phone's is. Prints what it saw as one JSON line.
//
// With `keysLost`, the worker is planted too as Safari's on an iPhone
// 2026-10-30 (KEYS_LOST): a message carrying a CryptoKey never reaches it.
// It also says what the lend's origin keeps of the pass (`keptPass`).
//
//   node web/tests/handover.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the lend's name>", "keysLost": bool}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { held, launch, play, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);

// Safari's worker: a read of its origin's store asks for a name never put.
const PLANT = `{
  const get = IDBObjectStore.prototype.get;
  IDBObjectStore.prototype.get = function (k) {
    return get.call(this, this.name === 'kept' ? 'planted: Safari reads nothing here' : k);
  };
}`;

// Safari's worker on an iPhone 2026-10-30, iOS Safari 26.6.1: any CryptoKey
// in a message to it, the holder's an open hands included, is lost, and
// `messageerror` fires in place of `message`; plain data and bytes arrive
// (demo/runs/ios/2026-10-30-observe/keyprobe/phone.log, cases B-E).
const KEYS_LOST = `{
  const add = self.addEventListener;
  const holds = (v) => v instanceof CryptoKey || (v !== null && typeof v === 'object' && Object.values(v).some(holds));
  self.addEventListener = function (type, f, o) {
    const lost = (e) => (holds(e.data) ? self.dispatchEvent(new MessageEvent('messageerror')) : f.call(this, e));
    return add.call(this, type, type === 'message' ? lost : f, o);
  };
}`;

// In a page of the lend's origin: the names of what it keeps of the pass
// (store.js `lendKeep`), and what its holder's key is there.
async function keptPass() {
  const open = indexedDB.open('svrngs-lend', 1);
  const db = await new Promise((done, fail) => Object.assign(open, { onsuccess: () => done(open.result), onerror: () => fail(open.error) }));
  const label = location.hostname.split('.')[0];
  const get = db.transaction('kept').objectStore('kept').get(`pass ${label}`);
  const pass = await new Promise((done, fail) => Object.assign(get, { onsuccess: () => done(get.result), onerror: () => fail(get.error) }));
  return { fields: Object.keys(pass ?? {}).filter((k) => pass[k] !== undefined).sort(), key: pass?.key?.constructor?.name ?? null };
}

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

// In a page the lend's worker controls: what the worker traced (lend.js
// `TRACE`).
const traced = (page) =>
  page.evaluate(() => fetch('/svrngs-trace').then((r) => r.text())).catch((e) => `the trace was not read: ${e}`);

const server = await serve(args.site, { worker: args.keysLost ? `${PLANT}\n${KEYS_LOST}` : PLANT });
const origin = `http://localhost:${server.address().port}/`;
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
  await held(page, origin, args);
  const listed = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
  const button = page.locator('#lent button', { hasText: args.open });
  const away = (url) => url.hostname.endsWith('.localhost');
  await Promise.all([page.waitForURL(away, { timeout: 90_000 }), button.click()]);
  await page.waitForSelector('video', { timeout: 60_000 });
  const opened = page.url();
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
  // The worker's own word (mnp-1): holding no pass and reading none, it
  // answers a page's fetch itself, marked as svrngs's, a pass refused.
  await page.goto(new URL('svrngs-trace', opened).href);
  const unheld = await page.evaluate(() =>
    fetch('/probe').then((r) => ({ status: r.status, refused: r.headers.get('x-svrngs-refused') })),
  );
  await page.goto(opened);
  await page.waitForSelector('video', { timeout: 60_000 }).catch(() => {});
  await page.waitForLoadState('load');
  const reopened = page.url();
  const replayed = await page.evaluate(play, { at: null, within: 15_000 });
  const stoppedTrace = await traced(page);
  const kept = await page.evaluate(keptPass).catch((e) => `not read: ${e}`);

  await page.goto(origin);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const unsealedAfter = await page.evaluate(unsealed);
  console.log(
    JSON.stringify({ listed, opened, first, handedTrace, unheld, reopened, replayed, stoppedTrace, kept, unsealedAfter, logs }),
  );
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
  if (profile) rmSync(profile, { recursive: true, force: true });
}
