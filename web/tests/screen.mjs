// The browser half of the screen's test (crates/svrngs-node/tests/browser,
// `the_screen_shows_what_is_lent_after_it_opened`): the home page opened as
// a group's screen at `args.at`, `/?screen=<group>`, read once its door is
// judged — no window still "asking…" (question.js `door`); then
// `args.lend`, something running on this computer, lent from the page's
// origin as the home page lends it (home.js `running`, `POST /running`),
// and the screen read again once its list says `args.says`, by itself.
//
//   node web/tests/screen.mjs '{"engine": "chromium"|"webkit", "at": "<url>", "lend": {"port": <n>, "name": "<name>", "kind": "<kind>"}, "says": "<words>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await page.goto(args.at);
  const judged = () => ((ws) => ws.length && ![...ws].some((w) => w.textContent.includes('asking…')))(document.querySelectorAll('.door li'));
  await page.waitForFunction(judged, null, { timeout: 30_000 });
  // The screen as a person reads it: its count, each window's class and
  // words and its links, where each opens, and what arrived, newest first.
  const read = () =>
    page.evaluate(() => ({
      count: document.querySelector('section > p')?.textContent ?? null,
      windows: [...document.querySelectorAll('.door li')].map((w) => ({ class: w.className, words: w.textContent, links: [...w.querySelectorAll('a')].map((a) => [a.textContent, a.getAttribute('href'), a.target]) })),
      since: [...document.querySelectorAll('aside ol li')].map((li) => li.textContent),
      marks: [...document.querySelectorAll('mark')].map((m) => m.parentElement.textContent),
    }));
  const opened = await read();
  const lent = await page.evaluate((l) => fetch('/running', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(l) }).then((r) => r.status), args.lend);
  const says = (w) => [...document.querySelectorAll('aside ol li')].some((li) => li.textContent.includes(w));
  await page.waitForFunction(says, args.says, { timeout: 30_000 });
  const after = await read();
  console.log(JSON.stringify({ opened, lent, after, logs }));
} catch (e) {
  console.error(`the screen did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
