// The browser half of crates/svrngs-node/tests/browser (DESIGN §8, U4): it
// serves the built page on 127.0.0.1, opens it in Playwright's headless
// `engine` (page.mjs), and prints what the page did as one JSON line. The
// Rust half owns the relay and the node, so every byte stays on this
// computer.
//
//   node web/tests/drive.mjs '{"engine": "chromium"|"webkit", "site": "<dir>",
//     "key": "<64 hex>", "token": "<token>"}'
//
// First it leaves the page while it loads, and opens it again.
//
// With `"http": {...}` in the args, once the page is in it also speaks HTTP
// over the dial to the lends named there ([`httpChecks`], u4-4), and holds
// its upload's tail until a line `go` arrives on stdin: the Rust half sends
// it when the service has the upload's first byte.
//
// Every address off this computer is refused it, and listed as `away`
// (relay-default (b)): n0 unreachable, as a phone's page must not need it.
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent — the browser step fails then, never skips; 1 on
// anything else.
import { hold, launch, offline, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);

// In the page: HTTP over the dial through the node's door, by the page's own
// `connect` and `fetch` (web/src/lib.rs), imported as app.js imports them.
// Each answer is read as a page reads one, chunk by chunk.
async function httpChecks(a) {
  const app = document.querySelector('script[type=module]').src;
  const m = await import(new URL(`./wasm/svrngs_web.js${new URL(app).search}`, app).href);
  await m.default();
  const key = Uint8Array.from(a.key.match(/../g), (h) => parseInt(h, 16));
  const d = await m.connect(key, a.token);
  const hex = (b) => [...new Uint8Array(b)].map((x) => x.toString(16).padStart(2, '0')).join('');
  // A body to its end: how many chunks, the largest, the bytes, the SHA-256.
  async function drain(res) {
    const chunks = [];
    const reader = res.body.getReader();
    for (let r = await reader.read(); !r.done; r = await reader.read()) chunks.push(r.value);
    const whole = new Uint8Array(chunks.reduce((n, c) => n + c.length, 0));
    chunks.reduce((at, c) => (whole.set(c, at), at + c.length), 0);
    const sha = hex(await crypto.subtle.digest('SHA-256', whole));
    return { chunks: chunks.length, largest: Math.max(0, ...chunks.map((c) => c.length)), bytes: whole.length, sha };
  }
  const answer = async (res) => ({
    status: res.status,
    contentRange: res.headers.get('content-range'),
    ...(await drain(res)),
  });
  // Each step's answer, or why it failed or hung: one step's failure never
  // keeps the next from being judged.
  const step = (run) =>
    Promise.race([
      run(),
      new Promise((_, no) => setTimeout(() => no(new Error('no answer within 60 s')), 60_000)),
    ]).catch((e) => ({ error: String(e) }));
  const out = {};
  out.range = await step(async () => answer(await d.fetch('GET', a.range.path, [['range', a.range.header]], null)));
  out.past = await step(async () => answer(await d.fetch('GET', a.past.path, [['range', a.past.header]], null)));

  // A chunked answer that never ends: read until `want` bytes, then leave.
  out.chunked = await step(async () => {
    const tap = await d.fetch('GET', a.chunked.path, [], null);
    const reader = tap.body.getReader();
    let text = '';
    while (text.length < a.chunked.want) {
      const r = await reader.read();
      if (r.done) break;
      text += new TextDecoder().decode(r.value);
    }
    await reader.cancel();
    return { status: tap.status, text };
  });

  // An upload made as it is read, `chunk` bytes a pull; past `ahead` bytes
  // the page makes no more until the computer has the first.
  const { size, chunk, ahead } = a.post;
  let made = 0;
  let held = null;
  const released = new Promise((go) => {
    window.__release = go;
  });
  let stall = null;
  const stalled = () =>
    new Promise((_, no) =>
      setTimeout(() => {
        stall = `the computer had no byte of the upload while the page held ${made} bytes of it`;
        no(new Error(stall));
      }, 30_000),
    );
  const body = new ReadableStream(
    {
      async pull(c) {
        if (made >= size) return c.close();
        if (made >= ahead && held === null) {
          await Promise.race([released, stalled()]);
          held = made;
        }
        const n = Math.min(chunk, size - made);
        const b = new Uint8Array(n);
        for (let i = 0; i < n; i++) b[i] = ((made + i) * 7 + ((made + i) >> 10)) & 255;
        made += n;
        c.enqueue(b);
      },
    },
    { highWaterMark: 0 },
  );
  const headers = [['content-length', String(size)], ['content-type', 'application/octet-stream']];
  out.post = await step(async () => {
    // A stalled body fails the request with hyper's words; the stall's are
    // the reason.
    const posted = await d.fetch('POST', a.post.path, headers, body).catch((e) => {
      throw new Error(stall ?? String(e));
    });
    return { status: posted.status, text: await posted.text(), held };
  });
  await d.close();
  return out;
}

const server = await serve(args.site);
const origin = `http://127.0.0.1:${server.address().port}/`;
const { context, close } = await launch(args.engine);
const away = await offline(context);
const logs = [];
try {
  const page = await context.newPage();
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  // A phone's back and reopen (u4-7): the page left while it loads, three
  // times, then opened again. WebKit wedged the tab when app.js awaited
  // the wasm at its module's top (ralph/REVIEW_FINDINGS.md).
  try {
    for (let i = 0; i < 3; i++) {
      await page.goto(origin, { timeout: 15_000 });
      await page.goto('about:blank', { timeout: 15_000 });
    }
    await page.goto(origin, { timeout: 15_000 });
  } catch (e) {
    throw new Error(`the page, left while it loaded, did not open again: ${e}`);
  }
  await page.evaluate(hold, { hex: args.key, token: args.token });
  await page.goto('about:blank');
  await page.goto(`${origin}#${args.token}`);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const result = await page.evaluate(() => ({
    status: document.getElementById('status').textContent,
    answer: document.getElementById('response').textContent,
    lent: [...document.querySelectorAll('#lent button')].map((b) => b.textContent),
    removals: [...document.querySelectorAll('#removals li')].map((l) => l.textContent),
    lines: window.__lines,
  }));
  if (args.http) {
    process.stdin.on('data', (line) => {
      if (String(line).includes('go')) page.evaluate(() => window.__release()).catch(() => {});
    });
    result.http = await page.evaluate(httpChecks, { key: args.key, token: args.token, ...args.http });
    result.lines = await page.evaluate(() => window.__lines);
  }
  console.log(JSON.stringify({ ...result, away, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\naway: ${away.join(' ')}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
  // stdin, read while the Rust half holds it open, would keep this process
  // alive; exiting instead could cut the JSON line short, as a pipe's
  // writes on macOS are asynchronous.
  if (args.http) process.stdin.destroy();
}
