// The browser half of V1.1.3's phone test (crates/svrngs-node/tests/
// browser `a_phone_opens_a_lend_that_answers_only_in_gzip`): the page holds
// a joined group, opens the lend `args.open` from its list and waits for
// its page's title, `args.title`. A computer that stops answering, and a
// socket opened again once it answers again, are Node laws now (their
// rows in docs/seams.tsv).
//
//   node web/tests/night.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the lend's name>",
//     "title": "<its page's title>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { driving } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const shown = async (page) => ({
  url: page.url(),
  title: await page.title(),
  text: await page.evaluate(() => document.body.innerText),
});
await driving(args, async ({ page }) => {
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  await page.locator('#lent button', { hasText: args.open }).click();
  await page
    .waitForFunction((t) => document.title === t, args.title, { timeout: 60_000 })
    .catch(() => {});
  return { opened: await shown(page) };
});
