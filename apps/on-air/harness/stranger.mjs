// The stranger: a headless Chromium on the page, one tab per link, each
// until it says "ended", "cannot reach" or another word it stops on, or
// until --for seconds pass. Prints one JSON line per link: what the page
// decided (`window.air`), and the console's errors.
//
//   node stranger.mjs --for <seconds> <link>...
//
// Exits 3, naming the install, when Playwright or its Chromium is absent.
import { parseArgs } from 'node:util';

const { values: o, positionals: links } = parseArgs({ options: { for: { type: 'string', default: '120' } }, allowPositionals: true });
let chromium;
try {
  ({ chromium } = await import(new URL('../../../web/node_modules/playwright/index.mjs', import.meta.url).href));
} catch (e) {
  console.log(JSON.stringify({ error: `no Playwright: ${e.message}`, install: 'npm ci --prefix web && npx --prefix web playwright install chromium' }));
  process.exit(3);
}
const browser = await chromium.launch({ args: ['--autoplay-policy=no-user-gesture-required'] }).catch((e) => {
  console.log(JSON.stringify({ error: `no Chromium: ${e.message.split('\n')[0]}`, install: 'npx --prefix web playwright install chromium' }));
  process.exit(3);
});
const STOPS = new Set(['ended', 'cannot reach', 'cannot play', 'cannot tune in']);
const until = Date.now() + Number(o.for) * 1000;

await Promise.all(links.map(async (link) => {
  const page = await browser.newPage();
  const errors = [];
  // A line not yet written is a 404 the page waits through: counted, not listed.
  let waited = 0;
  page.on('console', (m) => m.type() === 'error' && (/status of 404/.test(m.text()) ? waited++ : errors.push(m.text())));
  page.on('pageerror', (e) => errors.push(e.message));
  const opened = Date.now();
  await page.goto(link);
  let air = null;
  while (Date.now() < until) {
    air = await page.evaluate(() => window.air && JSON.parse(JSON.stringify({ ...window.air, currentTime: document.getElementById('audio').currentTime }))).catch((e) => ({ status: `unread: ${e.message}` }));
    if (air && STOPS.has(air.status)) break;
    await page.waitForTimeout(250);
  }
  const stopped = air && STOPS.has(air.status) ? air.status : `still "${air?.status}" after ${o.for} s`;
  console.log(JSON.stringify({ link, opened, stopped, said_at: Date.now(), air, errors, waited_404s: waited }));
  await page.close();
}));
await browser.close();
