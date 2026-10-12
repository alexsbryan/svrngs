// Bo's Chromium in the llama-server run (run.py): open the lend's address
// on Bo's computer and say what llama-server's own page did through it —
// its title, the server's requests and their statuses — as one JSON line.
//
//   node browser.mjs <origin>
import { chromium } from '../../../../web/node_modules/playwright/index.mjs';

const [origin] = process.argv.slice(2);
const browser = await chromium.launch();
const page = await browser.newPage();
const asked = [];
page.on('response', (r) => {
  const u = new URL(r.url());
  if (u.origin === new URL(origin).origin) asked.push({ path: u.pathname, status: r.status() });
});
const errors = [];
page.on('pageerror', (e) => errors.push(String(e)));
const t0 = Date.now();
let opened;
try {
  const r = await page.goto(origin, { waitUntil: 'networkidle', timeout: 30000 });
  opened = { status: r?.status() ?? null, ms: Date.now() - t0 };
} catch (e) {
  opened = { status: null, why: String(e).slice(0, 200), ms: Date.now() - t0 };
}
const title = await page.title().catch(() => null);
const text = (await page.locator('body').innerText().catch(() => '')).slice(0, 200);
console.log(JSON.stringify({ origin, opened, title, text, asked, errors }));
await browser.close();
