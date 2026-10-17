// Each phone's browser for run.py here (PREDICTIONS.md §1): WebKit as an
// iPhone (web/tests/page.mjs), one browser each, kept for the whole run so
// what a phone keeps stays. It prints `{"ready": true}`, then reads one
// JSON command a line on stdin and prints one JSON answer a line:
//
//   {"do": "open", "who"}                       the phone's browser
//   {"do": "join", "who", "link", "name"}       open its link, type the
//                                               name if asked
//   {"do": "read", "who", "url", "want", "within"}
//                                               open the group's page and
//                                               read it until a line of its
//                                               removals matches `want`, or
//                                               `within` ms pass
//
// Every answer about a page is its status line, its removals and its words.
//
//   node people.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

const phones = {};

const shown = (page) => page.evaluate(() => ({
  status: document.getElementById('status')?.textContent ?? null,
  removals: [...document.querySelectorAll('#removals li')].map((l) => l.textContent),
  text: (document.body?.innerText ?? '').slice(0, 3000),
}));

async function act(c) {
  const t0 = Date.now();
  if (c.do === 'open') {
    const { context, close } = await launch('webkit');
    phones[c.who] = { close, page: await context.newPage() };
    return { ms: Date.now() - t0 };
  }
  const { page } = phones[c.who];
  if (c.do === 'join') {
    await page.goto(c.link);
    await page.waitForFunction(
      () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
      null, { timeout: 90_000 });
    const asked = await page.evaluate(() => !document.getElementById('join').hidden);
    if (asked) {
      await page.fill('#name', c.name);
      await page.click('#join button');
    }
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    return { ms: Date.now() - t0, asked, ...(await shown(page)) };
  }
  if (c.do === 'read') {
    await page.goto('about:blank');
    await page.goto(c.url);
    const want = new RegExp(c.want);
    const found = await page.waitForFunction(
      (w) => [...document.querySelectorAll('#removals li')].some((l) => new RegExp(w).test(l.textContent)),
      c.want, { timeout: c.within ?? 60_000 }).then(() => true, () => false);
    return { ms: Date.now() - t0, found, want: String(want), ...(await shown(page)) };
  }
  return { error: `no such command: ${c.do}` };
}

console.log(JSON.stringify({ ready: true }));
const input = createInterface({ input: process.stdin });
for await (const line of input) {
  if (!line.trim()) continue;
  const c = JSON.parse(line);
  let answer;
  try {
    answer = await act(c);
  } catch (e) {
    answer = { error: String(e).slice(0, 600) };
  }
  console.log(JSON.stringify(answer));
}
for (const p of Object.values(phones)) await p.close();
