// The browser half of page-open (e)'s test (crates/svrngs/tests/
// pages/open.rs): the home page opened at `args.ask` (`/?made=Trip`),
// read once it is drawn, its address last; then loaded again, and read
// again. For each load: what `#home > p` says, every request it made in
// order, and the address it shows; and the trace.
//
//   node web/tests/pages/open-home.mjs '{"engine": "chromium", "ask": "http://svrngs.localhost:<port>/?made=Trip"}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { launch } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  let sent = [];
  page.on('request', (r) => sent.push(new URL(r.url()).pathname));
  // Drawn: its address, the page's last part, and its group's lends
  // answered (GROUP's trace; v1 traces nothing, so is read at its address).
  const read = async () => {
    await page.locator('#home p.address').waitFor({ timeout: 30_000 });
    await page.waitForFunction(() => !window.__lines || window.__lines.some((l) => l.startsWith('GROUP: answered') && l.includes('(lends')), null, { timeout: 30_000 });
    const seen = { said: await page.locator('#home > p').allTextContents(), sent, url: page.url() };
    sent = [];
    return seen;
  };
  await page.goto(args.ask);
  const first = await read();
  await page.reload();
  const again = await read();
  console.log(JSON.stringify({ first, again, lines: await page.evaluate(() => window.__lines ?? null), logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
