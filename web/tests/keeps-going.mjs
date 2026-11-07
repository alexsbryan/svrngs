// The browser half of crates/svrngs-node/tests/browser/keep_going.rs's whole
// scene (KEEP_GOING_AT_HOME 11, 14): a phone that opened an app while
// everything answered, its browser then fully closed. The Rust half takes
// the internet away and writes `<dir>/gone`. With the page's site gone too,
// the browser opens again on what it saved, reopens the app, writes
// `args.outage` and leaves the page open, saying so in `<dir>/wrote`. The
// Rust half brings the internet back and writes `<dir>/back`; the same page
// writes `args.after`, and says whether it was ever loaded again.
//
//   node web/tests/keeps-going.mjs '{"engine": ..., "site": ..., "key": ...,
//     "token": ..., "id": ..., "open": "<the app>", "dir": "<dir>",
//     "outage": "<words>", "after": "<words>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { held, inApp, launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const profile = mkdtempSync(join(tmpdir(), 'svrngs-keeps-going-'));
const server = await serve(args.site);
const origin = pageAt(server);
let { context, close } = await launch(args.engine, { profile });
const logs = [];
const watch = (page) => {
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  return page;
};
const told = async (name) => {
  while (!existsSync(join(args.dir, name))) await new Promise((done) => setTimeout(done, 100));
};
const open = async (page) => {
  const button = page.locator('#lent button', { hasText: args.open });
  await Promise.all([page.waitForURL(inApp, { timeout: 90_000 }), button.click()]);
  await page.waitForFunction(() => document.title === 'Money', null, { timeout: 60_000 });
};
const write = (page, words) =>
  page.evaluate(async (description) => {
    const res = await fetch('svrngs/acts', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ op: { kind: 'expense', amount_cents: 1200, description } }),
    });
    return { status: res.status, said: await res.text() };
  }, words);
try {
  let page = watch(await context.newPage());
  await held(page, origin, args);
  await open(page);
  await page.goto(origin);
  await page.waitForSelector('#lent button', { timeout: 60_000 });
  await close();
  writeFileSync(join(args.dir, 'closed'), '');
  await told('gone');
  server.closeAllConnections();
  await new Promise((done) => server.close(done));
  ({ context, close } = await launch(args.engine, { profile }));
  page = watch(context.pages()[0] ?? (await context.newPage()));
  await page.goto(origin);
  await page.waitForSelector('#lent button', { timeout: 90_000 });
  await open(page);
  const during = await write(page, args.outage);
  const app = page.url();
  await page.evaluate(() => {
    window.stayed = 'the same page';
  });
  writeFileSync(join(args.dir, 'wrote'), JSON.stringify(during));
  await told('back');
  const after = await write(page, args.after);
  const stayed = await page.evaluate(() => ({
    stayed: window.stayed,
    loads: performance.getEntriesByType('navigation').length,
    type: performance.getEntriesByType('navigation')[0]?.type,
  }));
  console.log(JSON.stringify({ during, after, stayed, same: page.url() === app, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.closeAllConnections();
  server.close();
  rmSync(profile, { recursive: true, force: true });
}
