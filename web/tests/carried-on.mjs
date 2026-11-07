// The browser half of crates/svrngs-node/tests/browser/carried.rs
// `the_same_page_writes_through_the_next_computer_once_the_first_stops`: a
// phone opens an app while the computer it reached answers and writes
// `args.first`, saying so in `<dir>/wrote`. The Rust half stops that
// computer and writes `<dir>/gone`; the same page, never loaded again,
// writes `args.second` and reads every entry back.
//
//   node web/tests/carried-on.mjs '{"engine": ..., "site": ..., "key": ...,
//     "token": ..., "open": "<the app>", "dir": "<dir>", "first": "<words>",
//     "second": "<words>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { held, inApp, launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const profile = mkdtempSync(join(tmpdir(), 'svrngs-carried-on-'));
const server = await serve(args.site);
const origin = pageAt(server);
const { context, close } = await launch(args.engine, { profile });
const logs = [];
const write = (page, description) =>
  page.evaluate(async (description) => {
    const res = await fetch('svrngs/acts', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ op: { kind: 'expense', amount_cents: 1200, description } }),
    });
    return { status: res.status, said: await res.text() };
  }, description);
try {
  const page = await context.newPage();
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(page, origin, args);
  const button = page.locator('#lent button', { hasText: args.open });
  await Promise.all([page.waitForURL(inApp, { timeout: 90_000 }), button.click()]);
  await page.waitForFunction((title) => document.title === title, args.open, { timeout: 60_000 });
  const first = await write(page, args.first);
  writeFileSync(join(args.dir, 'wrote'), '');
  while (!existsSync(join(args.dir, 'gone'))) await new Promise((done) => setTimeout(done, 100));
  const second = await write(page, args.second);
  const read = await page.evaluate(async () => await (await fetch('svrngs/acts')).text());
  const loads = await page.evaluate(() => performance.getEntriesByType('navigation').length);
  console.log(JSON.stringify({ first, second, read, loads, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.closeAllConnections();
  server.close();
  rmSync(profile, { recursive: true, force: true });
}
