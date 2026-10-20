// The browser half of guests-link (a)'s test (crates/svrngs-node/tests/
// browser/guests.rs; svrn-docs `rings/current/GUESTS.md` §5, "How a visit
// is made"): a visit made with a link in any chat. It serves the built page
// on 127.0.0.1, as `localhost`, and prints its origin as one JSON line; the
// Rust half keeps it as Kim's group's page and answers with one JSON line on
// stdin, `{"home": <Kim's home page>, "name": <the guest's>, "open": <a
// lend's name>}`. Then, in headless `engine` (page.mjs):
//
// 1. On Kim's home page, Kim taps **Have a guest**, reads the length it
//    starts at, keeps it, and makes the link, read as the page shows it.
// 2. In a browser of its own, holding nothing of any group, Sam opens the
//    link, is asked their name once, gives it, and opens `open` from what
//    the page lists, at its own origin, where its `<video>` plays.
//
// Prints what it saw as one JSON line.
//
//   node web/tests/visit.mjs '{"engine": "chromium"|"webkit", "site": "<dir>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { createInterface } from 'node:readline';
import { launch, play, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
console.log(JSON.stringify({ origin }));
const input = createInterface({ input: process.stdin });
const told = await new Promise((done) => input.once('line', done));
input.close();
const { home, name, open } = JSON.parse(told);

const logs = [];
const watch = (who, context) => {
  context.on('console', (m) => logs.push(`${who} ${m.type()}: ${m.text()}`));
  context.on('page', (p) => p.on('pageerror', (e) => logs.push(`${who} pageerror: ${e}`)));
};
const kim = await launch(args.engine);
let sam;
try {
  watch('kim', kim.context);
  const page = await kim.context.newPage();
  await page.goto(home);
  const section = page.locator('#home section', { has: page.locator('button', { hasText: 'Have a guest' }) }).first();
  await section.locator('button', { hasText: 'Have a guest' }).click({ timeout: 30_000 });
  // The length the page asks before it makes the link, as chosen to start.
  const length = await section.locator('select').first().evaluate((s) => s.selectedOptions[0]?.textContent ?? null, null, { timeout: 30_000 });
  const lengths = await section.locator('select option').allTextContents();
  await section.locator('button', { hasText: 'Make a link for your guest' }).click({ timeout: 30_000 });
  await section.locator('p.link code').waitFor({ timeout: 30_000 });
  const link = await section.locator('p.link code').textContent();
  const said = await section.locator('p.link').textContent();

  sam = await launch(args.engine);
  watch('sam', sam.context);
  const phone = await sam.context.newPage();
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
  console.log(JSON.stringify({ length, lengths, link, said, asking, status, askedAgain: asked, listed, text, opened, first, logs }));
} catch (e) {
  console.error(`the visit did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await kim.close();
  await sam?.close();
  server.close();
}
