// The browser half of capability-what-acts's tests (crates/svrngs-node/
// tests/browser/acts.rs): "What acts as you", and its Stop.
//
// With `args.home`, on a computer: the reading lend opened from the home
// page by its button reading `args.words`, the yes, its tab fetching
// `args.read` at `opened`; the home page opened again and its section read,
// `before`; the yes's Stop pressed, and the section read again, `after`;
// then a page of the reading lend, `args.page`, fetching `args.read` at
// `stopped`.
//
// Else on a phone, the group's page holding the group `args.token` joins:
// the lend `args.open` opened from its list, the yes, and what its page is
// told it reads, `yes`; the group's page again, its section read,
// `before`; the yes's Stop pressed, and the line read, `stopped`; then the
// page opened at `#open=<args.label>`, as the lend's origin asks for
// another pass, and what its page is told it reads, `after`.
//
//   node web/tests/acts.mjs '{"engine": "chromium"|"webkit", "home": "<url>", "page": ..., "read": ..., "words": ...}'
//   node web/tests/acts.mjs '{"engine": "chromium"|"webkit", "site": ..., "key": ..., "token": ..., "open": "<the lend's name>", "label": "<its label>"}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent; 1 on anything else.
import { driving, launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const section = (page) => page.locator('#acts').innerText({ timeout: 60_000 });
// A fetch from `page` of `to`: svrngs's refusal is readable to the page.
const fetched = (page, to) =>
  page.evaluate((u) => fetch(u).then(async (r) => ({ status: r.status, refused: r.headers.get('x-svrngs-refused'), error: (await r.json().catch(() => ({}))).error ?? null }), (e) => ({ status: `${e}` })), to);

if (args.home) {
  const { context, close } = await launch(args.engine);
  const seen = {};
  try {
    const home = await context.newPage();
    const loaded = () => home.locator('#home p.address').waitFor({ timeout: 60_000 });
    await home.goto(args.home);
    await loaded();
    const [tab] = await Promise.all([context.waitForEvent('page'), home.locator('#home table.index button', { hasText: args.words }).click({ timeout: 10_000 })]);
    await tab.waitForLoadState();
    seen.opened = await fetched(tab, `${args.read}opened`);
    await home.reload();
    await loaded();
    seen.before = await section(home);
    await Promise.all([home.waitForEvent('load'), home.locator('#acts p', { hasText: 'as you. Since' }).locator('button', { hasText: 'Stop' }).click({ timeout: 10_000 })]);
    await loaded();
    seen.after = await section(home);
    const page = await context.newPage();
    await page.goto(args.page);
    seen.stopped = await fetched(page, `${args.read}stopped`);
  } catch (e) {
    seen.error = `the page did not finish: ${e}`;
  }
  await close();
  console.log(JSON.stringify(seen));
} else {
  // What the lend's page, once open at its origin, is told it reads.
  const told = async (page) => {
    await page.waitForURL((u) => u.hostname.endsWith('.localhost') && !u.pathname.endsWith('/open.html'), { timeout: 90_000 });
    await page.waitForLoadState('load');
    return page.evaluate(() => fetch('/svrngs-reads').then(async (r) => ({ status: r.status, body: await r.text() })));
  };
  await driving(args, async ({ page, origin }) => {
    const seen = {};
    try {
      await page.locator('#lent button', { hasText: args.open }).click({ timeout: 90_000 });
      seen.yes = await told(page);
      await page.goto(origin);
      await page.locator('#acts p', { hasText: 'as you. Since' }).waitFor({ timeout: 90_000 });
      seen.before = await section(page);
      const line = page.locator('#acts p', { hasText: 'as you. Since' });
      await line.locator('button', { hasText: 'Stop' }).click({ timeout: 10_000 });
      await page.locator('#acts p', { hasText: 'within 12 hours, when' }).waitFor({ timeout: 30_000 });
      seen.stopped = await section(page);
      await page.goto('about:blank');
      await page.goto(`${origin}#open=${args.label}`);
      seen.after = await told(page);
    } catch (e) {
      seen.error = `the page did not finish: ${e}`;
    }
    return seen;
  });
}
