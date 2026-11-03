// The browser half of lean-4 (c)'s test (crates/svrngs-node/tests/browser):
// the home page on a computer asks svrngs's question. Opened at
// `args.ask`, its `/ask?lend=<label>&app=<label>`, it is read for its
// heading, which the test holds to the one source (question.js).
//
//   node web/tests/home.mjs '{"engine": "chromium"|"webkit", "ask": "<url>", "heading"?: false, "read"?: "<selector>", "all"?: "<selector>", "do"?: [[<selector>, <value>? | true, <stays>?], ...], "timed"?: [<selector>, ...], "drawn"?: "<selector>", "text"?: true, "open"?: "<a link's words>", "opens"?: ["<link>", ...], "older"?: true, "unfold"?: true, "look"?: <words>, "visible"?: true, "after"?: <selector> | [<selector>, <words>]}'
//
// A `do` step's fourth element, `until`, is words `all`'s text holds before
// it is read. `visible` hides the page, then shows it, once the `do` steps
// are done, as a person comes back to its tab; `after`, a selector, is then
// read into `later`, once its text holds the words beside it. It answers
// `loads`, the `load` events the page fired (1: it never reloaded), and
// `lines`, the page's `window.__lines` (ui/trace.js; v1 has none).
//
// It also answers `sent`, every request the page made (method, URL and
// body), and each link's `href`: a page never sends a secret it is shown
// anywhere but to the person reading it (oidc-turn-on).
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { arrived, firstLook, launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
// Chromium reads its clipboard only when granted it (start-home: a lend's
// address copied for a program).
if (args.engine === 'chromium')
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
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
      const lenders = (lent) => [
        ...new Map(lent.map((l) => [l.key, { person: l.person, person_key: l.key }])).values(),
      ];
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
if (args.opens)
  await context.addInitScript(() =>
    Object.assign(window, { opened: [], open: (u) => void window.opened.push(String(u)) }),
  );
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  const sent = [];
  let loads = 0;
  page.on('load', () => loads++);
  page.on('request', (r) => sent.push([r.method(), r.url(), r.postData() ?? '']));
  // With `timed`, selectors: how long after the page was asked for each
  // first showed (start-home: a group shown before its slowest computer).
  const t0 = Date.now();
  const timing = Promise.all(
    (args.timed ?? []).map((s) =>
      page
        .locator(s)
        .first()
        .waitFor({ timeout: 30_000 })
        .then(() => [s, Date.now() - t0]),
    ),
  );
  await page.goto(args.ask);
  // With `heading: false`, a page heading none (an ask not lent).
  const heading =
    args.heading === false
      ? null
      : await page.locator('#home h2').first().textContent({ timeout: 30_000 });
  // With `read`, a selector, its first match's words too (U7: a removed
  // person's line).
  const read = args.read
    ? await page.locator(args.read).first().textContent({ timeout: 30_000 })
    : null;
  // With `all`, every match's words, once the page has drawn all it draws
  // as it loads, its address last (home.js), each group's index with it
  // (v1-keeper-office (b): the removals; REVIEW-audit-v1ui-auto-1: a group's
  // section is drawn before its index, which a loaded host read empty).
  if (args.all) await page.locator('#home p.address').waitFor({ timeout: 30_000 });
  // With `unfold`, every <details> opened, as a person opens "2 apps
  // changed. See what’s new" (capability-what-acts).
  if (args.unfold)
    await page.evaluate(() => document.querySelectorAll('details').forEach((d) => (d.open = true)));
  const all = args.all ? await page.locator(args.all).allTextContents() : null;
  // With `look`, a number of words: what one look of the first screen
  // takes in, as page.mjs `firstLook` reads it, once `all` has waited for
  // the page to draw.
  const look = args.look ? await firstLook(page, args.look) : null;
  // With `do`, each [selector, value?, stays?] in turn: filled with the
  // value, else clicked, and `all`'s words read once the page it reloads
  // has listed what this computer lends (bash-limit (b)); with `stays`, a
  // click that reloads nothing, read once the page has said what happened
  // (start-home) — not while it says it is still asking, "…" (v1ui-lend
  // (b): what runs here) — and in Chromium the clipboard then. Either is
  // read once the page is at rest (`atRest`, below).
  const after = [];
  let clipboard = null;
  // `all`'s words changed from `b` and no longer "…": what a press says.
  const settled = ([s, b]) =>
    ((t) => t !== b && !t.includes('…'))(
      [...document.querySelectorAll(s)].map((e) => e.textContent).join('\n'),
    );
  const holds = ([s, w]) =>
    [...document.querySelectorAll(s)]
      .map((e) => e.textContent)
      .join('\n')
      .includes(w);
  const until = (s, w) => w && page.waitForFunction(holds, [s, w], { timeout: 30_000 });
  // `all`'s words once the page is at rest, judged in the frame they are
  // read in (js-webkit (d)): they hold `w`, if given, and no "…", and no
  // machine is still at work by its trace, `window.__lines`. A frame
  // between two of a press's reads can hold no "…": while a new group is
  // read, what acts as you is not drawn (page/acts.js `Acting`), and a
  // press was read there, its group's heading drawn with nothing under it
  // (demo/runs/webkit-wall/2026-11-03/home-group-held.txt).
  const rest = ([s, w]) => {
    const texts = [...document.querySelectorAll(s)].map((e) => e.textContent);
    const words = texts.join('\n');
    if (words.includes('…') || (w && !words.includes(w))) return false;
    // ui/machine.js traces each step `NAME: type: from -> to` and the
    // effects it runs after `, running`, and a refusal, which changes
    // nothing, `NAME: type refused in …`. A machine is at work when its
    // last step runs effects, whose answer is its next step, or leaves it
    // reading. Machines of one name, a GROUP for each group, are judged
    // by that name's last step.
    const steps = new Map();
    for (const line of window.__lines ?? []) {
      const step = line.match(/^([A-Z]+): .* -> ([^\s,]+)(, running .*)?$/);
      if (step && !line.includes(' refused in ')) steps.set(step[1], step);
    }
    const working = [...steps.values()].some(
      ([, , to, running]) => running || to.split('/').some((r) => r.endsWith('reading')),
    );
    return !working && texts;
  };
  const atRest = async (s, w) =>
    (await page.waitForFunction(rest, [s, w], { timeout: 30_000 })).jsonValue();
  for (const [selector, value, stays, words] of args.do ?? []) {
    // A value `true` ticks the box (the shelf, its boxes unticked to start).
    if (value === true) {
      await page.locator(selector).first().check({ timeout: 30_000 });
      continue;
    }
    const filled = value !== undefined && value !== null;
    if (filled && !stays) {
      await page.locator(selector).first().fill(value);
      continue;
    }
    // A fill with `stays` is read as a click that stays is: once the page
    // has said what it says as the words are typed (#138).
    if (stays) {
      const before = (await page.locator(args.all).allTextContents()).join('\n');
      await (filled
        ? page.locator(selector).first().fill(value)
        : page.locator(selector).first().click());
      await page.waitForFunction(settled, [args.all, before], { timeout: 30_000 });
      after.push(await atRest(args.all, words));
      if (args.engine === 'chromium')
        clipboard = await page
          .evaluate(() => navigator.clipboard.readText())
          .catch((e) => `unread: ${e}`);
      continue;
    }
    // A reload, or a redraw in place as v2 draws (it never reloads to
    // redraw): whichever comes first. The redraw is `stays`'s rule, which a
    // reload's navigation ends with "Execution context was destroyed", so
    // caught, and then never settled, leaving the reload's `load` to win.
    const never = new Promise(() => {});
    const before = args.all && (await page.locator(args.all).allTextContents()).join('\n');
    const changed = args.all
      ? page.waitForFunction(settled, [args.all, before], { timeout: 30_000 }).catch(() => never)
      : never;
    await Promise.all([
      Promise.race([page.waitForEvent('load'), changed]),
      page.locator(selector).first().click(),
    ]);
    await page.locator('#lent h2, #home p.none ~ p.address').first().waitFor({ timeout: 30_000 });
    await page
      .locator('#lent li, #lent p, #home p.none ~ p.address')
      .first()
      .waitFor({ timeout: 30_000 });
    after.push(await atRest(args.all, words));
  }
  // As spike run.mjs shows a page again: hidden, then visible, each said.
  if (args.visible) {
    for (const state of ['hidden', 'visible']) {
      await page.evaluate((v) => {
        Object.defineProperty(document, 'visibilityState', { value: v, configurable: true });
        document.dispatchEvent(new Event('visibilitychange'));
      }, state);
    }
  }
  const [afterAt, laterWords] = [args.after].flat();
  await until(afterAt, laterWords);
  const later = afterAt ? await page.locator(afterAt).allTextContents() : null;
  const opens = [];
  for (const link of args.opens ?? []) {
    await page.locator('#open input').fill(link, { timeout: 30_000 });
    await page.locator('#open button').click();
    opens.push(
      await page.evaluate(() => [
        window.opened.splice(0),
        document.querySelector('#open p').textContent,
      ]),
    );
  }
  // With `drawn`, a selector: each match's class and words, as the page
  // drew them once loaded (v1ui-door (c): the door) and judged — none still
  // "asking…", as each is until `/v1/lends` answers or fails (question.js
  // `door`). Read at once, a loaded host read them unjudged (#193).
  const judged = (s) =>
    ((es) => es.length && !es.some((e) => e.textContent.includes('asking…')))([
      ...document.querySelectorAll(s),
    ]);
  if (args.drawn) await page.waitForFunction(judged, args.drawn, { timeout: 30_000 });
  const drawn = args.drawn
    ? await page
        .locator(args.drawn)
        .evaluateAll((es) => es.map((e) => [e.className, e.textContent]))
    : null;
  // With `open`, a link's words: the tab it opens, once loaded, its
  // address and title (v1ui-lend (b): bo opening what ada lent).
  let opened = null;
  if (args.open) {
    const [tab] = await Promise.all([
      context.waitForEvent('page'),
      page.locator('a', { hasText: args.open }).first().click({ timeout: 30_000 }),
    ]);
    await arrived(tab);
    opened = { url: tab.url(), title: await tab.title() };
  }
  // With `text`, the page's words as a person reads them, each cell apart,
  // once what this computer lends is drawn below its heading: a lend made
  // by `do` draws it again (REVIEW-audit-v1ui-auto-1: read loaded, between).
  // In no group there is no "What you lend" (home.js): the page's address,
  // drawn last, then.
  if (args.text)
    await page
      .locator('#lent h2 ~ *, #home p.none ~ p.address')
      .first()
      .waitFor({ timeout: 30_000 });
  // Opened again, as a press above may have drawn the page again.
  if (args.unfold)
    await page.evaluate(() => document.querySelectorAll('details').forEach((d) => (d.open = true)));
  const text = args.text ? await page.locator('body').innerText() : null;
  // Every link on the page, its words and where it opens (start-home).
  const links = await page
    .locator('a')
    .evaluateAll((as) => as.map((a) => ({ text: a.textContent, target: a.target, href: a.href })));
  const timed = Object.fromEntries(await timing);
  const lines = await page.evaluate(() => window.__lines ?? null);
  console.log(
    JSON.stringify({
      heading,
      read,
      all,
      look,
      after,
      later,
      clipboard,
      links,
      timed,
      drawn,
      text,
      opened,
      opens,
      sent,
      logs,
      loads,
      lines,
    }),
  );
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
