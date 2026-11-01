// The browser half of page-shelf (b)'s v2-only tests (crates/svrngs/tests/
// page_v2/shelf.rs): the shelf on the group's page (`site`, `key`, `token`,
// held as page.mjs `driving` holds it) or on the home page (`home`, its
// address), each of `apps` ticked and brought by the one button, pressed a
// second time at once, as a quick thumb does. Then, once it has answered:
// what the button's next sibling says; each `disabled` the button was drawn
// with from the press on; the shelf's words once `table` is said on the
// table, or as they stand after 20 s; the page's loads since the press;
// each request the home page sent; and the trace.
//
//   node web/tests/v2/shelf.mjs '{"engine": ..., "apps": [...], "table": "Money is on the table.", "home"?: <url> | "site": ..., "key": ..., "token": ...}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { driving, launch } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const root = args.home ? '#home' : '#shelf';
async function bring(page) {
  let reloads = 0;
  const sent = [];
  page.on('load', () => reloads++);
  page.on('request', (r) => sent.push(`${r.method()} ${new URL(r.url()).pathname}`));
  const button = page.locator(`${root} button:has-text('Bring the ticked ones')`);
  await button.waitFor({ timeout: 30_000 });
  for (const app of args.apps) await page.locator(`${root} label:has-text('${app}') input`).check({ timeout: 30_000 });
  await button.evaluate((b) => {
    window.busy = [];
    new MutationObserver(() => window.busy.push(b.disabled)).observe(b, { attributes: true });
  });
  await button.evaluate((b) => (b.click(), b.click()));
  await page.waitForFunction((r) => /You brought|already|Nothing was brought/.test(document.querySelector(r).innerText), root, { timeout: 60_000 });
  const next = await button.evaluate((b) => b.nextSibling?.textContent ?? null);
  const table = await page.waitForFunction(([r, t]) => document.querySelector(r).innerText.includes(t), [root, args.table], { timeout: 20_000 }).then(() => true, () => false);
  const shelf = await page.locator(root).innerText();
  return { next, busy: await page.evaluate(() => window.busy), table, shelf, reloads, sent, lines: await page.evaluate(() => window.__lines ?? []) };
}

if (args.home) {
  const { context, close } = await launch(args.engine);
  const logs = [];
  context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  try {
    const page = await context.newPage();
    page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
    await page.goto(args.home);
    console.log(JSON.stringify({ ...(await bring(page)), logs }));
  } catch (e) {
    console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
    process.exitCode = 1;
  } finally {
    await close();
  }
} else {
  await driving(args, ({ page }) => bring(page));
}
