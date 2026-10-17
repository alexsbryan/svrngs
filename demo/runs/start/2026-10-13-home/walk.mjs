// One state of the home page in the start-home walk (run.py), start-measure's
// walk.mjs (../2026-10-13/walk.mjs) with three changes: how long until the
// first group's people are shown (`people_ms`), apart from the whole load;
// a control found by its name, a button else a link, its name a part of
// what it says; and after a click, a tab it opened and, in Chromium, what
// the clipboard holds. Opens `url` in Chromium or WebKit, screenshots it to
// `shot`, says what the page offers its reader as one JSON line, and with
// `click` presses that control and says what the page then said, or that no
// control of that name is on the page.
//
//   node walk.mjs <chromium|webkit> <url> <shot.png> [click]
import { chromium, webkit } from '../../../../web/node_modules/playwright/index.mjs';

const [which, url, shot, click] = process.argv.slice(2);
// The fence's words (ralph/next/start PROMPT, fence 3), then those the run
// watches for besides: each a whole word, case-blind.
const FENCE = ['key', 'member', 'keeper', 'ring', 'act', 'journal', 'port', 'namespace', 'origin'];
const WATCH = ['lend', 'lends', 'lent', 'lending', 'share', 'shares', 'app', 'apps', 'svrngs', 'localhost', 'terminal', 'install', 'link', 'invite', 'phone', 'computer', 'group', 'folder', 'program'];
const count = (text, w) => (text.match(new RegExp(`\\b${w}\\b`, 'gi')) ?? []).length;

const browser = await (which === 'webkit' ? webkit : chromium).launch();
const context = await browser.newContext({ viewport: { width: 900, height: 900 } });
if (which === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']).catch(() => {});
const page = await context.newPage();
const errors = [];
page.on('pageerror', (e) => errors.push(String(e)));
const t0 = Date.now();
let opened;
const people = page.locator('#home section p').first().waitFor({ timeout: 30000 }).then(() => Date.now() - t0, () => null);
try {
  const r = await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
  opened = { status: r?.status() ?? null, ms: Date.now() - t0 };
} catch (e) {
  opened = { status: null, why: String(e).split('\n')[0].slice(0, 200), ms: Date.now() - t0 };
}
opened.people_ms = await people;
await page.waitForTimeout(500);
const read = async (p = page) => {
  const text = await p.locator('body').innerText().catch(() => '');
  return {
    at: p.url(),
    title: await p.title().catch(() => null),
    text,
    headings: await p.locator('h1, h2, h3').allInnerTexts().catch(() => []),
    buttons: await p.locator('button').allInnerTexts().catch(() => []),
    links: await p.locator('a').evaluateAll((as) => as.map((a) => ({ text: a.innerText, href: a.href, target: a.target }))).catch(() => []),
    boxes: await p.locator('input').evaluateAll((is) => is.map((i) => ({ type: i.type, label: i.ariaLabel, value: i.value, labelled: i.labels?.[0]?.innerText ?? null }))).catch(() => []),
    fence: Object.fromEntries(FENCE.map((w) => [w, count(text, w)]).filter(([, n]) => n)),
    watch: Object.fromEntries(WATCH.map((w) => [w, count(text, w)]).filter(([, n]) => n)),
  };
};
await page.screenshot({ path: shot, fullPage: true }).catch((e) => errors.push(`screenshot: ${e}`));
const out = { which, url, opened, ...(await read()), shot, errors };
if (click) {
  try {
    const button = page.getByRole('button', { name: click });
    const control = (await button.count()) ? button : page.getByRole('link', { name: click });
    if (!(await control.count())) throw new Error(`no button or link named “${click}” on the page`);
    out.clicked = await control.first().innerText();
    const tab = context.waitForEvent('page', { timeout: 5000 }).catch(() => null);
    await control.first().click();
    await page.waitForLoadState('networkidle', { timeout: 15000 }).catch(() => {});
    await page.waitForTimeout(1500);
    out.after = await read();
    const opens = await tab;
    if (opens) {
      await opens.waitForLoadState('networkidle', { timeout: 15000 }).catch(() => {});
      out.tab = await read(opens);
    }
    if (which === 'chromium') out.clipboard = await page.evaluate(() => navigator.clipboard.readText()).catch((e) => `unread: ${e}`);
    await page.screenshot({ path: shot.replace(/\.png$/, '-after.png'), fullPage: true }).catch(() => {});
  } catch (e) {
    out.after = { why: String(e).split('\n')[0].slice(0, 200) };
  }
}
console.log(JSON.stringify(out));
await browser.close();
