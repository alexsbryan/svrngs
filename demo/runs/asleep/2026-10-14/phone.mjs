// Cy's phone for run.py here (PREDICTIONS.md): WebKit as an iPhone
// (web/tests/page.mjs), one browser for the whole run, so its worker and
// what it keeps stay. Prints `{"ready": true}`, then reads one JSON command
// a line on stdin and prints one JSON answer a line:
//
//   {"do": "join", "link", "name"}  open the link, type the name if asked
//   {"do": "open", "url", "words"}  open an address — at `#open=`, the group's
//                                   page steps aside to the lend's origin — and
//                                   wait until the page shows `words`, or any
//                                   page at all past the lend's origin
//   {"do": "trace", "url"}          the lend worker's whole trace at `url`
//
//   node phone.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

const { context, close } = await launch('webkit');
const page = await context.newPage();

async function act(c) {
  const t0 = Date.now();
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
    return { ms: Date.now() - t0, status: await page.locator('#status').textContent() };
  }
  if (c.do === 'open') {
    await page.goto('about:blank');
    let res = null;
    if (/#open=/.test(c.url)) {
      const home = new URL(c.url).host;
      await page.goto(c.url);
      await page.waitForURL((u) => u.hostname.endsWith('.localhost') && u.host !== home, { timeout: 120_000 });
    } else {
      res = await page.goto(c.url, { timeout: 120_000 });
    }
    const answered = Date.now() - t0;
    await page.waitForFunction((w) => (document.body?.innerText ?? '').includes(w) || /did not answer/.test(document.body?.innerText ?? ''),
      c.words, { timeout: 120_000 }).catch(() => {});
    const text = (await page.evaluate(() => document.body?.innerText ?? '')).trim().slice(0, 300);
    return { ms: Date.now() - t0, answered_ms: answered, status: res ? res.status() : null,
      refused: res ? (res.headers()['x-svrngs-refused'] ?? null) : null, url: page.url(), text, shown: text.includes(c.words) };
  }
  if (c.do === 'trace') {
    return { text: await page.evaluate((u) => fetch(u).then((r) => r.text()), c.url) };
  }
  return { error: `no such command: ${c.do}` };
}

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
await close();
