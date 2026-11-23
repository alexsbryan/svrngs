// The browser half of the runtime's test (u4-6; crates/svrngs-node/tests/
// browser): a lent film played and sought at its lend's own origin (u5-2).
// It serves the built page on 127.0.0.1 as `localhost`, whose every name
// this computer answers, and, in Playwright's headless `engine`
// (page.mjs), holds a joined group in the page's store, opens the page and
// then the lend `args.open` from its list, so the page goes to the lend's
// origin, `<label>.localhost`, whose worker serves it from its lender's
// computer. There, with `args.seeks` (the playback step's alone,
// crates/svrngs-node/tests/browser/playback.rs), it plays the page's
// `<video>` and seeks it at `args.seeks[0..2]`; without, it fetches
// `args.film`, a range of the `<video>`'s file, through the worker
// (page.mjs `fetched`), which is what every other test asserts in place of
// the browser decoding it. Then it has the browser stop the worker —
// Chromium through its DevTools protocol, WebKit by closing and opening
// the browser again — and plays on, seeking at the last, or fetches the
// film's range again; and, when asked, loads a range of a lend it reads
// at its origin (`args.other`, the relay-rate measurement's). It prints
// what it saw as one JSON line.
//
//   node web/tests/runtime.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ...,
//     "open": "<the lend's name>", "seeks": [s, s, s] | "film": {"path", "range"},
//     "close": {"path", "range", "after"} | "withhold": true | "direct": true,
//     "again": true,
//     "other": {"lend": "<label>", "path": "<file>", "range": "bytes=a-b"}}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fetched, held, inApp, launch, pageAt, play, serve } from './page.mjs';

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
    return {
      status: res.status,
      contentRange: res.headers.get('content-range'),
      bytes: bytes.length,
      sha,
    };
  } catch (e) {
    return { error: String(e) };
  }
}

// In the lend's page: `close.path`'s range read whole, with the window's
// channel closed once `close.after` datagrams have crossed it
// (`__svrngsCloseWebrtc`, shim.js) — a wire-side count, so the close
// lands mid-film however far the page's reading runs behind. The fallback
// case also observes the actual close; time from close to EOF is diagnostic.
async function closedFetch({ path, range, after, fallback }) {
  const src = document.querySelector('video')?.currentSrc || document.querySelector('video')?.src;
  try {
    const signal = AbortSignal.timeout(fallback ? 30_000 : 60_000);
    globalThis.__svrngsCloseWebrtc = after;
    const res = await fetch(new URL(path, location.href), { headers: { range }, signal });
    const reader = res.body.getReader();
    const parts = [];
    let bytes = 0;
    for (;;) {
      const { value, done } = await reader.read();
      if (done) break;
      parts.push(value);
      bytes += value.byteLength;
    }
    const closed = globalThis.__closedFilmChannel;
    const fallbackMs = closed ? Math.round(performance.now() - closed.at) : null;
    const all = new Uint8Array(bytes);
    let at = 0;
    for (const part of parts) {
      all.set(part, at);
      at += part.byteLength;
    }
    const sha = [...new Uint8Array(await crypto.subtle.digest('SHA-256', all))]
      .map((x) => x.toString(16).padStart(2, '0'))
      .join('');
    return {
      src: src ?? null,
      status: res.status,
      type: res.headers.get('content-type'),
      contentRange: res.headers.get('content-range'),
      bytes,
      sha,
      closed: closed ? { wireBytes: closed.wireBytes, state: closed.channel.readyState } : null,
      fallbackMs,
    };
  } catch (e) {
    return { src: src ?? null, error: String(e) };
  }
}

const server = await serve(args.site);
const origin = pageAt(server);
// WebKit's context keeps a profile, so the browser can be closed and
// opened again on it; Chromium's is stopped through DevTools.
const profile =
  args.engine === 'webkit' ? mkdtempSync(join(tmpdir(), 'svrngs-webkit-')) : undefined;
let { context, close } = await launch(args.engine, { profile });
const logs = [];
const watch = (context) => context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  watch(context);
  if (args.fallback)
    await context.addInitScript(() => {
      const create = RTCPeerConnection.prototype.createDataChannel;
      const close = RTCPeerConnection.prototype.close;
      const channels = new WeakMap();
      RTCPeerConnection.prototype.createDataChannel = function (...args) {
        const channel = create.apply(this, args);
        const record = { channel, wireBytes: 0 };
        channels.set(this, record);
        channel.addEventListener('message', (event) => (record.wireBytes += event.data.byteLength));
        return channel;
      };
      RTCPeerConnection.prototype.close = function () {
        const record = channels.get(this);
        const open = record?.channel.readyState === 'open';
        close.call(this);
        if (open && this.signalingState === 'closed')
          globalThis.__closedFilmChannel = { ...record, at: performance.now() };
      };
    });
  if (args.withhold) await context.addInitScript(() => (globalThis.__svrngsNoWebrtc = true));
  let page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(page, origin, args);
  const listed = await page.evaluate(() =>
    [...document.querySelectorAll('#lent button')].map((b) => b.textContent),
  );
  const button = page.locator('#lent button', { hasText: args.open });
  const away = inApp;
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

  // The direct path's line in the worker's trace, waited for when asked
  // (`args.direct`, webrtc-page): the film's own fetch follows it, so the
  // film rides the data connection, not the relay. The trace is read just
  // after that fetch, before the worker is stopped below: the last lines a
  // run writes may not have been kept yet when it goes, and WebKit's
  // browser opens again without this context's init script (the plant's
  // flag would be lost).
  const traced = () => page.evaluate(() => fetch('svrngs-trace').then((r) => r.text()));
  if (args.direct) {
    const by = Date.now() + 20_000;
    while (Date.now() < by && !(await traced()).includes('goes direct'))
      await new Promise((r) => setTimeout(r, 250));
  }
  // The film played (`play`, the playback step's), or its range fetched;
  // `args.close` fetches it with the window's channel closed part-way
  // through (`closedFetch`, webrtc-page).
  const look = (page) => {
    if (args.seeks) return page.evaluate(play, { at: null, within: 15_000 });
    if (args.close) return page.evaluate(closedFetch, { ...args.close, fallback: args.fallback });
    return page.evaluate(fetched, args.film);
  };
  const first = await look(page);
  // `args.again`: the page reloaded once its first look is done, so the
  // shim announces itself again, as the phone's page did back on the WiFi
  // (demo/runs/phone-home/2026-11-20, "What it found" 1); the direct
  // path's second line waited for, and the film's range fetched again.
  let again = null;
  if (args.again) {
    await page.reload();
    await page.waitForSelector('video', { timeout: 60_000 });
    const by = Date.now() + 20_000;
    const twice = async () => (await traced()).split('goes direct').length > 2;
    while (Date.now() < by && !(await twice())) await new Promise((r) => setTimeout(r, 250));
    again = await page.evaluate(fetched, args.film);
  }
  const trace = args.direct || args.withhold ? await traced() : null;
  const seeks = [];
  for (const at of args.seeks?.slice(0, -1) ?? [])
    seeks.push(await page.evaluate(play, { at, within: 15_000 }));
  // Except in the fallback scene, stop the worker; the next range must still
  // answer. Chromium stops it through DevTools, the page left playing.
  // Playwright cannot reach WebKit's worker, so WebKit's browser is closed
  // and opened again, as a phone's is when it ends a browser left in the
  // background: no worker outlives it, and the page opens again at the
  // lend's address, where the worker it starts serves it.
  let reopened = null;
  let replayed = null;
  if (!args.fallback && cdp) {
    await cdp.send('ServiceWorker.stopAllWorkers');
    const stopped = Date.now();
    while (!runs.includes('stopped') && Date.now() - stopped < 10_000)
      await new Promise((r) => setTimeout(r, 50));
  } else if (!args.fallback) {
    await close();
    ({ context, close } = await launch(args.engine, { profile }));
    watch(context);
    page = context.pages()[0] ?? (await context.newPage());
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    await page.goto(opened);
    await page.waitForSelector('video', { timeout: 60_000 });
    reopened = page.url();
    replayed = await look(page);
  }
  const runsAtStop = [...runs];
  if (args.seeks) seeks.push(await page.evaluate(play, { at: args.seeks.at(-1), within: 15_000 }));
  // Chromium's worker, stopped with the page left open: the film's range
  // asked again starts it again, as a seek's would.
  else if (cdp && !args.fallback) replayed = await look(page);
  const other = args.other ? await page.evaluate(range, args.other) : null;
  console.log(
    JSON.stringify({
      listed,
      opened,
      reopened,
      replayed,
      first,
      again,
      seeks,
      runsAtStop,
      runs,
      other,
      trace,
      logs,
    }),
  );
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
  if (profile) rmSync(profile, { recursive: true, force: true });
}
