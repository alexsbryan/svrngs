// The browser half of crates/svrngs-node/tests/browser/sends.rs
// `a_carried_taste_sends_and_draws_what_comes_back` (row 13;
// apps/taste): on the person's home page, `args.home`, the row of
// the index's button reading `args.words`, its fold opened and its send
// ticked, the Open pressed; in Taste's tab, once it says what it did,
// `said`, the films it lists, `films`; `args.csv` brought as a Letterboxd
// `ratings.csv`, `brought`, and what it asks first, `waiting`; each of
// `args.rates`, a film's title and a rating's words, rated; what it guesses, `likely`, why, `because`, where
// to watch the first, `watch`, and its line on the model, `guessed`; then
// that Watch pressed (the navigation held back), the page opened again,
// `after` its question, answered `args.after`, and what is rated, `rated`.
//
//   node web/tests/taste-sends.mjs '{"engine": "chromium"|"webkit", "home": "<url>",
//     "words": "<its button's words>", "csv": "<a ratings.csv>",
//     "rates": [["<a film's title>", "<a rating's words>"]], "after": "<a rating's words>"}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent.
import { arrived, launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
const seen = {};
try {
  const home = await context.newPage();
  await home.goto(args.home);
  await home.locator('#home table.index button').first().waitFor({ timeout: 60_000 });
  const row = home.locator('#home table.index tr', { hasText: args.words });
  await row.locator('details summary').click({ timeout: 10_000 });
  await row
    .locator('details label', { hasText: 'Sends to' })
    .locator('input[type=checkbox]')
    .check({ timeout: 10_000 });
  const [tab] = await Promise.all([
    context.waitForEvent('page'),
    row.locator('button', { hasText: args.words }).click({ timeout: 10_000 }),
  ]);
  await arrived(tab);
  // Done once it says what it did, or what stopped it.
  const done = () =>
    tab.waitForFunction(
      () => {
        const words = document.getElementById('said')?.textContent ?? '';
        return document.body.dataset.ready === 'yes' && words !== '' && !words.endsWith('…');
      },
      null,
      { timeout: 30_000 },
    );
  const titles = (at) => tab.locator(at).evaluateAll((all) => all.map((li) => li.dataset.title));
  await done();
  seen.said = await tab.locator('#said').textContent();
  seen.films = await titles('li[data-title]');
  await tab.locator('#file').setInputFiles({
    name: 'ratings.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from(args.csv),
  });
  await tab.waitForFunction(() => document.getElementById('brought').textContent !== '');
  seen.brought = await tab.locator('#brought').textContent();
  seen.waiting = await tab.locator('#unguessed h2').textContent();
  for (const [title, words] of args.rates)
    await tab
      .locator(`li[data-title="${title}"]`)
      .locator('button', { hasText: words })
      .click({ timeout: 10_000 });
  seen.likely = await titles('#likely li');
  seen.because = await tab.locator('#likely .because').allTextContents();
  seen.watch = await tab.locator('#likely a.watch').first().getAttribute('href');
  seen.guessed = await tab.locator('#guessed').textContent();
  await tab.evaluate(() => addEventListener('click', (e) => e.preventDefault(), { once: true }));
  await tab.locator('#likely a.watch').first().click({ timeout: 10_000 });
  await tab.reload();
  await done();
  seen.after = await tab.locator('#after h2').textContent();
  await tab.locator('#after button', { hasText: args.after }).click({ timeout: 10_000 });
  seen.rated = await titles('#rated li');
} catch (e) {
  seen.error = `the page did not finish: ${e}`;
}
await close();
console.log(JSON.stringify(seen));
