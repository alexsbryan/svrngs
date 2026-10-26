// The browser half of give-session's test (crates/svrngs-node/tests/
// browser/opens.rs, `an_open_leads_home_then_to_a_lend_and_a_bookmark_opens_each_again`):
// a browser svrngs never opened, at the home page, `args.home`, and at a
// lend's origin, `args.lend`, as `fresh_home` and `fresh_lend`; then
// `args.open`, the address `svrngs open` hands the browser, as `opened`;
// the lend opened from the home page's index by its link naming
// `args.words`, as `lend`; and each again by its bookmark, typed, as
// `home_again` and `lend_again`.
//
//   node web/tests/open.mjs '{"engine": "chromium"|"webkit", "home": "<url>",
//     "lend": "<the lend's origin>", "open": "<svrngs open's address>", "words": "<its link's words>"}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine, { unopened: true });
const said = async (page) => ({ url: page.url(), title: await page.title(), text: (await page.locator('body').innerText()).slice(0, 400) });
const seen = {};
try {
  const page = await context.newPage();
  for (const [key, at] of [['fresh_home', args.home], ['fresh_lend', args.lend]]) {
    await page.goto(at);
    seen[key] = await said(page);
  }
  await page.goto(args.open);
  await page.locator('#home table.index a', { hasText: args.words }).first().waitFor({ timeout: 60_000 });
  seen.opened = page.url();
  const [tab] = await Promise.all([context.waitForEvent('page'), page.locator('#home table.index a', { hasText: args.words }).first().click()]);
  await tab.waitForLoadState();
  seen.lend = await said(tab);
  for (const [key, at] of [['home_again', args.home], ['lend_again', args.lend]]) {
    await tab.goto(at);
    seen[key] = await said(tab);
  }
} catch (e) {
  seen.error = `the page did not finish: ${e}`;
}
await close();
console.log(JSON.stringify(seen));
