// The browser half of the runtime's test (u4-6; crates/svrngs-node/tests/
// browser): a lent film played and sought at its lend's own origin (u5-2).
// It serves the built page on 127.0.0.1 as `localhost`, whose every name
// this computer answers, and, in Playwright's headless `engine`
// (page.mjs), holds a joined group in the page's store, opens the page and
// then the lend `args.open` from its list, so the page goes to the lend's
// origin, `<label>.localhost`, whose worker serves it from its lender's
// computer. There it plays the page's `<video>`, seeks it at
// `args.seeks[0..2]`, has the browser stop the worker — Chromium through
// its DevTools protocol, WebKit by closing and opening the browser again —
// seeks at the last, and loads a range of a second computer's lend, which
// the first reads, at its origin (`args.other`), and an open-ended range of
// it too (`args.openEnded`, `bytes=N-`), and asks paths of other lends
// it reads (`args.refusals`), each answer's status and whose word it is.
// It prints what it saw as one JSON line.
//
//   node web/tests/runtime.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ...,
//     "open": "<the lend's name>", "seeks": [s, s, s],
//     "other": {"lend": "<label>", "path": "<file>", "range": "bytes=a-b"},
//     "openEnded": {"path": ..., "range": "bytes=a-"},
//     "refusals": [{"lend": "<label>", "path": ...}]}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { hold, launch, play, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);

// In the lend's page: a range of `path` at the lend `lend`'s origin, read
// whole.
async function range({ lend, path, range }) {
  try {
    const at = `${location.protocol}//${lend}${location.host.slice(location.hostname.split('.')[0].length)}/${path}`;
    const res = await fetch(at, { headers: { range } });
    const bytes = new Uint8Array(await res.arrayBuffer());
    const sha = [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))]
      .map((x) => x.toString(16).padStart(2, '0'))
      .join('');
    return { status: res.status, contentRange: res.headers.get('content-range'), bytes: bytes.length, sha };
  } catch (e) {
    return { error: String(e) };
  }
}

// In the lend's page: `path` at the lend `lend`'s origin, its status and
// whose word it is (mnp-1): svrngs's kind, or null for the lend's own.
async function refusal({ lend, path }) {
  const at = `${location.protocol}//${lend}${location.host.slice(location.hostname.split('.')[0].length)}/${path}`;
  const res = await fetch(at).catch((e) => ({ status: String(e), headers: new Headers() }));
  return { path, status: res.status, refused: res.headers.get('x-svrngs-refused') };
}

const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
// WebKit's context keeps a profile, so the browser can be closed and
// opened again on it; Chromium's is stopped through DevTools.
const profile = args.engine === 'webkit' ? mkdtempSync(join(tmpdir(), 'svrngs-webkit-')) : undefined;
let { context, close } = await launch(args.engine, { profile });
const logs = [];
const watch = (context) => context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  watch(context);
  let page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await page.goto(origin);
  await page.evaluate(hold, { hex: args.key, token: args.token });
  await page.goto('about:blank');
  await page.goto(`${origin}#${args.token}`);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const listed = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
  const button = page.locator('#lent button', { hasText: args.open });
  const away = (url) => url.hostname.endsWith('.localhost');
  await Promise.all([page.waitForURL(away, { timeout: 90_000 }), button.click()]);
  await page.waitForSelector('video', { timeout: 60_000 });
  const opened = page.url();

  // The worker's runs, as Chromium reports them; WebKit reports none.
  const cdp = profile ? null : await context.newCDPSession(page);
  const runs = [];
  cdp?.on('ServiceWorker.workerVersionUpdated', ({ versions }) => {
    for (const v of versions) if (runs.at(-1) !== v.runningStatus) runs.push(v.runningStatus);
  });
  await cdp?.send('ServiceWorker.enable');

  const first = await page.evaluate(play, { at: null, within: 15_000 });
  const seeks = [];
  for (const at of args.seeks.slice(0, -1)) seeks.push(await page.evaluate(play, { at, within: 15_000 }));
  // The browser stops the worker mid-play; the next range must still
  // answer. Chromium stops it through DevTools, the page left playing.
  // Playwright cannot reach WebKit's worker, so WebKit's browser is closed
  // and opened again, as a phone's is when it ends a browser left in the
  // background: no worker outlives it, and the page opens again at the
  // lend's address, where the worker it starts serves it.
  let reopened = null;
  let replayed = null;
  if (cdp) {
    await cdp.send('ServiceWorker.stopAllWorkers');
    const stopped = Date.now();
    while (!runs.includes('stopped') && Date.now() - stopped < 10_000) await new Promise((r) => setTimeout(r, 50));
  } else {
    await close();
    ({ context, close } = await launch(args.engine, { profile }));
    watch(context);
    page = context.pages()[0] ?? (await context.newPage());
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    await page.goto(opened);
    await page.waitForSelector('video', { timeout: 60_000 });
    reopened = page.url();
    replayed = await page.evaluate(play, { at: null, within: 15_000 });
  }
  const runsAtStop = [...runs];
  seeks.push(await page.evaluate(play, { at: args.seeks.at(-1), within: 15_000 }));
  const other = await page.evaluate(range, args.other);
  const openEnded = args.openEnded ? await page.evaluate(range, args.openEnded) : null;
  const refusals = [];
  for (const r of args.refusals ?? []) refusals.push(await page.evaluate(refusal, r));
  console.log(
    JSON.stringify({ listed, opened, reopened, replayed, first, seeks, runsAtStop, runs, other, openEnded, refusals, logs }),
  );
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
  if (profile) rmSync(profile, { recursive: true, force: true });
}
