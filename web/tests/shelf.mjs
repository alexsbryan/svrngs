// The browser half of v1ui-shelf (c)'s phone test (crates/svrngs-node/tests/
// browser, `the_phone_brings_things_to_the_table`): the group's page,
// holding the group `args.id` joined by `args.token`, ticks each of the
// shelf's apps and brings them through the computer it reaches, by the one
// button, and the page's words are read once it says what happened.
//
//   node web/tests/shelf.mjs '{"engine": "chromium"|"webkit", "site": ..., "key": ..., "token": ..., "id": "<the group's id>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { driving } from './page.mjs';

const args = JSON.parse(process.argv[2]);
await driving(args, async ({ page }) => {
  const before = await page.locator('#shelf').innerText();
  // Its boxes start unticked (the operator, 2026-10-26): how many were
  // ticked as drawn, then each ticked, as a person ticks them.
  const ticked = await page.locator('#shelf input:checked').count();
  for (const box of await page.locator('#shelf input').all()) await box.check({ timeout: 30_000 });
  await page.locator('#shelf button').click({ timeout: 30_000 });
  await page.waitForFunction(() => /You brought|already|Nothing was brought/.test(document.getElementById('shelf').innerText), null, { timeout: 60_000 });
  return { before, ticked, after: await page.locator('#shelf').innerText() };
});
