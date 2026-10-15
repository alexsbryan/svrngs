// The browser half of v1ui-shelf (a)'s floor test (crates/svrngs-node/
// tests/browser, `a_carried_page_reaches_no_other_host`): a carried app's
// page opened at its origin on a computer, `args.url`. The page itself
// tries to reach another host each way the floor closes (COMMUNITY_APPS
// §6.2) — a fetch, an image, a script, a style, a socket — then titles
// itself `tried`; the Rust half counts what reached that host. What the
// page's console said is kept: a refusal by the policy is said there.
// With `do` (v1ui-shelf (b), the apps' pages), once the page has drawn
// what it read, each [selector, value?] in turn: filled with the value,
// else clicked and the page's words read once it has drawn them again.
//
//   node web/tests/carried.mjs '{"engine": "chromium"|"webkit", "url": "<the app's origin>",
//     "title"?: "<its title once done>", "do"?: [[<selector>, <value>?], ...]}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await page.goto(args.url);
  if (args.title) await page.waitForFunction((t) => document.title === t, args.title, { timeout: 30_000 });
  const words = () => page.locator('body').innerText();
  // Drawn once its first words are not the ones it says while it reads.
  if (args.do) await page.waitForFunction(() => !document.body.innerText.includes('…'), null, { timeout: 30_000 });
  const after = [];
  for (const [selector, value] of args.do ?? []) {
    if (value !== undefined) {
      await page.locator(selector).first().fill(value);
      continue;
    }
    const before = await words();
    await page.locator(selector).first().click();
    await page.waitForFunction((b) => document.body.innerText !== b, before, { timeout: 30_000 });
    after.push(await words());
  }
  console.log(JSON.stringify({ title: await page.title(), said: await words(), after, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
