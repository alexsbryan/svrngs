// The browser half of page-members' tests (crates/svrngs/tests/
// pages/members.rs): the group's page (`site`, `key`, `token`, held as
// page.mjs `driving` holds it) or the home page (`home`, its address),
// opened until `shown`, a selector, matches; then each of `press`, a
// selector, clicked in turn; then, within `within` ms, until `gone`'s
// matches are gone, or `said`, words, are in `at`'s text. It answers what
// `shown` matched first, whether the page got there (`went`), the text of
// `at` then, the page's reloads (0: it drew again in place), and the
// trace. Or a guest (`visit`, a guest's link's token): the link opened on
// the page `site` serves, in a browser holding nothing, and their name,
// `name`, given once asked; then the same, with the heading as it asked
// (`asking`). With `forget`, the session's key is dropped before the
// presses, as a session lapses (page.mjs `lapse`); with `hold`, the page is hidden and shown
// again once the test, told it is shown, says so (`watch`). With `never`,
// words, each change after `shown` that leaves them in `at`'s text is
// counted (`never`, 0 when none did).
//
//   node web/tests/pages/members.mjs '{"engine": ..., "shown": <selector>, "press"?: [<selector> | [<selector>, <gone>], ...], "gone"?: <selector>, "said"?: <words>, "at"?: <selector>, "traced"?: [<start>, <n>], "within": <ms>, "forget"?: true, "hold"?: <path>, "never"?: <words>, "home"?: <url> | "visit": <token>, "name": ..., "site": ... | "site": ..., "key": ..., "token": ...}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { existsSync } from 'node:fs';
import { writeFile } from 'node:fs/promises';
import { driving, lapse, launch, serve } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const at = args.at ?? 'body';
async function watch(page, reloads) {
  await page.locator(args.shown).first().waitFor({ timeout: 60_000 });
  const shown = await page.locator(args.shown).allTextContents();
  if (args.never) await page.evaluate(([at, w]) => ((window.__never = 0), new MutationObserver(() => document.querySelector(at)?.textContent.includes(w) && window.__never++).observe(document.body, { subtree: true, childList: true, characterData: true })), [at, args.never]);
  // With `forget`, the page's session forgotten before any press, as one lapses.
  if (args.forget) await lapse(page);
  // A press `[selector, gone]` is waited on, before the next, until `gone` is gone: whether it went, and `at`'s text then.
  const waited = [];
  for (const [press, gone] of (args.press ?? []).map((p) => [p].flat())) {
    await page.locator(press).first().click({ timeout: 30_000 });
    if (gone) waited.push({ went: await page.waitForFunction((g) => !document.querySelector(g), gone, { timeout: args.within, polling: 100 }).then(() => true, () => false), text: await page.locator(at).first().innerText() });
  }
  // With `hold`, a path: `<hold>.shown` written, then, once the test has
  // written `hold`, the page hidden and shown again (home.mjs `visible`).
  if (args.hold) {
    await writeFile(`${args.hold}.shown`, '');
    for (let t = 0; !existsSync(args.hold); t += 100) await (t > 60_000 ? Promise.reject(new Error(`${args.hold} never written`)) : new Promise((r) => setTimeout(r, 100)));
    for (const v of ['hidden', 'visible']) await page.evaluate((v) => (Object.defineProperty(document, 'visibilityState', { value: v, configurable: true }), document.dispatchEvent(new Event('visibilitychange'))), v);
  }
  // With `traced`, [start, n]: until the trace holds n lines that start so, too.
  const there = ([gone, said, at, [start, n]]) => (gone ? !document.querySelector(gone) : document.querySelector(at)?.innerText.includes(said)) && (window.__lines ?? []).filter((l) => !start || l.startsWith(start)).length >= (n ?? 0);
  const went = await page.waitForFunction(there, [args.gone, args.said, at, args.traced ?? []], { timeout: args.within, polling: 100 }).then(() => true, () => false);
  const text = await page.locator(at).first().innerText().catch(() => null);
  return { shown, waited, went, text, never: await page.evaluate(() => window.__never ?? null), reloads: reloads(), lines: await page.evaluate(() => window.__lines ?? []) };
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

if (args.home || args.visit) {
  const server = args.visit && (await serve(args.site));
  const { context, close } = await launch(args.engine, { unopened: Boolean(args.visit) });
  const logs = [];
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    const loads = counted(page);
    const asking = args.visit ? await guest(page, `http://localhost:${server.address().port}/`) : void (await page.goto(args.home));
    console.log(JSON.stringify({ asking, ...(await watch(page, () => loads() - 1)), logs }));
  } catch (e) {
    console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
    process.exitCode = 1;
  } finally {
    await close();
    server?.close();
  }
} else {
  // `driving` has opened the page by its link by now, so each load from here is a reload.
  await driving(args, ({ page }) => watch(page, counted(page)));
}
