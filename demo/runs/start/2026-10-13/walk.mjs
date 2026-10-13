// One state of the home page in the start-measure run (run.py): open `url`
// in Chromium or WebKit, screenshot it to `shot`, and say what the page
// offers its reader — its headings, words, buttons, links and boxes, and
// how often each word of the fence's list and of the run's watch list is
// in its words — as one JSON line. With `click`, press the button (else
// follow the link) of that name after, and say where the page went and what
// it then said.
//
//   node walk.mjs <chromium|webkit> <url> <shot.png> [click]
import { chromium, webkit } from '../../../../web/node_modules/playwright/index.mjs';

const [which, url, shot, click] = process.argv.slice(2);
// The fence's words (ralph/next/start PROMPT, fence 3), then those the run
// watches for besides: each a whole word, case-blind.
const FENCE = ['key', 'member', 'keeper', 'ring', 'act', 'journal', 'port', 'namespace', 'origin'];
const WATCH = ['lend', 'lends', 'lent', 'lending', 'app', 'apps', 'svrngs', 'localhost', 'terminal', 'install', 'link', 'invite', 'phone', 'computer', 'group', 'folder'];
const count = (text, w) => (text.match(new RegExp(`\\b${w}\\b`, 'gi')) ?? []).length;

const browser = await (which === 'webkit' ? webkit : chromium).launch();
const page = await browser.newPage({ viewport: { width: 900, height: 900 } });
const errors = [];
page.on('pageerror', (e) => errors.push(String(e)));
const t0 = Date.now();
let opened;
try {
  const r = await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
  opened = { status: r?.status() ?? null, ms: Date.now() - t0 };
} catch (e) {
  opened = { status: null, why: String(e).split('\n')[0].slice(0, 200), ms: Date.now() - t0 };
}
await page.waitForTimeout(500);
const read = async () => {
  const text = await page.locator('body').innerText().catch(() => '');
  return {
    at: page.url(),
    title: await page.title().catch(() => null),
    text,
    headings: await page.locator('h1, h2, h3').allInnerTexts().catch(() => []),
    buttons: await page.locator('button').allInnerTexts().catch(() => []),
    links: await page.locator('a').evaluateAll((as) => as.map((a) => ({ text: a.innerText, href: a.href }))).catch(() => []),
    boxes: await page.locator('input').evaluateAll((is) => is.map((i) => ({ type: i.type, label: i.ariaLabel, value: i.value }))).catch(() => []),
    fence: Object.fromEntries(FENCE.map((w) => [w, count(text, w)]).filter(([, n]) => n)),
    watch: Object.fromEntries(WATCH.map((w) => [w, count(text, w)]).filter(([, n]) => n)),
  };
};
await page.screenshot({ path: shot, fullPage: true }).catch((e) => errors.push(`screenshot: ${e}`));
const out = { which, url, opened, ...(await read()), shot, errors };
if (click) {
  try {
    const button = page.getByRole('button', { name: click });
    await ((await button.count()) ? button : page.getByRole('link', { name: click })).first().click();
    await page.waitForLoadState('networkidle', { timeout: 15000 }).catch(() => {});
    await page.waitForTimeout(1000);
    out.after = await read();
    await page.screenshot({ path: shot.replace(/\.png$/, '-after.png'), fullPage: true }).catch(() => {});
  } catch (e) {
    out.after = { why: String(e).split('\n')[0].slice(0, 200) };
  }
}
console.log(JSON.stringify(out));
await browser.close();
