// The browser half of lean-4 (c)'s test (crates/svrngs-node/tests/browser):
// the home page on a computer asks svrngs's question. Opened at
// `args.ask`, its `/ask?lend=<label>&app=<label>`, it is read for its
// heading, which the test holds to the one source (question.js).
//
//   node web/tests/home.mjs '{"engine": "chromium"|"webkit", "ask": "<url>", "heading"?: false, "read"?: "<selector>", "all"?: "<selector>", "do"?: [[<selector>, <value>?, <stays>?], ...], "timed"?: [<selector>, ...], "drawn"?: "<selector>", "text"?: true}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
// Chromium reads its clipboard only when granted it (start-home: a lend's
// address copied for a program).
if (args.engine === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  // With `timed`, selectors: how long after the page was asked for each
  // first showed (start-home: a group shown before its slowest computer).
  const t0 = Date.now();
  const timing = Promise.all((args.timed ?? []).map((s) => page.locator(s).first().waitFor({ timeout: 30_000 }).then(() => [s, Date.now() - t0])));
  await page.goto(args.ask);
  // With `heading: false`, a page heading none (an ask not lent).
  const heading = args.heading === false ? null : await page.locator('#home h2').first().textContent({ timeout: 30_000 });
  // With `read`, a selector, its first match's words too (U7: a removed
  // person's line).
  const read = args.read ? await page.locator(args.read).first().textContent({ timeout: 30_000 }) : null;
  // With `all`, every match's words, once the page has listed its groups
  // (v1-keeper-office (b): the removals).
  if (args.all) await page.locator('#home section').first().waitFor({ timeout: 30_000 });
  const all = args.all ? await page.locator(args.all).allTextContents() : null;
  // With `do`, each [selector, value?, stays?] in turn: filled with the
  // value, else clicked, and `all`'s words read once the page it reloads
  // has listed what this computer lends (bash-limit (b)); with `stays`, a
  // click that reloads nothing, read once the page has said what happened
  // (start-home), and in Chromium the clipboard then.
  const after = [];
  let clipboard = null;
  for (const [selector, value, stays] of args.do ?? []) {
    const filled = value !== undefined && value !== null;
    if (filled && !stays) {
      await page.locator(selector).first().fill(value);
      continue;
    }
    // A fill with `stays` is read as a click that stays is: once the page
    // has said what it says as the words are typed (#138).
    if (stays) {
      const before = (await page.locator(args.all).allTextContents()).join('\n');
      await (filled ? page.locator(selector).first().fill(value) : page.locator(selector).first().click());
      await page.waitForFunction(([s, b]) => [...document.querySelectorAll(s)].map((e) => e.textContent).join('\n') !== b, [args.all, before], { timeout: 30_000 });
      after.push(await page.locator(args.all).allTextContents());
      if (args.engine === 'chromium') clipboard = await page.evaluate(() => navigator.clipboard.readText()).catch((e) => `unread: ${e}`);
      continue;
    }
    await Promise.all([page.waitForEvent('load'), page.locator(selector).first().click()]);
    await page.locator('#lent h2').waitFor({ timeout: 30_000 });
    await page.locator('#lent li, #lent p').first().waitFor({ timeout: 30_000 });
    after.push(await page.locator(args.all).allTextContents());
  }
  // With `drawn`, a selector: each match's class and words, as the page
  // drew them once loaded (v1ui-door (c): the door).
  const drawn = args.drawn ? await page.locator(args.drawn).evaluateAll((es) => es.map((e) => [e.className, e.textContent])) : null;
  // With `text`, the page's words as a person reads them, each cell apart.
  const text = args.text ? await page.locator('body').innerText() : null;
  // Every link on the page, its words and where it opens (start-home).
  const links = await page.locator('a').evaluateAll((as) => as.map((a) => ({ text: a.textContent, target: a.target })));
  const timed = Object.fromEntries(await timing);
  console.log(JSON.stringify({ heading, read, all, after, clipboard, links, timed, drawn, text, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
