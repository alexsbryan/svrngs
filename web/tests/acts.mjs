// The browser half of capability-what-acts's tests (crates/svrngs-node/
// tests/browser/acts.rs): "What acts as you", and its Stop.
//
// On a phone, the group's page holding the group `args.token` joins:
// the lend `args.open` opened from its list, the yes, and what its page is
// told it reads, `yes`; the group's page again, its section read,
// `before`; the yes's Stop pressed, and the line read, `stopped`; then the
// page opened at `#open=<args.label>`, as the lend's origin asks for
// another pass, and what its page is told it reads, `after`.
//
//   node web/tests/acts.mjs '{"engine": "chromium"|"webkit", "site": ..., "key": ..., "token": ..., "open": "<the lend's name>", "label": "<its label>"}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent; 1 on anything else.
import { driving, inApp } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const section = (page) => page.locator('#acts').innerText({ timeout: 60_000 });
// "What acts as you" folded under its heading, then opened by it, as a
// person opens it: whether it was folded.
const unfold = async (page) => {
  const folded = await page
    .locator('#acts details')
    .evaluate((d) => !d.open, null, { timeout: 60_000 });
  await page.locator('#acts summary').click({ timeout: 10_000 });
  return folded;
};
// What the lend's page, once open at its origin, is told it reads.
const told = async (page) => {
  await page.waitForURL((u) => inApp(u) && !u.pathname.endsWith('/open.html'), {
    timeout: 90_000,
  });
  await page.waitForLoadState('load');
  return page.evaluate(() =>
    fetch('/svrngs-reads').then(async (r) => ({ status: r.status, body: await r.text() })),
  );
};
await driving(args, async ({ page, origin }) => {
  const seen = {};
  try {
    await page.locator('#lent button', { hasText: args.open }).click({ timeout: 90_000 });
    seen.yes = await told(page);
    await page.goto(origin);
    await page
      .locator('#acts p', { hasText: 'as you. Since' })
      .waitFor({ state: 'attached', timeout: 90_000 });
    seen.folded = await unfold(page);
    seen.before = await section(page);
    const line = page.locator('#acts p', { hasText: 'as you. Since' });
    await line.locator('button', { hasText: 'Stop' }).click({ timeout: 10_000 });
    await page
      .locator('#acts p', { hasText: 'within 12 hours, when' })
      .waitFor({ timeout: 30_000 });
    seen.stopped = await section(page);
    await page.goto('about:blank');
    await page.goto(`${origin}#open=${args.label}`);
    seen.after = await told(page);
  } catch (e) {
    seen.error = `the page did not finish: ${e}`;
  }
  return seen;
});
