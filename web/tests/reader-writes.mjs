// The browser half of crates/svrngs-node/tests/browser/carried.rs
// `a_carried_app_that_read_friends_lends_writes_on_a_phone` (svrn-docs
// rings/current/CARRIED_APP_READS.md §5 item 5): on the group's page, the
// carried app `args.open`, whose reads.txt names a kind a friend lends, is
// opened from the list, the yes. At its origin it is told what it reads
// (`told`), reads the first of those at `args.ask` (`read`), and saves an
// entry (`saved`). On the group's page the yes's Stop is pressed; the app's
// page still open saves again (`held`); then the app is opened at
// `#open=<args.label>`, as its origin asks for another pass, told what it
// reads (`after`), and saves once more (`stopped`).
//
//   node web/tests/reader-writes.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the app's name>", "label": "<its label>", "ask": "<path>"}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent; 1 on anything else.
import { driving, inApp } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const away = (u) => inApp(u) && !u.pathname.endsWith('/open.html');
const ready = (page) =>
  page.waitForFunction(() => document.title === 'ready', null, { timeout: 90_000 });
const told = (page) =>
  page.evaluate(() =>
    fetch('/svrngs-reads').then(async (r) => ({ status: r.status, body: await r.text() })),
  );
const read = (page, ask) =>
  page.evaluate(async (ask) => {
    const { lends } = await fetch('/svrngs-reads').then((r) => r.json());
    if (!lends?.length) return { status: 'none told' };
    const r = await fetch(`${lends[0].address}${ask}`);
    return { status: r.status, body: (await r.text()).slice(0, 80) };
  }, ask);
const save = (page, what) => page.evaluate((w) => window.save(w), what);

await driving(args, async ({ page, context, origin }) => {
  const seen = {};
  try {
    await Promise.all([
      page.waitForURL(away, { timeout: 90_000 }),
      page.locator('#lent button', { hasText: args.open }).click({ timeout: 90_000 }),
    ]);
    await ready(page);
    seen.told = await told(page);
    seen.read = await read(page, args.ask);
    seen.saved = await save(page, 'after a read');
    const group = await context.newPage();
    await group.goto(origin);
    const line = group.locator('#acts p', { hasText: 'as you. Since' });
    await line.waitFor({ state: 'attached', timeout: 90_000 });
    await group.locator('#acts summary').click({ timeout: 10_000 });
    await line.locator('button', { hasText: 'Stop' }).click({ timeout: 10_000 });
    await group
      .locator('#acts p', { hasText: 'within 12 hours, when' })
      .waitFor({ timeout: 30_000 });
    seen.held = await save(page, 'after the Stop, its pass held');
    await group.goto('about:blank');
    await Promise.all([
      group.waitForURL(away, { timeout: 90_000 }),
      group.goto(`${origin}#open=${args.label}`),
    ]);
    await ready(group);
    seen.after = await told(group);
    seen.stopped = await save(group, 'after the Stop, a new pass');
  } catch (e) {
    seen.error = `the page did not finish: ${e}`;
  }
  return seen;
});
