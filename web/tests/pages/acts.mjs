// The browser half of page-acts' test (crates/svrngs/tests/pages/acts.rs):
// the group's page (`site`, `key`, `token`, `id`, held as page.mjs
// `driving` holds it), the group kept with `plant`, a line, unsent and
// `line` its chain's last, then the page opened again, until it traces
// `carried` (`went`): how many lines the group keeps unsent then.
//
//   node web/tests/pages/acts.mjs '{"engine": ..., "site": ..., "key": ..., "token": ..., "id": ..., "plant": <line>, "line": {"seq", "id"}, "carried": <start>, "within": <ms>}'
//
// Exit 0 with the JSON line; 1 on anything else.
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
await driving(args, async ({ page, origin }) => {
  await page.goto('about:blank');
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
});
process.exit();
