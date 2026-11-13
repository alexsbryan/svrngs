// The browser half of page-members' tests (crates/svrngs/tests/
// pages/members.rs): the group's page (`site`, `key`, `token`, held as
// page.mjs `driving` holds it), opened until `shown`, a selector,
// matches; then, within `within` ms, until `gone`'s matches are gone, or
// `said`, words, are in `at`'s text. It answers what `shown` matched
// first, whether the page got there (`went`), the text of `at` then, the
// page's reloads (0: it drew again in place), and the trace. Or a guest
// (`visit`, a guest's link's token): the link opened on the page `site`
// serves, in a browser holding nothing, and their name, `name`, given once
// asked; then the same, with the heading as it asked (`asking`).
//
//   node web/tests/pages/members.mjs '{"engine": ..., "shown": <selector>, "gone"?: <selector>, "said"?: <words>, "at"?: <selector>, "within": <ms>, "visit": <token>, "name": ..., "site": ... | "site": ..., "key": ..., "token": ...}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { driving, launch, pageAt, serve } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const at = args.at ?? 'body';
async function watch(page, reloads) {
  await page.locator(args.shown).first().waitFor({ timeout: 60_000 });
  const shown = await page.locator(args.shown).allTextContents();
  const there = ([gone, said, at]) =>
    gone ? !document.querySelector(gone) : document.querySelector(at)?.innerText.includes(said);
  const went = await page
    .waitForFunction(there, [args.gone, args.said, at], {
      timeout: args.within,
      polling: 100,
    })
    .then(
      () => true,
      () => false,
    );
  const text = await page
    .locator(at)
    .first()
    .innerText()
    .catch(() => null);
  return {
    shown,
    went,
    text,
    reloads: reloads(),
    lines: await page.evaluate(() => window.__lines ?? []),
  };
}
// The page's loads from now on.
const counted = (page, n = { loads: 0 }) => (page.on('load', () => n.loads++), () => n.loads);

// The guest's link opened and their name given once asked: the heading as it asked.
async function guest(page, origin) {
  await page.goto(`${origin}#visit=${args.visit}`);
  await page.locator('#join').waitFor({ state: 'visible', timeout: 90_000 });
  const asking = await page.locator('#status').textContent();
  await page.locator('#name').fill(args.name);
  await page.locator('#join button').first().click();
  return asking;
}

if (args.visit) {
  const server = await serve(args.site);
  const { context, close } = await launch(args.engine, { unopened: true });
  const logs = [];
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    const loads = counted(page);
    const asking = await guest(page, pageAt(server));
    console.log(JSON.stringify({ asking, ...(await watch(page, () => loads() - 1)), logs }));
  } catch (e) {
    console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
    process.exitCode = 1;
  } finally {
    await close();
    server.close();
  }
} else {
  // `driving` has opened the page by its link by now, so each load from here is a reload.
  await driving(args, ({ page }) => watch(page, counted(page)));
}
