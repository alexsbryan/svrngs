// The browser half of v1-onboarding (c)'s test (crates/svrngs-node/tests/
// browser, `the_page_offers_a_computer_one_line_and_sends_the_token_nowhere`):
// the group's link opened on a computer, `engine` as a desktop, and again
// on a phone, as an iPhone. On the computer the page offers the line that
// gets svrngs (DESIGN §7), composed from the link's `#` part, and a button
// that copies it; on the phone it offers none. Every request either page
// makes, the service worker's included, is kept, and the page host's own
// answer to `/` as a chat's preview reads it. With an empty token, the
// page opened with no link, which offers a founder's line (start-group).
//
//   node web/tests/computer.mjs '{"engine": "chromium"|"webkit", "site": "<dir>", "token": "<token>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
const link = `${origin}#${args.token}`;
const DESKTOP = { chromium: 'Desktop Chrome', webkit: 'Desktop Safari' };
const logs = [];
const requests = [];

// Open the link as `device`; what the page offered, once it has said
// whether it is in, and what copying the line put on the clipboard.
async function opened(device) {
  const { context, close } = await launch(args.engine, { device });
  try {
    if (args.engine === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin });
    context.on('request', (r) => requests.push({ url: r.url(), headers: r.headers(), body: r.postData() }));
    context.on('console', (m) => logs.push(`${device} ${m.type()}: ${m.text()}`));
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`${device} pageerror: ${e}`));
    await page.goto(link);
    await page.waitForSelector('#status.good, #status.bad, #join:not([hidden])', { timeout: 90_000 });
    const offered = await page.locator('#computer').isVisible();
    let [line, words] = [null, null];
    let [copied, written, button] = [null, null, null];
    if (offered) {
      line = await page.locator('#line').textContent();
      // What the page says the line does, a joiner's or a founder's (start-group).
      words = await page.locator('#computer p:not([hidden])').textContent();
      // What the page hands the clipboard, kept as it passes: WebKit
      // refuses a test's read of the clipboard (NotAllowedError), so there
      // the copy is read from this and the button's word alone.
      await page.evaluate(() => {
        const write = navigator.clipboard.writeText.bind(navigator.clipboard);
        navigator.clipboard.writeText = (text) => write((window.__written = text));
      });
      await page.click('#copy');
      await page.waitForFunction(() => document.getElementById('copy').textContent !== 'Copy', null, { timeout: 10_000 });
      button = await page.locator('#copy').textContent();
      written = await page.evaluate(() => window.__written);
      copied = await page.evaluate(() => navigator.clipboard.readText()).catch((e) => `unread: ${e}`);
    }
    return { offered, line, words, written, copied, button, status: await page.locator('#status').textContent() };
  } finally {
    await close();
  }
}

try {
  const computer = await opened(DESKTOP[args.engine]);
  const phone = await opened('iPhone 15');
  const served = await (await fetch(origin)).text();
  const asked = server.asked;
  console.log(JSON.stringify({ origin, computer, phone, served, requests, asked, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  server.close();
}
