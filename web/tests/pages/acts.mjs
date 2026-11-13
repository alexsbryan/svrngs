// The browser half of page-acts' tests (crates/svrngs/tests/
// pages/acts.rs): the group's page (`site`, `key`, `token`, `id`, held as
// page.mjs `driving` holds it). With `ask`, a fragment, opened at it until
// #ask is shown, then, once the test, told so (`<hold>.shown`), has written
// `hold`, Yes pressed, and within `within` ms the heading changed (`went`):
// the heading as it asked (`asked`) and then (`status`), and whether #ask is
// hidden; or with `plant`, a line, the group kept with it unsent and
// `line` its chain's last, then the page opened again, until it traces
// `carried` (`went`): how many lines the group keeps unsent then.
//
//   node web/tests/pages/acts.mjs '{"engine": ..., "site": ..., "key": ..., "token": ..., "id": ..., "ask": <fragment>, "hold": <path>, "within": <ms>}'
//   node web/tests/pages/acts.mjs '{"engine": ..., "site": ..., "key": ..., "token": ..., "id": ..., "plant": <line>, "line": {"seq", "id"}, "carried": <start>, "within": <ms>}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { existsSync } from 'node:fs';
import { writeFile } from 'node:fs/promises';
import { driving } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
// web/store.js's database.
const DB = 'svrngs';
// The group last opened, as web/store.js keeps it in IndexedDB, changed
// by `change` when given: from any page of its origin, script or none.
function kept([db, change]) {
  const done = (req) =>
    new Promise(
      (ok, no) => ((req.onsuccess = () => ok(req.result)), (req.onerror = () => no(req.error))),
    );
  return done(indexedDB.open(db)).then(async (d) => {
    const [g] = (await done(d.transaction('groups').objectStore('groups').getAll())).sort(
      (a, b) => b.opened - a.opened,
    );
    return change
      ? done(
          d
            .transaction('groups', 'readwrite')
            .objectStore('groups')
            .put({ ...g, ...change }),
        ).then(() => g)
      : g;
  });
}
const status = (page) => page.locator('#status').textContent();
await driving(args, async ({ page, origin }) => {
  await page.goto('about:blank');
  if (args.plant) {
    // Planted where no page holds the group, so none keeps it over the plant.
    await page.goto(`${origin}away.html`);
    await page.evaluate(kept, [DB, { line: args.line, unsent: [args.plant] }]);
    await page.goto('about:blank');
    await page.goto(origin);
    const went = await page
      .waitForFunction((c) => (window.__lines ?? []).some((l) => l.startsWith(c)), args.carried, {
        timeout: args.within,
        polling: 100,
      })
      .then(
        () => true,
        () => false,
      );
    return {
      went,
      unsent: (await page.evaluate(kept, [DB])).unsent?.length ?? 0,
      lines: await page.evaluate(() => window.__lines ?? []),
    };
  }
  await page.goto(`${origin}#${args.ask}`);
  await page.locator('#ask:not([hidden])').waitFor({ timeout: 90_000 });
  const asked = await status(page);
  await writeFile(`${args.hold}.shown`, '');
  for (let t = 0; !existsSync(args.hold); t += 100)
    await (t > 60_000
      ? Promise.reject(new Error(`${args.hold} never written`))
      : new Promise((r) => setTimeout(r, 100)));
  await page.locator('#ask button:text-is("Yes")').click({ timeout: 10_000 });
  const went = await page
    .waitForFunction((a) => document.getElementById('status')?.textContent !== a, asked, {
      timeout: args.within,
      polling: 100,
    })
    .then(
      () => true,
      () => false,
    );
  return {
    asked,
    went,
    status: await status(page),
    hidden: await page.locator('#ask').evaluate((f) => f.hidden),
    lines: await page.evaluate(() => window.__lines ?? []),
  };
});
process.exit();
