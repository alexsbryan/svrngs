// The browser half of page-acts' v2-only tests (crates/svrngs/tests/
// page_v2/acts.rs). The home page, `home`, its address, opened until
// `shown`, a selector, matches; with `forget`, its session cookies then
// cleared, as a session lapses; then each of `press` in turn, `{at,
// traced?: [start, n]}`: `at` clicked, then waited on, within `within` ms,
// until the trace holds n lines that start so (`went`, one a press). It
// answers the text of `at`, a selector, after each press (`texts`), the
// page's reloads (0: it drew again in place), and the trace.
//
//   node web/tests/v2/acts.mjs '{"engine": ..., "home": <url>, "shown": <selector>, "forget"?: true, "press": [{"at": <selector>, "traced"?: [<start>, <n>]}, ...], "at": <selector>, "within": <ms>}'
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
  let loads = 0;
  page.on('load', () => loads++);
  await page.goto(args.home);
  await page.locator(args.shown).first().waitFor({ timeout: 60_000 });
  if (args.forget) await context.clearCookies();
  const went = [], texts = [];
  const traced = ([start, n]) => (window.__lines ?? []).filter((l) => l.startsWith(start)).length >= n;
  for (const p of args.press ?? []) {
    await page.locator(p.at).first().click({ timeout: 30_000 });
    went.push(p.traced ? await page.waitForFunction(traced, p.traced, { timeout: args.within, polling: 100 }).then(() => true, () => false) : null);
    texts.push(await page.locator(args.at).first().innerText().catch(() => null));
  }
  console.log(JSON.stringify({ went, texts, reloads: loads - 1, lines: await page.evaluate(() => window.__lines ?? []), logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
