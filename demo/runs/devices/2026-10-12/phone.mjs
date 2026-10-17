// Ana's phone for run.py here (PREDICTIONS.md): Playwright's WebKit as an
// iPhone (web/tests/page.mjs), one page kept for the whole run so what it
// keeps stays. It prints `{"ready": true}`, then reads one JSON command a
// line on stdin and prints one JSON answer a line:
//
//   {"do": "join", "link": <url>}   open a link; never types a name
//   {"do": "open", "lend": <name>}  open a lend from the page's list, and
//                                   wait up to 60 s for `want` in it
//   {"do": "page", "url": <url>}    open the page alone
//
//   node phone.mjs
import { createInterface } from 'node:readline';
import { launch } from '../../../../web/tests/page.mjs';

const { context, close } = await launch('webkit');
// Every heading the page shows, in order, a step said and gone included.
await context.addInitScript(() => {
  window.__said = [];
  new MutationObserver(() => {
    const s = document.getElementById('status')?.textContent;
    if (s && window.__said.at(-1) !== s) window.__said.push(s);
  }).observe(document, { subtree: true, childList: true, characterData: true });
});
const page = await context.newPage();
const logs = [];
page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`.slice(0, 300)));
let home = null;

// Once the page says it is in, or why not, or asks a name: what it shows.
async function settled(t0) {
  await page.waitForFunction(
    () => !document.getElementById('join')?.hidden || /good|bad/.test(document.getElementById('status')?.className),
    null,
    { timeout: 90_000 },
  );
  return page.evaluate((t0) => ({
    ms: Date.now() - t0,
    asked: !document.getElementById('join').hidden,
    status: document.getElementById('status').textContent,
    said: window.__said,
    lent: [...document.querySelectorAll('#lent button')].map((b) => b.textContent),
  }), t0);
}

async function act(c) {
  const t0 = Date.now();
  if (c.do === 'join' || c.do === 'page') {
    const url = c.link ?? c.url;
    home = new URL(url).origin + '/';
    await page.goto('about:blank');
    await page.goto(url);
    return settled(t0);
  }
  if (c.do === 'open') {
    if (page.url().split('#')[0] !== home) {
      await page.goto(home);
      await settled(t0);
    }
    await page.waitForSelector(`#lent button:has-text("${c.lend}")`, { timeout: 60_000 });
    const away = (url) => url.hostname.endsWith('.localhost') && url.origin + '/' !== home;
    await Promise.all([page.waitForURL(away, { timeout: 90_000 }), page.click(`#lent button:has-text("${c.lend}")`)]);
    const want = c.want ?? '';
    await page
      .waitForFunction((w) => (document.body?.innerText ?? '').trim().length > 0 && (document.body?.innerText ?? '').includes(w), want, { timeout: 60_000 })
      .catch(() => {});
    await page.waitForTimeout(c.dwell ?? 0);
    return {
      ms: Date.now() - t0,
      origin: new URL(page.url()).origin,
      text: (await page.evaluate(() => document.body?.innerText ?? '')).slice(0, 600),
    };
  }
  return { error: `no such command: ${c.do}` };
}

console.log(JSON.stringify({ ready: true }));
const input = createInterface({ input: process.stdin });
for await (const line of input) {
  if (!line.trim()) continue;
  let answer;
  try {
    answer = await act(JSON.parse(line));
  } catch (e) {
    answer = { error: String(e).slice(0, 400), logs: logs.slice(-20) };
  }
  console.log(JSON.stringify(answer));
}
await close();
