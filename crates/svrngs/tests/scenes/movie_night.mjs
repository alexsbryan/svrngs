// Ana's browser for the Movie Night scene (movie_night.rs): a computer's
// Chromium, every response it receives written to `dump`
// (apps/jellyfin/check/person.mjs `record`). Ana opens her home page by
// the address her svrngs opened it at, waits for Movie Night's button
// there, `opener`, and presses it, which posts her yes in a tab of its
// own; there she searches `film`, waits until every library has said
// what it found (app.js `WORDS`), and follows the first copy's link,
// until the page there says `landed`.
//
//   node movie_night.mjs '{"engine", "home", "opener", "film", "landed", "dump"}'
//
// Prints one JSON line: what each page said, or where it stopped and why.
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
let at = 'her home page';
try {
  page.on('pageerror', (e) => seen.errors.push(String(e).slice(0, 300)));
  await page.goto(args.home);
  await arrived(page, 90_000);
  // The home page reads its groups and lends again each second.
  const button = page.getByRole('button', { name: args.opener, exact: true });
  await button.waitFor({ timeout: 60_000 });
  seen.row = await button.locator('xpath=ancestor::td[1]').innerText();

  at = 'Movie Night, opened';
  const [tab] = await Promise.all([
    context.waitForEvent('page', { timeout: 30_000 }),
    button.click(),
  ]);
  tab.on('pageerror', (e) => seen.errors.push(String(e).slice(0, 300)));
  await arrived(tab, 90_000);
  await tab.getByRole('heading', { name: 'Movie Night' }).waitFor({ timeout: 90_000 });
  seen.opened = { url: tab.url(), text: await words(tab) };

  at = 'Movie Night, searched';
  await tab.fill('#q', args.film);
  await tab.getByRole('button', { name: 'Search', exact: true }).click();
  await tab.waitForFunction(
    () => {
      const said = [...document.querySelectorAll('#said li')];
      return said.length > 0 && said.every((l) => l.dataset.state !== 'asking');
    },
    null,
    { timeout: 90_000 },
  );
  seen.searched = {
    said: await tab.locator('#said').innerText(),
    states: await tab.$$eval('#said li', (ls) => ls.map((l) => l.dataset.state)),
    cards: await tab.$$eval('#results > li', (ls) => ls.map((l) => l.innerText)),
  };

  at = 'the copy, opened';
  const copy = tab.locator('#results a.copy.first').first();
  if (await copy.count()) {
    seen.copy = await copy.getAttribute('href');
    await copy.click();
    await tab.waitForFunction((w) => document.body?.innerText.includes(w), args.landed, {
      timeout: 90_000,
    });
    seen.landed = {
      url: tab.url(),
      title: await tab.title(),
      text: (await words(tab)).slice(0, 400),
    };
  }
} catch (e) {
  seen.stopped = { at, error: String(e).split('\n')[0], ms: Date.now() - t0 };
  seen.stopped.text = (await Promise.all(context.pages().map(words)))
    .join('\n---\n')
    .slice(0, 2000);
}
seen.ms = Date.now() - t0;
// The last bodies `record` is still writing, written before it closes.
await new Promise((done) => setTimeout(done, 1000));
await close();
console.log(JSON.stringify(seen));
