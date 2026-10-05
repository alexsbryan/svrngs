// U4's dry run, a phone's half (dry.sh): what opening a group's link costs
// in Playwright's headless Chromium and its WebKit as an iPhone
// (web/tests/page.mjs), cold — a fresh browser, nothing cached, no key.
//
//   node drive.mjs serve <site dir>     serve the built page; prints `port <n>`
//   node drive.mjs run '{"engine": "chromium"|"webkit", "link": "<origin>#<token>",
//     "name": "<the person's name>", "lend": "<what the list's button says>",
//     "seeks": [s, ...]}'
//
// `run` opens the link, types the name when the page asks, and waits for
// the list; opens the link again; opens the lend from the list, plays its
// first <video> and seeks it to each of `seeks`. It prints one JSON line:
// each step's milliseconds, and every response the page's origin sent
// before the list, with its bytes. A step that did not happen within its
// time is `null` with what the page showed — never a number guessed.
import { launch, serve } from '../../../../web/tests/page.mjs';

const [mode, arg] = process.argv.slice(2);
if (mode === 'serve') {
  // Each answer's path and bytes, a line each after the port: dry.sh sums
  // what the origin sent during each run, the service worker's fetches too.
  const server = await serve(arg);
  server.on('request', (req, res) => {
    const end = res.end;
    res.end = function (body, ...rest) {
      console.log(`${res.statusCode} ${req.url} ${body?.length ?? 0}`);
      return end.call(this, body, ...rest);
    };
  });
  console.log(`port ${server.address().port}`);
} else if (mode !== 'run') {
  console.error('usage: node drive.mjs serve <dir> | run <json>');
  process.exit(2);
} else {
  await run(JSON.parse(arg));
}

// In the lend's page: play its <video>, or seek it to `at`, and answer how
// long from then until the element says `seeked` (a seek), until the first
// frame the browser presents from there (`requestVideoFrameCallback`), and
// until it plays on — runtime.mjs's measure, u4-6's seek bar: not seeking,
// not paused, a quarter second past where it started.
async function play({ at, within }) {
  const v = document.querySelector('video');
  if (!v) return { error: `no <video>; the page says: ${document.body.innerText.slice(0, 200)}` };
  const t0 = performance.now();
  const ms = () => Math.round(performance.now() - t0);
  const out = { at, t: Math.round(performance.timeOrigin + t0), seeked_ms: null, frame_ms: null, plays_ms: null };
  if (at !== null) v.addEventListener('seeked', () => (out.seeked_ms ??= ms()), { once: true });
  if ('requestVideoFrameCallback' in v) {
    const from = at ?? 0;
    const frame = (_, m) => {
      if (Math.abs(m.mediaTime - from) < 2.5) out.frame_ms ??= ms();
      else v.requestVideoFrameCallback(frame);
    };
    v.requestVideoFrameCallback(frame);
  } else {
    out.frame_ms = 'unsupported';
  }
  if (at === null) {
    v.muted = true;
    v.play().catch((e) => (out.error = String(e)));
  } else {
    v.currentTime = at;
  }
  const from = at ?? 0;
  return new Promise((done) => {
    const check = () => {
      if (!v.seeking && !v.paused && v.currentTime > from + 0.25) out.plays_ms ??= ms();
      if (out.plays_ms !== null && out.frame_ms !== null) return done(out);
      if (performance.now() - t0 > within) {
        return done({ ...out, stood: v.currentTime, seeking: v.seeking, paused: v.paused, error: out.error ?? v.error?.message ?? null });
      }
      setTimeout(check, 10);
    };
    check();
  });
}

async function run(args) {
  const { context, close } = await launch(args.engine);
  const logs = [];
  const fetched = [];
  const out = { engine: args.engine, browser: context.browser()?.version() ?? null };
  const origin = new URL(args.link).origin;
  try {
    const page = await context.newPage();
    page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    // What failed, by its address: WebKit's console names none.
    page.on('requestfailed', (r) => logs.push(`failed: ${r.url()}: ${r.failure()?.errorText}`));
    page.on('response', async (r) => {
      const url = new URL(r.url());
      if (url.origin !== origin || url.pathname.includes('/l/')) return;
      const bytes = await r.body().then((b) => b.length, () => null);
      fetched.push({ path: url.pathname, status: r.status(), bytes, worker: r.fromServiceWorker() });
    });
    const shown = () => page.evaluate(() => document.getElementById('status').textContent);
    const timed = async (what, step, within = 120_000) => {
      const t0 = Date.now();
      try {
        await step(within);
        return Date.now() - t0;
      } catch (e) {
        out.stopped = `${what}: ${e.message.split('\n')[0]}; the page says: ${await shown().catch(() => '?')}`;
        return null;
      }
    };
    const settled = (within) =>
      page.waitForFunction(
        () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
        null,
        { timeout: within },
      );
    const listed = (within) => page.waitForSelector('#status.good, #status.bad', { timeout: within });

    // Cold: the link opened in a browser that never loaded the page.
    out.open_to_name_ms = await timed('the name asked', async (w) => {
      await page.goto(args.link);
      await settled(w);
      if (await page.evaluate(() => document.getElementById('join').hidden)) throw new Error('the page did not ask a name');
    });
    if (out.open_to_name_ms === null) return;
    await page.fill('#name', args.name);
    out.name_to_list_ms = await timed('the list', async (w) => {
      await page.click('#join button');
      await listed(w);
    });
    out.status = await shown();
    out.lent = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
    out.fetched = [...fetched];
    if (out.name_to_list_ms === null || !/good/.test(await page.getAttribute('#status', 'class'))) return;

    // Again: the link opened once more, the page cached and the key kept.
    out.again_to_list_ms = await timed('the list again', async (w) => {
      await page.goto('about:blank');
      await page.goto(args.link);
      await listed(w);
    });

    // The lend: its address under the worker's scope, then its <video>.
    const button = page.locator('#lent button', { hasText: args.lend });
    out.open_lend_ms = await timed('the lend', async (w) => {
      await Promise.all([page.waitForURL(/\/l\//, { timeout: w }), button.click()]);
      await page.waitForSelector('video', { timeout: w });
    });
    if (out.open_lend_ms === null) return;
    out.first = await page.evaluate(play, { at: null, within: 30_000 });
    out.seeks = [];
    for (const at of args.seeks) out.seeks.push(await page.evaluate(play, { at, within: 30_000 }));
  } catch (e) {
    out.stopped = String(e);
  } finally {
    out.logs = logs.slice(-40);
    console.log(JSON.stringify(out));
    await close();
  }
}
