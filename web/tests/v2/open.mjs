// The browser half of page-open's v2-only tests (crates/svrngs/tests/
// page_v2/open.rs): the group's link opened in `engine` as a desktop, on the
// page in `site`. What the page offered once #computer shows, what pressing
// #copy wrote and said, every policy violation and page error, and the
// trace (`window.__lines`).
//
//   node web/tests/v2/open.mjs '{"engine": "chromium", "site": "<dir>", "token": "<token>"}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { launch, serve } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
const DESKTOP = { chromium: 'Desktop Chrome', webkit: 'Desktop Safari' };
const { context, close } = await launch(args.engine, { device: DESKTOP[args.engine] });
const errors = [];
try {
  if (args.engine === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin });
  // Each violation of the page's one policy (web/index.html:51), as the
  // browser tells the page, from before its first script.
  await context.addInitScript(() => document.addEventListener('securitypolicyviolation', (e) => (window.__violations ??= []).push(`${e.violatedDirective}: ${e.blockedURI} ${e.sample}`)));
  const page = await context.newPage();
  page.on('pageerror', (e) => errors.push(String(e)));
  await page.goto(`${origin}#${args.token}`);
  await page.waitForSelector('#computer:not([hidden])', { timeout: 90_000 });
  const line = await page.locator('#line').textContent();
  const words = await page.locator('#computer p:not([hidden])').textContent();
  // What the page hands the clipboard, kept as it passes (web/tests/computer.mjs).
  await page.evaluate(() => {
    const write = navigator.clipboard.writeText.bind(navigator.clipboard);
    navigator.clipboard.writeText = (text) => write((window.__written = text));
  });
  await page.click('#copy');
  await page.waitForFunction(() => document.getElementById('copy').textContent !== 'Copy', null, { timeout: 10_000 });
  const seen = await page.evaluate(() => ({ button: document.getElementById('copy').textContent, written: window.__written, violations: window.__violations ?? [], lines: window.__lines, ui: window.__ui }));
  const copied = await page.evaluate(() => navigator.clipboard.readText()).catch((e) => `unread: ${e}`);
  console.log(JSON.stringify({ origin, line, words, copied, errors, ...seen }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${errors.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
