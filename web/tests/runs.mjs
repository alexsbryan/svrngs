// The browser half of canon-pin (c)'s phone test (crates/svrngs-node/tests/
// browser, `a_new_version_runs_as_each_person_only_after_their_yes`): the
// group's page, holding the group joined by `args.token`, says in its shelf
// that someone changed `args.open`'s files; it opens `args.open` from its
// list and reads its title; then, on the group's page again, it says yes by
// the shelf's button, and opens `args.open` again and reads its title.
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
await driving(args, async ({ page, origin }) => {
  // The shelf's words once it asks, or as they are when it never does.
  await page.locator('#shelf p', { hasText: 'Open the new version?' }).waitFor({ timeout: 60_000 }).catch(() => {});
  const asked = await page.locator('#shelf').innerText();
  if (!asked.includes('Open the new version?')) return { asked };
  const before = await opened(page, origin);
  await page.goto(origin);
  await page.locator('#shelf button', { hasText: 'Yes, use the new version' }).click({ timeout: 60_000 });
  // Said once written, or why not beside the button; read either way.
  await page.waitForFunction(() => document.getElementById('shelf').innerText.includes('new version for you now'), null, { timeout: 60_000 }).catch(() => {});
  const said = await page.locator('#shelf').innerText();
  return { asked, before, said, after: await opened(page, origin) };
});
