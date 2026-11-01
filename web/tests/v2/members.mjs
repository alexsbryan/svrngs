// The browser half of page-members' v2-only tests (crates/svrngs/tests/
// page_v2/members.rs): the group's page (`site`, `key`, `token`, held as
// page.mjs `driving` holds it) or the home page (`home`, its address),
// opened until `shown`, a selector, matches; then each of `press`, a
// selector, clicked in turn; then, within `within` ms, until `gone`'s
// matches are gone, or `said`, words, are in `at`'s text. It answers what
// `shown` matched first, whether the page got there (`went`), the text of
// `at` then, the page's reloads (0: it drew again in place), and the
// trace.
//
//   node web/tests/v2/members.mjs '{"engine": ..., "shown": <selector>, "press"?: [<selector>, ...], "gone"?: <selector>, "said"?: <words>, "at"?: <selector>, "within": <ms>, "home"?: <url> | "site": ..., "key": ..., "token": ...}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { driving, launch } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const at = args.at ?? 'body';
async function watch(page, reloads) {
  await page.locator(args.shown).first().waitFor({ timeout: 60_000 });
  const shown = await page.locator(args.shown).allTextContents();
  for (const press of args.press ?? []) await page.locator(press).first().click({ timeout: 30_000 });
  const there = ([gone, said, at]) => (gone ? !document.querySelector(gone) : document.querySelector(at)?.innerText.includes(said));
  const went = await page.waitForFunction(there, [args.gone, args.said, at], { timeout: args.within, polling: 100 }).then(() => true, () => false);
  const text = await page.locator(at).first().innerText().catch(() => null);
  return { shown, went, text, reloads: reloads(), lines: await page.evaluate(() => window.__lines ?? []) };
}
// The page's loads from now on.
const counted = (page, n = { loads: 0 }) => (page.on('load', () => n.loads++), () => n.loads);

if (args.home) {
  const { context, close } = await launch(args.engine);
  const logs = [];
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    const loads = counted(page);
    await page.goto(args.home);
    console.log(JSON.stringify({ ...(await watch(page, () => loads() - 1)), logs }));
  } catch (e) {
    console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
    process.exitCode = 1;
  } finally {
    await close();
  }
} else {
  // `driving` has opened the page by its link by now, so each load from here is a reload.
  await driving(args, ({ page }) => watch(page, counted(page)));
}
