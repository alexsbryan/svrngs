// The browser half of page-index's v2-only tests of "What you lend"
// (crates/svrngs/tests/page_v2/index.rs): the home page at `home`, each of
// `steps` in turn, [selector, value, line]: filled with the value, else
// clicked, then read once the page has traced one more line starting with
// `line` (ui/machine.js), or has loaded again, as v1 does, and drawn
// "What you lend" again; one neither did in 10 s is read as it stands,
// "unsettled". With `visible`, the page is then hidden and shown again, as
// a person comes back to its tab, and read once more after one more
// "LENDING: answered". It answers each read of `read`'s words, the page's
// loads (1: it never reloaded), every request it sent with its body, each
// link, and the trace.
//
//   node web/tests/v2/index.mjs '{"engine": ..., "home": <url>, "read": <selector>, "steps": [[<selector>, <value>|null, <line>?], ...], "visible"?: true}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { launch } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  let loads = 0;
  const sent = [];
  page.on('load', () => loads++);
  page.on('request', (r) => sent.push([r.method(), r.url(), r.postData() ?? '']));
  await page.goto(args.home);
  await page.locator('#lent h2 ~ *').first().waitFor({ timeout: 30_000 });
  const count = (line) => page.evaluate((l) => (window.__lines ?? []).filter((t) => t.startsWith(l)).length, line).catch(() => 0);
  const read = () => page.locator(args.read).allTextContents();
  // One more line starting with `line`, or a load and the list drawn again.
  async function settled(line, act) {
    const [n, at] = [await count(line), loads];
    await act();
    const traced = page.waitForFunction(([l, k]) => (window.__lines ?? []).filter((t) => t.startsWith(l)).length > k, [line, n], { timeout: 10_000 });
    // Counted here, not in the page: a waitForFunction runs on across a load.
    const loaded = (async () => {
      for (let i = 0; i < 100 && loads === at; i++) await new Promise((r) => setTimeout(r, 100));
      if (loads === at) throw new Error('not loaded');
      await page.locator('#lent h2 ~ *').first().waitFor({ timeout: 10_000 });
    })();
    return Promise.race([traced, loaded]).then(() => 'settled', () => 'unsettled');
  }
  const after = [];
  for (const [selector, value, line] of args.steps) {
    if (value !== null) {
      await page.locator(selector).first().fill(value, { timeout: 30_000 });
      continue;
    }
    const how = await settled(line, () => page.locator(selector).first().click({ timeout: 30_000 }));
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
  console.log(JSON.stringify({ after, later, loads, sent, links, lines, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
