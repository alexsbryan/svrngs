// The browser half of page-index's v2-only tests (crates/svrngs/tests/
// page_v2/index.rs): the home page at `home`, or the group's page (`site`,
// `key`, `token`, held as page.mjs `driving` holds it), and each of `steps`
// in turn, [selector, value, line, until?]: a value `true` ticks the box,
// another fills it, and null clicks it, then reads once the page has
// traced one more line starting with `line` (ui/machine.js), or has loaded
// again, as v1 does, and drawn "What you lend" again; one neither did in
// 10 s is "unsettled". With `until`, it is read once `read`'s words hold
// it too, or as it stands 30 s on, "unsettled". With `visible`, the page is then
// hidden and shown again, as a person comes back to its tab, and read once
// more after one more "LENDING: answered". It answers each read of
// `read`'s words, the page's loads (1: it never reloaded; on the group's
// page, `driving`'s own load counted), every request it sent with its
// body, each link, and the trace.
//
//   node web/tests/v2/index.mjs '{"engine": ..., "home": <url> | "site": ..., "key": ..., "token": ..., "read": <selector>, "steps": [[<selector>, <value>|true|null, <line>?, <until>?], ...], "visible"?: true}'
//
// Exit 0 with the JSON line; 1 on anything else.
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
  const count = (line) => page.evaluate((l) => (window.__lines ?? []).filter((t) => t.startsWith(l)).length, line).catch(() => 0);
  const read = () => page.locator(args.read).allTextContents();
  // One more line starting with `line`, or a load and the list drawn again.
  async function settled(line, act) {
    const [n, at] = [await count(line), loads];
    await act();
    const traced = page.waitForFunction(([l, k]) => (window.__lines ?? []).filter((t) => t.startsWith(l)).length > k, [line, n], { timeout: 10_000 });
    // Counted here, not in the page: a waitForFunction runs on across a load.
    const reloaded = (async () => {
      for (let i = 0; i < 100 && loads === at; i++) await new Promise((r) => setTimeout(r, 100));
      if (loads === at) throw new Error('not loaded');
      await page.locator(ready).first().waitFor({ timeout: 10_000 });
    })();
    return Promise.race([traced, reloaded]).then(() => 'settled', () => 'unsettled');
  }
  const holds = ([s, w]) => [...document.querySelectorAll(s)].map((e) => e.textContent).join('\n').includes(w);
  const after = [];
  for (const [selector, value, line, until] of args.steps) {
    if (value === true) {
      await page.locator(selector).first().check({ timeout: 30_000 });
      continue;
    }
    if (value !== null) {
      await page.locator(selector).first().fill(value, { timeout: 30_000 });
      continue;
    }
    let how = await settled(line, () => page.locator(selector).first().click({ timeout: 30_000 }));
    if (until) how = await page.waitForFunction(holds, [args.read, until], { timeout: 30_000 }).then(() => how, () => 'unsettled');
    after.push({ how, words: await read() });
  }
  let later = null;
  if (args.visible) {
    const how = await settled('LENDING: answered', () => page.evaluate(() => ['hidden', 'visible'].forEach((v) => {
      Object.defineProperty(document, 'visibilityState', { value: v, configurable: true });
      document.dispatchEvent(new Event('visibilitychange'));
    })));
    later = { how, words: await read() };
  }
  const links = await page.locator('a').evaluateAll((as) => as.map((a) => a.href));
  const lines = await page.evaluate(() => window.__lines ?? null);
  return { after, later, loads, sent, links, lines };
}

if (args.home) {
  const { context, close } = await launch(args.engine);
  const logs = [];
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    const seen = await run(page, '#lent h2 ~ *', 0, () => page.goto(args.home));
    console.log(JSON.stringify({ ...seen, logs }));
  } catch (e) {
    console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
    process.exitCode = 1;
  } finally {
    await close();
  }
} else {
  await driving(args, ({ page }) => run(page, '#shelf button', 1, async () => {}));
}
