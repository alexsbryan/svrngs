// The browser half of mnp-2's test (crates/svrngs-node/tests/browser): a
// lent page told the lends it reads, and one opened at a path. The page
// holds a joined group, opens the link and then the lend `args.open` from
// its list; at that lend's origin its page asks `/svrngs-reads` (lend.js
// `told`), and the answer is kept with the pass and holder key its origin's
// store keeps, for the test to show neither is in it. The page asks the
// first lend read at the address it was told, `args.ask` under it; then it
// is navigated to `args.go` under that address, and lands wherever the
// browser takes it, which the test judges; then Back once.
//
//   node web/tests/reads.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the lend's name>", "ask": "<path>",
//     "go": "<path>", "title": "<the title of the page it lands on>"}'
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
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(page, origin, args);
  const said = await page.evaluate(() => document.getElementById('response').textContent);
  // At the lend's origin, its start page gone.
  const away = (url) => url.hostname.endsWith('.localhost') && !url.pathname.endsWith('/open.html');
  await Promise.all([page.waitForURL(away, { timeout: 90_000 }), page.locator('#lent button', { hasText: args.open }).click()]);
  await page.waitForLoadState('load');
  const opened = page.url();
  // What the lend's page is told, and what its origin keeps beside it.
  const told = await page.evaluate(async (label) => {
    const res = await fetch('/svrngs-reads');
    const kept = await new Promise((done) => {
      const req = indexedDB.open('svrngs-lend');
      req.onsuccess = () => {
        const get = req.result.transaction('kept').objectStore('kept').get(`pass ${label}`);
        get.onsuccess = () => done(get.result ?? null);
      };
    });
    return { status: res.status, refused: res.headers.get('x-svrngs-refused'), body: await res.text(), pass: kept?.pass, holder: kept?.holder };
  }, new URL(opened).hostname.split('.')[0]);
  const address = JSON.parse(told.body).lends?.[0]?.address;
  const asked = address
    ? await page.evaluate((url) => fetch(url).then(async (r) => ({ status: r.status, body: await r.text() })), `${address}${args.ask}`)
    : null;
  // Navigated to an address it was told, as a link there is followed.
  let landed = null;
  let backAt = null;
  if (address) {
    await Promise.all([
      page.waitForURL((u) => !u.href.startsWith(opened), { timeout: 90_000 }).catch(() => {}),
      page.evaluate((url) => location.assign(url), `${address}${args.go}`),
    ]);
    await page.waitForFunction((t) => document.title === t, args.title, { timeout: 90_000 }).catch(() => {});
    landed = { url: page.url(), title: await page.title() };
    await page.goBack({ waitUntil: 'commit', timeout: 30_000 }).catch((e) => logs.push(`back: ${e}`));
    await page.waitForLoadState('load').catch(() => {});
    backAt = page.url();
  }
  console.log(JSON.stringify({ said, opened, told, asked, landed, backAt, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
