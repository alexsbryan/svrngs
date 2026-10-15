// The browser half of v1ui-shelf (c)'s phone test (crates/svrngs-node/tests/
// browser, `the_phone_brings_things_to_the_table`): the group's page,
// holding the group `args.id` joined by `args.token`, brings the shelf's
// ticked apps through the computer it reaches, by the one button, and the
// page's words are read once it says what happened.
//
//   node web/tests/shelf.mjs '{"engine": "chromium"|"webkit", "site": ..., "key": ..., "token": ..., "id": "<the group's id>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { held, launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(page, origin, args);
  const before = await page.locator('#shelf').innerText();
  await page.locator('#shelf button').click({ timeout: 30_000 });
  await page.waitForFunction(() => /You brought|already|Nothing was brought/.test(document.getElementById('shelf').innerText), null, { timeout: 60_000 });
  console.log(JSON.stringify({ before, after: await page.locator('#shelf').innerText(), logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
