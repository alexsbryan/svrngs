// Ana's browser for the taste scene (taste.rs): a computer's Chromium, every
// response it receives written to `dump` (apps/jellyfin/check/person.mjs
// `record`). Ana opens her home page, presses `opener`, which posts her yes
// in a tab of its own, and waits for taste's page to list her friends'
// films. She marks each of `rates` (title and button words), choosing the
// Letterboxd file `letterboxd` after, and reads what the page says. The page
// then asks its own origin for /diary, as any script there could. Last she
// follows `watch`'s Watch link to its Jellyfin, comes back, and answers the
// page's "How was…?" with `after`.
//
//   node taste.mjs '{"engine", "home", "opener", "rates", "letterboxd", "watch", "after", "dump"}'
//
// Prints one JSON line: what the page said, or where it stopped and why.
import { arrived, launch } from '../../../../web/tests/page.mjs';
import { record } from '../../../../apps/jellyfin/check/person.mjs';

const args = JSON.parse(process.argv[2]);
const t0 = Date.now();
const { context, close } = await launch(args.engine);
const page = await context.newPage();
const seen = { errors: [], logs: [] };
context.on('console', (m) => seen.logs.push(`${m.type()}: ${m.text()}`.slice(0, 300)));
record(context, page, 'ana', { DUMP: args.dump, T0: t0 });
const words = (p) => p.evaluate(() => document.body?.innerText ?? '').catch(() => '');
// What each part of taste's page lists, in order.
const lists = (tab) =>
  tab.evaluate(() => {
    const titles = (id) =>
      [...document.querySelectorAll(`#${id}:not([hidden]) li`)].map((li) => li.dataset.title);
    const because = [...document.querySelectorAll('.because')].map((b) => [
      b.closest('li').dataset.title,
      b.textContent,
    ]);
    return {
      likely: titles('likely'),
      unlikely: titles('unlikely'),
      unguessed: titles('unguessed'),
      rated: titles('rated'),
      because,
      guessed: document.getElementById('guessed').textContent,
      heading: document.querySelector('#unguessed h2').textContent,
      said: document.getElementById('said').innerText,
    };
  });
let at = 'her home page';
try {
  page.on('pageerror', (e) => seen.errors.push(String(e).slice(0, 300)));
  await page.goto(args.home);
  await arrived(page, 90_000);
  const button = page.getByRole('button', { name: args.opener, exact: true });
  await button.waitFor({ timeout: 60_000 });
  seen.row = await button.locator('xpath=ancestor::td[1]').innerText();

  at = 'taste, opened';
  const [tab] = await Promise.all([
    context.waitForEvent('page', { timeout: 30_000 }),
    button.click(),
  ]);
  tab.on('pageerror', (e) => seen.errors.push(String(e).slice(0, 300)));
  await arrived(tab, 90_000);
  await tab.waitForSelector('body[data-ready="yes"]', { timeout: 120_000 });
  seen.opened = { url: tab.url(), text: await words(tab), lists: await lists(tab) };

  at = 'taste, rated';
  for (const [title, said] of args.rates) {
    const row = tab.locator(`li[data-title="${title}"]`);
    await row.getByRole('button', { name: said, exact: true }).click();
    // The lists drawn again, her rating among them.
    await tab
      .locator(`#rated li[data-title="${title}"] button[aria-pressed="true"]`)
      .waitFor({ timeout: 60_000 });
  }
  seen.rated = await lists(tab);

  at = 'taste, Letterboxd chosen';
  await tab.setInputFiles('#file', args.letterboxd);
  await tab.waitForFunction(
    () => /also rated/.test(document.getElementById('guessed').textContent),
    null,
    {
      timeout: 60_000,
    },
  );
  seen.brought = await tab.locator('#brought').innerText();
  seen.lists = await lists(tab);
  seen.text = await words(tab);

  at = 'its /diary, asked from its page';
  seen.diary = await tab.evaluate(async () => {
    const r = await fetch('/diary');
    return { status: r.status, refused: r.headers.get('x-svrngs-refused'), body: await r.text() };
  });

  at = 'a film watched, and back';
  const origin = new URL(tab.url()).origin;
  const watch = tab.locator(`#likely li[data-title="${args.watch}"] a.watch`);
  seen.watch = await watch.innerText();
  await watch.click();
  await tab.waitForURL((u) => !u.href.startsWith(origin), { timeout: 60_000 });
  await arrived(tab, 90_000);
  seen.watched = tab.url();
  // Back the way she came; where the opening's steps leave no way back,
  // she opens taste again, which asks as well.
  await tab.goBack().catch(() => null);
  seen.back = tab.url().startsWith(origin) ? 'back' : 'opened again';
  if (seen.back !== 'back') await tab.goto(`${origin}/`);
  await tab.waitForSelector('#after:not([hidden])', { timeout: 60_000 });
  seen.after = await tab.locator('#after').innerText();
  await tab.locator('#after').getByRole('button', { name: args.after, exact: true }).click();
  await tab
    .locator(`#rated li[data-title="${args.watch}"] button[aria-pressed="true"]`)
    .waitFor({ timeout: 60_000 });
  seen.asked_after = await tab.locator('#after').isVisible();
} catch (e) {
  seen.stopped = { at, error: String(e).split('\n')[0], ms: Date.now() - t0 };
  seen.stopped.text = (await Promise.all(context.pages().map(words)))
    .join('\n---\n')
    .slice(0, 3000);
}
seen.ms = Date.now() - t0;
// The last bodies `record` is still writing, written before it closes.
await new Promise((done) => setTimeout(done, 1000));
await close();
console.log(JSON.stringify(seen));
