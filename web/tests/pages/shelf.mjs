// The browser half of page-shelf (b)'s test (crates/svrngs/tests/
// pages/shelf.rs): the shelf on the group's page (`site`, `key`, `token`,
// held as page.mjs `driving` holds it), each of `apps` ticked and brought
// by the one button, pressed a second time at once, as a quick thumb does.
// Then, once it has answered: what the button's next sibling says; each
// `disabled` the button was drawn with from the press on; the shelf's
// words once `table` is said on the table, or as they stand after 20 s;
// the page's loads since the press; and the trace.
//
//   node web/tests/pages/shelf.mjs '{"engine": ..., "apps": [...], "table": "Money is on the table.", "site": ..., "key": ..., "token": ...}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { driving } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const root = '#shelf';
async function bring(page) {
  let reloads = 0;
  page.on('load', () => reloads++);
  const button = page.locator(`${root} button:has-text('Bring the ticked ones')`);
  await button.waitFor({ timeout: 30_000 });
  for (const app of args.apps)
    await page.locator(`${root} label:has-text('${app}') input`).check({ timeout: 30_000 });
  await button.evaluate((b) => {
    window.busy = [];
    new MutationObserver(() => window.busy.push(b.disabled)).observe(b, { attributes: true });
  });
  await button.evaluate((b) => (b.click(), b.click()));
  await page.waitForFunction(
    (r) => /You brought|already|Nothing was brought/.test(document.querySelector(r).innerText),
    root,
    { timeout: 60_000 },
  );
  const next = await button.evaluate((b) => b.nextSibling?.textContent ?? null);
  const table = await page
    .waitForFunction(
      ([r, t]) => document.querySelector(r).innerText.includes(t),
      [root, args.table],
      { timeout: 20_000 },
    )
    .then(
      () => true,
      () => false,
    );
  const shelf = await page.locator(root).innerText();
  return {
    next,
    busy: await page.evaluate(() => window.busy),
    table,
    shelf,
    reloads,
    lines: await page.evaluate(() => window.__lines ?? []),
  };
}

await driving(args, ({ page }) => bring(page));
