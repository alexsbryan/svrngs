// Cy's phone in u7's run (run.py): the built page served on `localhost`,
// as web/tests/join.mjs serves it, and Playwright's WebKit as an iPhone
// (web/tests/page.mjs). It prints `{"origin": ...}`, then takes one JSON
// command a line on stdin and answers each with one JSON line:
//
//   {"do": "join", "link": "<origin>#<token>", "name": "Cy"}   open the link, the name typed when asked
//   {"do": "open", "lend": "<words on its button>"}           the group's page again, then that lend
//   {"do": "quit"}
//
// Each answer carries what the page said, the lends it listed and, for
// `open`, the lend's page's words; a step that did not happen inside its
// time says where it stopped — never a number guessed.
import { createInterface } from 'node:readline';
import { launch, serve } from '../../../../web/tests/page.mjs';

const [site] = process.argv.slice(2);
const server = await serve(site);
const origin = `http://localhost:${server.address().port}/`;
console.log(JSON.stringify({ origin }));

const { context, close } = await launch('webkit');
const page = await context.newPage();
const logs = [];
page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));

const said = () =>
  page.evaluate(() => ({
    status: document.getElementById('status')?.textContent ?? null,
    lent: [...document.querySelectorAll('#lent button')].map((b) => b.textContent),
  }));

// The group's page at `url`, the name typed if it asks: what it says.
async function opening(url, name) {
  const t0 = Date.now();
  await page.goto('about:blank');
  await page.goto(url);
  await page.waitForFunction(
    () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
    null,
    { timeout: 90_000 },
  );
  if (await page.evaluate(() => !document.getElementById('join').hidden)) {
    await page.fill('#name', name);
    await page.click('#join button');
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  }
  return { ms: Date.now() - t0, ...(await said()) };
}

// The group's page, waited on until it lists `lend` (a lender's computer
// may answer late), then that lend opened at its own origin.
async function open(lend) {
  let seen = null;
  for (let i = 0; i < 6; i++) {
    seen = await opening(origin, 'Cy');
    if (seen.lent.some((l) => l.includes(lend))) break;
    await page.waitForTimeout(5_000);
  }
  if (!seen.lent.some((l) => l.includes(lend))) return { ...seen, opened: null, stopped: `no ${lend} listed` };
  const t0 = Date.now();
  const away = (url) => url.hostname.endsWith('.localhost') && url.pathname === '/';
  await Promise.all([page.waitForURL(away, { timeout: 90_000 }), page.locator('#lent button', { hasText: lend }).click()]);
  await page.waitForFunction(() => document.body?.innerText.trim().length > 0, null, { timeout: 60_000 }).catch(() => {});
  const text = await page.evaluate(() => document.body.innerText.trim());
  return { ...seen, opened: { origin: new URL(page.url()).origin, text, ms: Date.now() - t0 } };
}

const input = createInterface({ input: process.stdin });
for await (const line of input) {
  const cmd = JSON.parse(line);
  if (cmd.do === 'quit') break;
  let out;
  try {
    out = cmd.do === 'join' ? await opening(cmd.link, cmd.name) : await open(cmd.lend);
  } catch (e) {
    out = { stopped: String(e).split('\n')[0], ...(await said().catch(() => ({}))) };
  }
  console.log(JSON.stringify({ do: cmd.do, t: Date.now() / 1000, ...out, logs: logs.splice(0).slice(-20) }));
}
await close();
server.close();
