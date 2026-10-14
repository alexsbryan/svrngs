// The browser half of events-links (a)'s test (crates/svrngs-node/tests/
// browser): a link a lent app writes into itself, opened on a phone. The
// page, served at `args.port` — the group's page the test keeps for it —
// holds a joined group, opens the link and then the lend `args.open` from
// its list; at the lend's origin its page's `#link` is read, the link the
// app wrote from the door's headers, or null when it wrote none. The link
// is opened in a new tab, as a person taps one in a chat, and lands
// wherever the browser takes it, which the test judges.
//
//   node web/tests/links.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "port": N, "key": ..., "token": ..., "open": "<the lend's name>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { held, launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site, { port: args.port });
const origin = `http://localhost:${server.address().port}/`;
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(page, origin, args);
  const away = (url) => url.hostname.endsWith('.localhost') && !url.pathname.endsWith('/open.html');
  await Promise.all([page.waitForURL(away, { timeout: 90_000 }), page.locator('#lent button', { hasText: args.open }).click()]);
  await page.waitForFunction(() => document.title === 'the app', null, { timeout: 90_000 }).catch(() => {});
  const link = await page.evaluate(() => document.getElementById('link')?.getAttribute('href') ?? null);
  let landed = null;
  if (link) {
    const tab = await context.newPage();
    tab.on('pageerror', (e) => logs.push(`tab pageerror: ${e}`));
    await tab.goto(link);
    await tab.waitForFunction(() => document.title === 'the app at a path', null, { timeout: 90_000 }).catch(() => {});
    landed = { url: tab.url(), title: await tab.title() };
  }
  console.log(JSON.stringify({ opened: page.url(), link, landed, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
