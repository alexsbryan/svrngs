// The browser half of lean-4 (c)'s test (crates/svrngs-node/tests/browser):
// the home page on a computer asks svrngs's question. Opened at
// `args.ask`, its `/ask?lend=<label>&app=<label>`, it is read for its
// heading, which the test holds to the one source (question.js).
//
//   node web/tests/home.mjs '{"engine": "chromium"|"webkit", "ask": "<url>", "heading"?: false, "read"?: "<selector>", "all"?: "<selector>", "do"?: [[<selector>, <value>?, <stays>?], ...], "timed"?: [<selector>, ...], "drawn"?: "<selector>", "text"?: true, "open"?: "<a link's words>", "opens"?: ["<link>", ...], "older"?: true, "unfold"?: true}'
//
// It also answers `sent`, every request the page made (method, URL and
// body), and each link's `href`: a page never sends a secret it is shown
// anywhere but to the person reading it (oidc-turn-on).
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
// Chromium reads its clipboard only when granted it (start-home: a lend's
// address copied for a program).
if (args.engine === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
// With `older`, every lender's computer in the group's answer as this
// computer names one running an older svrngs (arch-version; `node.rs`
// `answered`): in `older`, each by name and first key, a computer's key
// its person's first here, and none of its lends listed. Planted where the
// page reads the answer: a route's own fetch would lose the page's
// Sec-Fetch headers, which the home page answers only with.
if (args.older) {
  await context.addInitScript(() => {
    const json = Response.prototype.json;
    Response.prototype.json = async function () {
      const body = await json.call(this);
      const lenders = (lent) => [...new Map(lent.map((l) => [l.key, { person: l.person, person_key: l.key }])).values()];
      if (body?.lent && body.unanswered) [body.older, body.lent] = [lenders(body.lent), []];
      return body;
    };
  });
}
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
// With `opens`, links pasted under "Open a link from your group", one at a
// time: where each would open a tab — `window.open` kept, not followed — and
// what the section then says (#153).
if (args.opens) await context.addInitScript(() => Object.assign(window, { opened: [], open: (u) => void window.opened.push(String(u)) }));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  const sent = [];
  page.on('request', (r) => sent.push([r.method(), r.url(), r.postData() ?? '']));
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
  // With `all`, every match's words, once the page has drawn all it draws
  // as it loads, its address last (home.js), each group's index with it
  // (v1-keeper-office (b): the removals; REVIEW-audit-v1ui-auto-1: a group's
  // section is drawn before its index, which a loaded host read empty).
  if (args.all) await page.locator('#home p.address').waitFor({ timeout: 30_000 });
  // With `unfold`, every <details> opened, as a person opens "2 apps
  // changed. See what’s new" (capability-what-acts).
  if (args.unfold) await page.evaluate(() => document.querySelectorAll('details').forEach((d) => (d.open = true)));
  const all = args.all ? await page.locator(args.all).allTextContents() : null;
  // With `do`, each [selector, value?, stays?] in turn: filled with the
  // value, else clicked, and `all`'s words read once the page it reloads
  // has listed what this computer lends (bash-limit (b)); with `stays`, a
  // click that reloads nothing, read once the page has said what happened
  // (start-home) — not while it says it is still asking, "…" (v1ui-lend
  // (b): what runs here) — and in Chromium the clipboard then.
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
      await page.waitForFunction(([s, b]) => ((t) => t !== b && !t.includes('…'))([...document.querySelectorAll(s)].map((e) => e.textContent).join('\n')), [args.all, before], { timeout: 30_000 });
      after.push(await page.locator(args.all).allTextContents());
      if (args.engine === 'chromium') clipboard = await page.evaluate(() => navigator.clipboard.readText()).catch((e) => `unread: ${e}`);
      continue;
    }
    await Promise.all([page.waitForEvent('load'), page.locator(selector).first().click()]);
    await page.locator('#lent h2').waitFor({ timeout: 30_000 });
    await page.locator('#lent li, #lent p').first().waitFor({ timeout: 30_000 });
    after.push(await page.locator(args.all).allTextContents());
  }
  const opens = [];
  for (const link of args.opens ?? []) {
    await page.locator('#open input').fill(link, { timeout: 30_000 });
    await page.locator('#open button').click();
    opens.push(await page.evaluate(() => [window.opened.splice(0), document.querySelector('#open p').textContent]));
  }
  // With `drawn`, a selector: each match's class and words, as the page
  // drew them once loaded (v1ui-door (c): the door) and judged — none still
  // "asking…", as each is until `/v1/lends` answers or fails (question.js
  // `door`). Read at once, a loaded host read them unjudged (#193).
  const judged = (s) => ((es) => es.length && !es.some((e) => e.textContent.includes('asking…')))([...document.querySelectorAll(s)]);
  if (args.drawn) await page.waitForFunction(judged, args.drawn, { timeout: 30_000 });
  const drawn = args.drawn ? await page.locator(args.drawn).evaluateAll((es) => es.map((e) => [e.className, e.textContent])) : null;
  // With `open`, a link's words: the tab it opens, once loaded, its
  // address and title (v1ui-lend (b): bo opening what ada lent).
  let opened = null;
  if (args.open) {
    const [tab] = await Promise.all([context.waitForEvent('page'), page.locator('a', { hasText: args.open }).first().click({ timeout: 30_000 })]);
    await tab.waitForLoadState();
    opened = { url: tab.url(), title: await tab.title() };
  }
  // With `text`, the page's words as a person reads them, each cell apart,
  // once what this computer lends is drawn below its heading: a lend made
  // by `do` draws it again (REVIEW-audit-v1ui-auto-1: read loaded, between).
  if (args.text) await page.locator('#lent h2 ~ *').first().waitFor({ timeout: 30_000 });
  const text = args.text ? await page.locator('body').innerText() : null;
  // Every link on the page, its words and where it opens (start-home).
  const links = await page.locator('a').evaluateAll((as) => as.map((a) => ({ text: a.textContent, target: a.target, href: a.href })));
  const timed = Object.fromEntries(await timing);
  console.log(JSON.stringify({ heading, read, all, after, clipboard, links, timed, drawn, text, opened, opens, sent, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
