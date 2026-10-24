// The browser half of canon-pin (c)'s phone test (crates/svrngs-node/tests/
// browser, `a_new_version_runs_as_each_person_only_after_their_yes`): the
// group's page, holding the group joined by `args.token`, says under "What
// acts as you" that someone changed `args.open`'s files; it opens
// `args.open` from its list and reads its title; then, on the group's page
// again, it says yes by its button, and opens `args.open` again and reads
// its title.
//
//   node web/tests/runs.mjs '{"engine": "chromium"|"webkit", "site": ..., "key": ..., "token": ..., "id": "<the group's id>", "open": "<the app's name>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { driving } from './page.mjs';

const args = JSON.parse(process.argv[2]);
// The app opened from the group's page at `origin`, once the page lists
// it, and its title once its script has set one.
const opened = async (page, origin) => {
  await page.goto(origin);
  await page.locator('#lent button', { hasText: args.open }).click({ timeout: 60_000 });
  await page.waitForFunction((o) => document.title.startsWith(`${o} `), args.open, { timeout: 60_000 }).catch(() => {});
  return page.title();
};
// "1 app changed. See what’s new" opened, as a person opens it.
const unfolded = (page) => page.locator('#acts summary', { hasText: 'See what’s new' }).click({ timeout: 60_000 });
await driving(args, async ({ page, origin }) => {
  // The section's words once it asks, or as they are when it never does.
  await unfolded(page).catch(() => {});
  const asked = await page.locator('#acts').innerText();
  if (!asked.includes('changed its files on')) return { asked };
  const before = await opened(page, origin);
  await page.goto(origin);
  await unfolded(page);
  await page.locator('#acts button', { hasText: 'Open the new version' }).click({ timeout: 60_000 });
  // Said once written, or why not beside the button; read either way.
  await page.waitForFunction(() => document.getElementById('acts').innerText.includes('new version for you now'), null, { timeout: 60_000 }).catch(() => {});
  const said = await page.locator('#acts').innerText();
  return { asked, before, said, after: await opened(page, origin) };
});
