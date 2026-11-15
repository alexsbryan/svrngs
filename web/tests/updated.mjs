// The browser half of crates/svrngs-node/tests/browser/updated.rs: the
// group's page at an invitation's address, its `#fragment` and all, under
// build A's worker, then build B deployed at the same origin. On
// 2026-11-14 a browser that had opened the page before opened a new
// build's invitation and ran the old build's page, which could not read it,
// until a hard reload: A's worker answered the page itself from its cache,
// and its update moved the open pages with `navigate(page.url)`, which at
// an address with a fragment moves within the page and loads nothing.
//
//   node web/tests/updated.mjs '{"engine": ..., "site": ...}'
//
// Exit 0 with the JSON line: the build each tab runs once B's worker holds
// the origin alone; 3, naming the install command, when Playwright or its
// browser is absent; 1 on anything else.
import { cpSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const site = mkdtempSync(join(tmpdir(), 'svrngs-updated-'));
cpSync(args.site, site, { recursive: true });
const server = await serve(site);
const invitation = `${pageAt(server)}#an-invitation`;

// The site as a new build makes it, as build-web.sh stamps one: every file
// naming the version names another (reopen.mjs `rebuild`).
function rebuild() {
  const version = /\?v=([0-9a-f]{12})/.exec(readFileSync(join(site, 'store.js'), 'utf8'))[1];
  const next = version.replace(/./g, (c) => (c === 'a' ? 'b' : 'a'));
  const files = readdirSync(site, { recursive: true }).filter((f) => /\.(js|html)$/.test(f));
  for (const file of files) {
    const path = join(site, file);
    writeFileSync(path, readFileSync(path, 'utf8').replaceAll(version, next));
  }
  return { version, next };
}
// `test` of `arg` read in `page`, or null where it gives no answer in 2 s:
// Chromium under Playwright never finishes the navigation an old worker
// moves a page with (reopen.mjs), and a read waits on it for good.
const read = (page, test, arg) =>
  Promise.race([
    page.evaluate(test, arg).catch(() => null),
    new Promise((done) => setTimeout(() => done(null), 2_000)),
  ]);
// The build the document in `page` runs, by its app.js's address.
const running = (page) =>
  read(page, () =>
    new URL(document.querySelector('script[type=module]').src).searchParams.get('v'),
  );
// Until `test` of `arg` holds in `page`, at most `ms`, read again across a
// reload.
async function until(page, test, ms, arg) {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    if (await read(page, test, arg)) return true;
    await new Promise((done) => setTimeout(done, 200));
  }
  return false;
}

const { context, close } = await launch(args.engine, { unopened: true, b: true });
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const left = await context.newPage();
  await left.goto(invitation);
  const controlled = await until(left, () => Boolean(navigator.serviceWorker.controller), 30_000);
  const { version, next } = rebuild();
  // The same invitation opened again in a tab of its own, as the person did;
  // the tab first opened is left open, as a phone's is.
  const opened = await context.newPage();
  await opened.goto(invitation);
  // B's worker in place, A's cache gone (web/sw.js `activated`).
  const alone = await until(
    opened,
    async (name) => JSON.stringify(await caches.keys()) === JSON.stringify([name]),
    30_000,
    `svrngs-web-${next}`,
  );
  // A moment for a page the update reloads to come back.
  for (const page of [opened, left])
    await until(page, () => document.readyState === 'complete', 10_000);
  const runs = async () => ({ opened: await running(opened), left: await running(left) });
  let seen = await runs();
  for (let i = 0; i < 50 && (seen.opened !== next || seen.left !== next); i++) {
    await new Promise((done) => setTimeout(done, 200));
    seen = await runs();
  }
  const urls = { opened: opened.url(), left: left.url() };
  console.log(JSON.stringify({ a: version, b: next, controlled, alone, runs: seen, urls, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.closeAllConnections();
  server.close();
  rmSync(site, { recursive: true, force: true });
}
