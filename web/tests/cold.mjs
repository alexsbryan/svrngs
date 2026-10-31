// The browser half of the browser rows' tests (crates/svrngs-node/tests/
// browser/main.rs, `a_lends_address_opens_cold_in_a_browser_that_joined`
// and `a_lends_alias_opens_it_from_the_groups_page`; svrn-docs
// rings/current/IN_THE_BROWSER.md §4 and "Decided"). The page, served at
// `args.port` when given, holds a joined group and has opened no lend, so
// no lend's worker is in this browser. Each step is asked by its arg:
//
// - `cold`, a lend's label: its address, at `/?n=1`, opened as a bookmark
//   is, and where it lands; then, with the lend's worker taken away again
//   at once, its address opened a second time, `/?n=2`, and where that
//   lands: never sent to the group's page twice;
// - `keep`: the addresses the group's page shows to keep, a row each;
// - `aliases`: each opened on the group's page at `#open=<alias>`, and
//   where it lands.
//
//   node web/tests/cold.mjs '{"engine": ..., "site": ..., "port": N,
//     "key": ..., "token": ..., "cold": "<label>", "keep": true,
//     "aliases": ["app@ada", ...]}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { driving } from './page.mjs';

const args = JSON.parse(process.argv[2]);

// Where `page` lands: a lent page (titled "the app…"), the group's page
// refusing or saying why it opened nothing, or a lend's start saying it
// could not; what it says either way.
async function landed(page) {
  await page
    .waitForFunction(
      () => document.title.startsWith('the app') || document.querySelector('#status.bad') || /[^…]$/.test(document.getElementById('response')?.textContent ?? '') || /[^…]$/.test(document.getElementById('said')?.textContent ?? ''),
      null,
      { timeout: 90_000 },
    )
    .catch(() => {});
  return page.evaluate(() => ({
    url: location.href,
    title: document.title,
    status: document.getElementById('status')?.textContent ?? null,
    said: document.getElementById('response')?.textContent ?? document.getElementById('said')?.textContent ?? null,
  }));
}

await driving(args, async ({ page, origin }) => {
  const seen = {};
  const port = new URL(origin).port;
  if (args.cold) {
    const at = `http://${args.cold}.localhost:${port}`;
    await page.goto(`${at}/?n=1`);
    seen.cold = await landed(page);
    // The lend's worker gone again, as when a browser clears it: the page
    // host answers its address once more, within the minute.
    await page.evaluate(() => navigator.serviceWorker?.getRegistrations().then((rs) => Promise.all(rs.map((r) => r.unregister()))));
    await page.goto(`${at}/?n=2`);
    seen.again = await landed(page);
  }
  if (args.keep) {
    await page.goto('about:blank');
    await page.goto(origin);
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    await page.locator('#lent tr').first().waitFor({ timeout: 90_000 });
    seen.keep = await page.evaluate(() => [...document.querySelectorAll('#lent tr')].map((r) => r.querySelector('small')?.textContent ?? null));
    seen.index = await page.evaluate(() => [...document.querySelectorAll('#lent tr')].map((r) => r.textContent));
  }
  seen.aliases = [];
  for (const alias of args.aliases ?? []) {
    await page.goto('about:blank');
    await page.goto(`${origin}#open=${alias}`);
    seen.aliases.push({ alias, ...(await landed(page)) });
  }
  return seen;
});
