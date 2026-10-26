// The browser half of guests-link (a)'s test (crates/svrngs-node/tests/
// browser/guests.rs; svrn-docs `rings/current/GUESTS.md` §5, "How a visit
// is made"): a visit made with a link in any chat. It serves the built page
// on 127.0.0.1, as `localhost`, and prints its origin as one JSON line; the
// Rust half keeps it as Kim's group's page and answers with one JSON line on
// stdin, `{"home": <Kim's home page>, "name": <the guest's>, "open": <a
// lend's name>}`. Then, in headless `engine` (page.mjs):
//
// 1. On Kim's home page, Kim taps **Have a guest**, reads the length it
//    starts at, keeps it, and makes the link, read as the page shows it;
//    and, as no button offers, asks `/link` for a visit of 72 hours.
// 2. In a browser of its own, Ana opens the 72-hour link and gives her name.
// 3. In a browser of its own, holding a group of Sam's own (`own`, its key
//    `ownKey`), Sam opens the link, is asked their name once, gives it, and
//    opens `open` from what the page lists, at its own origin, where its
//    `<video>` plays; then opens the link again, and again once the visit
//    the page keeps has ended.
// 4. Ana opens Sam's link and gives her name.
//
// Prints what it saw as one JSON line.
//
//   node web/tests/visit.mjs '{"engine": "chromium"|"webkit", "site": "<dir>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { createInterface } from 'node:readline';
import { hold, launch, play, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
console.log(JSON.stringify({ origin }));
const input = createInterface({ input: process.stdin });
const told = await new Promise((done) => input.once('line', done));
input.close();
const { home, name, open, group, ownKey } = JSON.parse(told);

const logs = [];
// Every URL any page asked, its worker's answer or not, so a link's secret
// sent past the fragment is seen however the page is served.
const requested = [];
const watch = (who, context) => {
  context.on('request', (r) => requested.push(r.url()));
  context.on('console', (m) => logs.push(`${who} ${m.type()}: ${m.text()}`));
  context.on('page', (p) => p.on('pageerror', (e) => logs.push(`${who} pageerror: ${e}`)));
};
const kim = await launch(args.engine);
let sam, ana;
// `link` opened in `context`, the name asked given as `name`, if asked:
// the page, what it said, and whether it asked.
async function opens(context, link, name) {
  const p = await context.newPage();
  await p.goto(link);
  await p.waitForSelector('#status.good, #status.bad, #join:not([hidden])', { timeout: 90_000 });
  const asked = await p.locator('#join').isVisible();
  if (asked) await p.locator('#name').fill(name);
  if (asked) await p.locator('#join button').first().click();
  await p.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  return { p, asked, status: await p.locator('#status').textContent() };
}
try {
  watch('kim', kim.context);
  const page = await kim.context.newPage();
  await page.goto(home);
  // The group's section, held by its place: its button goes once tapped.
  await page.locator('#home section button', { hasText: 'Have a guest' }).first().waitFor({ timeout: 30_000 });
  const at = await page.locator('#home section').evaluateAll((ss) => ss.findIndex((s) => [...s.querySelectorAll('button')].some((b) => b.textContent === 'Have a guest')));
  const section = page.locator('#home section').nth(at);
  await section.locator('button', { hasText: 'Have a guest' }).click({ timeout: 30_000 });
  // The length the page asks before it makes the link, as chosen to start.
  const asks = section.locator('label', { hasText: 'For how long' }).locator('select');
  const length = await asks.evaluate((s) => s.selectedOptions[0]?.textContent ?? null, null, { timeout: 30_000 });
  const lengths = await asks.locator('option').allTextContents();
  await section.locator('button', { hasText: 'Make a link for your guest' }).click({ timeout: 30_000 });
  await section.locator('p.link code').waitFor({ timeout: 30_000 });
  const link = await section.locator('p.link code').textContent();
  const said = await section.locator('p.link').textContent();
  const sent = { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ group, device: false, visit: 72 * 3600 }) };
  const long = await page.evaluate((sent) => fetch('/link', sent).then((r) => r.json()), sent);

  ana = await launch(args.engine, { unopened: true });
  watch('ana', ana.context);
  const anaLong = (await opens(ana.context, long.https, 'Ana')).status;

  sam = await launch(args.engine, { unopened: true });
  watch('sam', sam.context);
  const phone = await sam.context.newPage();
  await phone.goto(origin);
  await phone.evaluate(hold, { hex: ownKey, token: 'own', person: 'sam', id: 'own' });
  await phone.goto('about:blank');
  await phone.goto(link);
  // Asked once: the name form, then never again.
  await phone.locator('#join').waitFor({ state: 'visible', timeout: 90_000 });
  const asking = await phone.locator('#status').textContent();
  await phone.locator('#name').fill(name);
  await phone.locator('#join button').first().click();
  await phone.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const status = await phone.locator('#status').textContent();
  const asked = await phone.locator('#join').isVisible();
  const listed = await phone.locator('#lent button').allTextContents();
  const text = await phone.locator('body').innerText();
  const away = (url) => url.hostname.endsWith('.localhost');
  await Promise.all([phone.waitForURL(away, { timeout: 90_000 }), phone.locator('#lent button', { hasText: open }).first().click()]);
  await phone.waitForSelector('video', { timeout: 60_000 });
  const opened = phone.url();
  const first = await phone.evaluate(play, { at: null, within: 15_000 });

  // The link again, then again once the visit the page keeps has ended.
  const again = await opens(sam.context, link, name);
  const token = link.slice(link.indexOf('#visit=') + '#visit='.length);
  // The page's store, imported as app.js imports it (page.mjs `hold`).
  const groups = await again.p.evaluate(async (token) => {
    const app = document.querySelector('script[type=module]').src;
    const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
    const { key, ...v } = await store.visitOf(token);
    await store.keepVisit(token, { ...v, until: Math.floor(Date.now() / 1000) - 60 }, key);
    return (await store.groups()).map((g) => g.id);
  }, token);
  const ended = await opens(sam.context, link, name);
  const anaAgain = (await opens(ana.context, link, 'Ana')).status;
  console.log(JSON.stringify({ length, lengths, link, said, asking, status, askedAgain: asked, listed, text, opened, first, anaLong, again: { status: again.status, asked: again.asked }, groups, ended: { status: ended.status, asked: ended.asked }, anaAgain, secretAsked: requested.filter((u) => u.split('#')[0].includes(token)), logs }));
} catch (e) {
  console.error(`the visit did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await kim.close();
  await sam?.close();
  await ana?.close();
  server.close();
}
