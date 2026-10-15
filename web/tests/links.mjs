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
import { driving, launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
await driving(args, async ({ page, context, logs }) => {
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
  // The same link on a computer whose browser keeps no group: its person
  // joined with svrngs, not in this browser (events-links (c)). What the
  // group's page says and the link it shows to copy; then, as the person,
  // that link pasted on the computer's home page, `args.home`, under "Open
  // a link from your group", and where the tab it opens lands.
  let computer = null;
  if (link && args.home) {
    const desk = await launch('chromium');
    try {
      const tab = await desk.context.newPage();
      await tab.goto(link);
      await tab.waitForSelector('#status.good, #status.bad', { timeout: 90_000 }).catch(() => {});
      const said = await tab.evaluate(() => document.getElementById('status').textContent);
      const shown = await tab.evaluate(() => (document.getElementById('computer').hidden ? null : document.getElementById('line').textContent));
      await tab.goto(args.home);
      const box = tab.locator('#open input');
      let opened = null;
      if (shown && (await box.waitFor({ timeout: 30_000 }).then(() => true, () => false))) {
        await box.fill(shown);
        const [lent] = await Promise.all([desk.context.waitForEvent('page'), tab.locator('#open button').click()]);
        await lent.waitForFunction(() => document.title === 'the app at a path', null, { timeout: 90_000 }).catch(() => {});
        opened = { url: lent.url(), title: await lent.title() };
      }
      computer = { said, shown, opened };
    } finally {
      await desk.close();
    }
  }
  return { opened: page.url(), link, landed, computer };
});
