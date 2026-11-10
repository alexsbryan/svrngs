// The browser half of crates/svrngs-node/tests/browser/sends.rs
// `a_carried_taste_sends_and_draws_what_comes_back` (row 13;
// apps/taste): on the person's home page, `args.home`, the row of
// the index's button reading `args.words`, its fold opened and its send
// ticked, the Open pressed; in Taste's tab, once it says what it did,
// `said`, the films it lists, `films`; then `args.rate` rated `args.as`,
// and what it guesses, `likely`.
//
//   node web/tests/taste-sends.mjs '{"engine": "chromium"|"webkit", "home": "<url>",
//     "words": "<its button's words>", "rate": "<a film's title>", "as": "<a rating's words>"}'
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
  const said = tab.locator('#said');
  // Done once it says what it did, or what stopped it.
  await tab.waitForFunction(
    () => {
      const words = document.getElementById('said')?.textContent ?? '';
      return words !== '' && !words.endsWith('…');
    },
    null,
    { timeout: 30_000 },
  );
  seen.said = await said.textContent();
  const films = tab.locator('#films li');
  seen.films = await films.evaluateAll((all) => all.map((li) => li.dataset.title));
  await tab
    .locator(`#films li[data-title="${args.rate}"]`)
    .locator('button', { hasText: args.as })
    .click({ timeout: 10_000 });
  seen.likely = await tab.locator('#likely li').allTextContents();
} catch (e) {
  seen.error = `the page did not finish: ${e}`;
}
await close();
console.log(JSON.stringify(seen));
