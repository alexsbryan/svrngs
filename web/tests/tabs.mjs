// The browser half of groups-tabs' test (crates/svrngs-node/tests/browser;
// ralph/REVIEW_FINDINGS.md #176): the group's page open in two tabs of one
// browser, its one person's key from that browser's store, as a person
// opens their group twice. Tab one opens the link and lists the group; tab
// two then opens it too. For `args.for` ms after, each tab in turn asks
// "What apps use" of the lend that asks, so each dials its lender through
// its page's own endpoint, as a person's tap there does. Each answer is
// kept with its tab, its time since tab two opened, and its words; with
// each tab's heading at the start and the end, and its trace. When tab two
// then offers to open the group here, it is pressed (groups-tabs (b)), and
// what each tab says after, with tab two's next ask, is kept as `taken`.
//
//   node web/tests/tabs.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "for": <ms>}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { held, launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
const out = { asks: [] };
const watchdog = setTimeout(() => {
  console.error(`the tabs hung after ${JSON.stringify(out)}\n${logs.join('\n')}`);
  process.exit(1);
}, args.for + 240_000);

// What a tab says: its heading, whether good, and all its words.
const said = (page) =>
  page.evaluate(() => ({
    status: document.getElementById('status').textContent,
    good: document.getElementById('status').className === 'good',
    text: document.body.innerText,
  }));

// "What apps use", closed and opened again in `page`: its first line, or
// null when the tab shows no such list.
async function asked(page) {
  if (await page.locator('#uses').isHidden()) return null;
  await page.evaluate(() => (document.getElementById('uses').open = false));
  await page.waitForTimeout(200);
  await page.evaluate(() => document.getElementById('used').replaceChildren());
  await page.locator('#uses summary').click();
  await page.waitForSelector('#used li', { timeout: 30_000 });
  return page.locator('#used').innerText();
}

try {
  const one = await context.newPage();
  one.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(one, origin, args);
  out.one = await said(one);
  const two = await context.newPage();
  two.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await two.goto(`${origin}#${args.token}`);
  await two.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const began = Date.now();
  out.two = await said(two);
  while (Date.now() - began < args.for) {
    for (const [tab, page] of [['one', one], ['two', two]]) {
      const at = Date.now() - began;
      const words = await asked(page).catch((e) => `no line: ${e}`);
      out.asks.push({ tab, at, ms: Date.now() - began - at, words });
    }
    await one.waitForTimeout(2_000);
  }
  out.oneAfter = await said(one);
  out.twoAfter = await said(two);
  // Taken over: tab two's "Open it here", when it offers one; then tab
  // one, opened again, says what it says, and tab two asks once.
  const here = two.locator('#door button', { hasText: 'Open it here' });
  if (await here.isVisible()) {
    await here.click();
    await two.waitForSelector('#status.good:has-text("You’re in"), #status.bad', { timeout: 90_000 });
    await one.waitForFunction(() => /another tab/.test(document.getElementById('status')?.textContent), null, { timeout: 30_000 }).catch(() => {});
    out.taken = { one: await said(one), two: await said(two), ask: await asked(two).catch((e) => `no line: ${e}`) };
  }
  out.lines = { one: await one.evaluate(() => window.__lines), two: await two.evaluate(() => window.__lines) };
  console.log(JSON.stringify({ ...out, logs }));
} catch (e) {
  console.error(`the tabs did not finish: ${e}\n${JSON.stringify(out)}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  clearTimeout(watchdog);
  await close();
  server.close();
}
