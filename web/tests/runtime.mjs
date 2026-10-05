// The browser half of the runtime's test (u4-6; crates/svrngs-node/tests/
// browser): a lent film played and sought under the service worker's
// scope. It serves the built page on 127.0.0.1 and, in Playwright's
// headless `engine` (page.mjs), holds a joined group in the page's store,
// opens the page and then the lend `args.open` from its list, so the page
// goes to the lend's address, which the worker serves from its lender's
// computer. There it plays the page's `<video>`, seeks it at
// `args.seeks[0..2]`, has the browser stop the worker — Chromium through
// its DevTools protocol, WebKit by closing and opening the browser again —
// seeks at the last, and loads a range of a second computer's lend by a
// path relative to the page (`args.other`), and, when `args.open` names
// one, an open-ended range of it too (`args.openEnded`, `bytes=N-`). It
// prints what it saw as one JSON line.
//
//   node web/tests/runtime.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ...,
//     "open": "<the lend's name>", "seeks": [s, s, s],
//     "other": {"path": "../../<key>/<lend>/<file>", "range": "bytes=a-b"},
//     "openEnded": {"path": ..., "range": "bytes=a-"}}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { hold, launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);

// In the lend's page: play its `<video>`, or seek it to `at`, and answer
// how long until it plays on from there — `ms`, or `null` past `within` —
// with where it stood then, or what the page says when it holds no video.
async function play({ at, within }) {
  const v = document.querySelector('video');
  if (!v) return { ms: null, error: `no <video> on the page, which says: ${document.body.innerText.slice(0, 200)}` };
  const started = performance.now();
  if (at === null) {
    v.muted = true;
    v.play().catch(() => {});
  } else {
    v.currentTime = at;
  }
  const from = at ?? 0;
  return new Promise((done) => {
    const check = () => {
      const t = performance.now() - started;
      if (!v.seeking && !v.paused && v.currentTime > from + 0.25) {
        done({ ms: Math.round(t), at: v.currentTime });
      } else if (t > within) {
        done({ ms: null, at: v.currentTime, seeking: v.seeking, paused: v.paused, error: v.error?.message ?? null });
      } else {
        setTimeout(check, 20);
      }
    };
    check();
  });
}

// In the lend's page: a range of `path`, relative to the page, read whole.
async function range({ path, range }) {
  try {
    const res = await fetch(path, { headers: { range } });
    const bytes = new Uint8Array(await res.arrayBuffer());
    const sha = [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))]
      .map((x) => x.toString(16).padStart(2, '0'))
      .join('');
    return { status: res.status, contentRange: res.headers.get('content-range'), bytes: bytes.length, sha };
  } catch (e) {
    return { error: String(e) };
  }
}

const server = await serve(args.site);
const origin = `http://127.0.0.1:${server.address().port}/`;
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
  await Promise.all([page.waitForURL(/\/l\//, { timeout: 90_000 }), button.click()]);
  await page.waitForLoadState('load');
  const opened = new URL(page.url()).pathname;

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
    await page.goto(new URL(opened, origin).href);
    reopened = new URL(page.url()).pathname;
    replayed = await page.evaluate(play, { at: null, within: 15_000 });
  }
  const runsAtStop = [...runs];
  seeks.push(await page.evaluate(play, { at: args.seeks.at(-1), within: 15_000 }));
  const other = await page.evaluate(range, args.other);
  const openEnded = args.openEnded ? await page.evaluate(range, args.openEnded) : null;
  console.log(
    JSON.stringify({ listed, opened, reopened, replayed, first, seeks, runsAtStop, runs, other, openEnded, logs }),
  );
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
  if (profile) rmSync(profile, { recursive: true, force: true });
}
