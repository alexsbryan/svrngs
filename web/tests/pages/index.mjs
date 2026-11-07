// The browser half of page-index's tests (crates/svrngs/tests/
// pages/index.rs): the home page at `home`, on B and that selector's sign
// that it is drawn given as `b`, or the group's page (`site`,
// `key`, `token`, held as page.mjs `driving` holds it), and each of `steps`
// in turn, [selector, value, line, until?]: a value `true` ticks the box,
// another fills it, and null clicks it, then reads once the page has
// traced one more line starting with `line` (ui/machine.js), or has loaded
// again, as v1 does, and drawn "What you lend" again; one neither did in
// 10 s is "unsettled". With `until`, it is read once `read`'s words hold
// it too, or as it stands 30 s on, "unsettled". With `ready`, a file,
// `read`'s words are then read into `shown`, the file written, and a line
// `go` waited for on stdin (drive.rs `release`). With `visible`, [line,
// until?], the page is then hidden and shown again, as a person comes back
// to its tab, and read once more as a step is. With `held`, [[<path>,
// ...], <words>], each request to one of the paths waits, from the page's
// first drawing until the steps are done, then goes on, and `read`'s words
// are read into `released` once they no longer hold the words, or as they
// stand 30 s on, "unsettled": each step is read while what the paths
// answer is held, at no frame of the browser's choosing. It answers each
// read of `read`'s words, the page's loads (1: it never reloaded; on the
// group's page, `driving`'s own load counted), every request it sent with
// its body, each link, and the trace.
//
//   node web/tests/pages/index.mjs '{"engine": ..., "home": <url> | "site": ..., "key": ..., "token": ..., "read": <selector>, "steps": [[<selector>, <value>|true|null, <line>?, <until>?], ...], "ready"?: <file>, "visible"?: [<line>, <until>?], "held"?: [[<path>, ...], <words>]}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { driving, launch } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
// The page gone to by `go` (`loaded` the loads before), `ready` drawn:
// each step, then coming back.
async function run(page, ready, loaded, go) {
  let loads = loaded;
  const sent = [];
  page.on('load', () => loads++);
  page.on('request', (r) => sent.push([r.method(), r.url(), r.postData() ?? '']));
  await go();
  await page.locator(ready).first().waitFor({ timeout: 30_000 });
  let release = () => {};
  // Held at the context, which the home page's session worker fetches
  // through (home/session-worker.js), and the page's own route does not.
  if (args.held) {
    const letGo = new Promise((done) => (release = done));
    await page.context().route(
      (url) => args.held[0].includes(url.pathname),
      (route) => letGo.then(() => route.continue()),
    );
  }
  const count = (line) =>
    page
      .evaluate((l) => (window.__lines ?? []).filter((t) => t.startsWith(l)).length, line)
      .catch(() => 0);
  const read = () => page.locator(args.read).allTextContents();
  // One more line starting with `line`, or a load and the list drawn again.
  async function settled(line, act) {
    const [n, at] = [await count(line), loads];
    await act();
    const traced = page.waitForFunction(
      ([l, k]) => (window.__lines ?? []).filter((t) => t.startsWith(l)).length > k,
      [line, n],
      { timeout: 10_000 },
    );
    // Counted here, not in the page: a waitForFunction runs on across a load.
    const reloaded = (async () => {
      for (let i = 0; i < 100 && loads === at; i++) await new Promise((r) => setTimeout(r, 100));
      if (loads === at) throw new Error('not loaded');
      await page.locator(ready).first().waitFor({ timeout: 10_000 });
    })();
    return Promise.race([traced, reloaded]).then(
      () => 'settled',
      () => 'unsettled',
    );
  }
  const holds = ([s, w]) =>
    [...document.querySelectorAll(s)]
      .map((e) => e.textContent)
      .join('\n')
      .includes(w);
  const until = (how, w) =>
    w
      ? page.waitForFunction(holds, [args.read, w], { timeout: 30_000 }).then(
          () => how,
          () => 'unsettled',
        )
      : how;
  const after = [];
  for (const [selector, value, line, words] of args.steps) {
    if (value === true) {
      await page.locator(selector).first().check({ timeout: 30_000 });
      continue;
    }
    if (value !== null) {
      await page.locator(selector).first().fill(value, { timeout: 30_000 });
      continue;
    }
    let how = await settled(line, () => page.locator(selector).first().click({ timeout: 30_000 }));
    how = await until(how, words);
    after.push({ how, words: await read() });
  }
  let released = null;
  if (args.held) {
    release();
    const lacks = ([s, w]) =>
      ![...document.querySelectorAll(s)]
        .map((e) => e.textContent)
        .join('\n')
        .includes(w);
    const how = await page
      .waitForFunction(lacks, [args.read, args.held[1]], { timeout: 30_000 })
      .then(
        () => 'settled',
        () => 'unsettled',
      );
    released = { how, words: await read() };
  }
  let shown = null;
  if (args.ready) {
    shown = await read();
    const told = createInterface({ input: process.stdin });
    writeFileSync(args.ready, '');
    await new Promise((done) => told.on('line', (l) => l.includes('go') && done()));
    told.close();
  }
  let later = null;
  if (args.visible) {
    const [line, words] = args.visible;
    const how = await settled(line, () =>
      page.evaluate(() =>
        ['hidden', 'visible'].forEach((v) => {
          Object.defineProperty(document, 'visibilityState', { value: v, configurable: true });
          document.dispatchEvent(new Event('visibilitychange'));
        }),
      ),
    );
    later = { how: await until(how, words), words: await read() };
  }
  const links = await page.locator('a').evaluateAll((as) => as.map((a) => a.href));
  const lines = await page.evaluate(() => window.__lines ?? null);
  return { after, released, shown, later, loads, sent, links, lines };
}

if (args.home) {
  const { context, close } = await launch(args.engine, { b: Boolean(args.b) });
  const logs = [];
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    const seen = await run(page, args.b ?? '#lent h2 ~ *', 0, () => page.goto(args.home));
    console.log(JSON.stringify({ ...seen, logs }));
  } catch (e) {
    console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
    process.exitCode = 1;
  } finally {
    await close();
  }
} else {
  await driving(args, ({ page }) => run(page, '#lent tr, #shelf button', 1, async () => {}));
}
