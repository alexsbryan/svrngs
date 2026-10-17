// Bo's phone for run.py here (PREDICTIONS.md): WebKit as an iPhone
// (web/tests/page.mjs), one browser for each path, so what the page keeps
// stays between opens. Every line the page logs at debug — its own steps
// and the dial's layers — is stamped in the page by performance.now() as
// it is logged. Prints `{"ready": true}`, then reads one JSON command a line
// on stdin and prints one JSON answer a line:
//
//   {"do": "join", "link", "name"}  open the link, type the name if asked
//   {"do": "open", "url"}           open the group's page at `url` and wait
//                                   for its status, good or bad
//   {"do": "fresh"}                 a new browser, keeping nothing
//
//   node phone.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

let browser = null;
let page = null;

async function fresh() {
  if (browser) await browser.close();
  browser = await launch('webkit');
  await browser.context.addInitScript(() => {
    const debug = console.debug.bind(console);
    window.__stamped = [];
    console.debug = (l) => {
      window.__stamped.push([Math.round(performance.now()), String(l)]);
      debug(l);
    };
  });
  page = await browser.context.newPage();
}

const status = async () => ({
  status: await page.locator('#status').textContent(),
  good: await page.evaluate(() => document.getElementById('status').className === 'good'),
  said: await page.evaluate(() => document.getElementById('response').textContent),
  lent: await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent)),
  text: (await page.evaluate(() => document.body?.innerText ?? '')).trim().slice(0, 600),
  lines: await page.evaluate(() => window.__stamped),
});

async function act(c) {
  const t0 = Date.now();
  if (c.do === 'fresh') return (await fresh(), { ok: true });
  if (c.do === 'join') {
    await page.goto(c.link);
    await page.waitForFunction(
      () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
      null, { timeout: 90_000 });
    if (await page.evaluate(() => !document.getElementById('join').hidden)) {
      await page.fill('#name', c.name);
      await page.click('#join button');
    }
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    return { ms: Date.now() - t0, ...(await status()) };
  }
  if (c.do === 'open') {
    await page.goto('about:blank');
    const t = Date.now();
    await page.goto(c.url);
    await page.waitForSelector('#status.good, #status.bad', { timeout: 120_000 });
    return { ms: Date.now() - t, ...(await status()) };
  }
  return { error: `no such command: ${c.do}` };
}

await fresh();
console.log(JSON.stringify({ ready: true }));
for await (const line of createInterface({ input: process.stdin })) {
  if (!line.trim()) continue;
  let answer;
  try {
    answer = await act(JSON.parse(line));
  } catch (e) {
    answer = { error: String(e).slice(0, 600) };
  }
  console.log(JSON.stringify(answer));
}
await browser?.close();
