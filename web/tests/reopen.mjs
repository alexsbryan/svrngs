// The browser half of crates/svrngs-node/tests/browser/keep_going.rs
// (KEEP_GOING_AT_HOME 4, 6, 10, 29): a phone that opened the group, and with
// `args.open` an app of it, once while everything answered, its browser then
// fully closed and opened again on what it saved. Between the two it writes
// `args.ready`, a file, and waits for `go`, while the Rust half cuts what the
// scene cuts. With `args.siteDown` the page's public site is gone too: the
// app reopens at its address, each piece a lend's origin saves answers, each
// taken out in turn stops answering (the check KEEP_GOING_AT_HOME 10 demands
// is seen to fail), and the app opens again from the group's page, which
// signs a new pass. With `args.rebuilt` a new build is deployed instead, and
// the app is opened from the group's page.
//
//   node web/tests/reopen.mjs '{"engine": ..., "site": ..., "key": ...,
//     "token": ..., "ready": "<file>", "open": "<the lend's name>",
//     "siteDown": true, "rebuilt": true}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { cpSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline';
import { held, inApp, launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const profile = mkdtempSync(join(tmpdir(), 'svrngs-reopen-'));
// A copy of the site, so `rebuild` can deploy a new build in its place.
const site = `${profile}-site`;
cpSync(args.site, site, { recursive: true });
const server = await serve(site);
const origin = pageAt(server);
// The files a lend's origin saves (web/lend.js `SAVED`), named again here
// so a piece dropped from that list fails this test.
const PIECES = [
  'open.html',
  'open.js',
  'lend.js',
  'shim.js',
  'store.js',
  'authority.js',
  'away.html',
  'wasm/svrngs_web.js',
  'wasm/svrngs_web_bg.wasm',
];
// The site as a new build makes it: every file naming the version names
// another, and the wasm and the rest are as they were.
function rebuild() {
  const version = /\?v=([0-9a-f]{12})/.exec(readFileSync(join(site, 'store.js'), 'utf8'))[1];
  const next = version.replace(/./g, (c) => (c === 'a' ? 'b' : 'a'));
  const files = readdirSync(site, { recursive: true }).filter((f) => /\.(js|html)$/.test(f));
  for (const file of files) {
    const path = join(site, file);
    writeFileSync(path, readFileSync(path, 'utf8').replaceAll(version, next));
  }
  return next;
}
let { context, close } = await launch(args.engine, { profile });
const logs = [];
const watch = (page) => {
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  return page;
};
// In the group's page: the group as its store keeps it.
const keptGroup = async () => {
  const app = document.querySelector('script[type=module]').src;
  const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
  return (await store.groups())[0];
};
// In the group's page: what its readiness check says of the kept group and a
// lend, and of a group this phone never kept.
const readiness = async ({ label, name }) => {
  const app = document.querySelector('script[type=module]').src;
  const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
  const group = (await store.groups())[0];
  const asked = {};
  for (const state of ['granted', 'denied']) {
    navigator.permissions.query = async () => ({ state });
    asked[state] = await store.unready(group, []);
  }
  return {
    ...asked,
    kept: await store.unready(group, [{ label, name }]),
    unkept: await store.unready({ id: 'none', name: 'unkept' }, []),
  };
};
const shown = (page, line) =>
  page.waitForFunction((said) => document.body.innerText.includes(said), line, { timeout: 30_000 });
// In a lend's page: what this origin's caches hold, by the file's name.
const pieces = async () => {
  const held = {};
  for (const name of await caches.keys())
    held[name] = (await (await caches.open(name)).keys()).map((r) => new URL(r.url).pathname);
  return held;
};
// In a lend's page: the status each piece answers at, this build's, then
// what it answers with its saved copy taken out, which is put back.
const answers = async (pieces) => {
  const script = document.querySelector('script[src*="shim.js"]');
  const version = new URL(script.src).searchParams.get('v');
  const at = (piece) => `/${piece}?v=${version}`;
  const asked = (piece) =>
    fetch(at(piece)).then(
      (res) => res.status,
      (e) => String(e),
    );
  const whole = {};
  const lost = {};
  const [name] = (await caches.keys()).filter((one) => one.startsWith('svrngs-web-'));
  const cache = await caches.open(name);
  for (const piece of pieces) {
    whole[piece] = await asked(piece);
    const kept = await cache.match(at(piece));
    await cache.delete(at(piece));
    lost[piece] = await asked(piece);
    if (kept) await cache.put(at(piece), kept);
  }
  return { version, whole, lost };
};
// `page` opened at `url`, and how long until its app says so.
const hi = (page) =>
  page.waitForFunction(() => document.body.innerText.includes('hi'), null, { timeout: 60_000 });
const fromList = async (page) => {
  const button = page.locator('#lent button', { hasText: args.open });
  await Promise.all([page.waitForURL(inApp, { timeout: 90_000 }), button.click()]);
  await hi(page);
};
const status = (page) =>
  page.evaluate(() => {
    const said = document.getElementById('status');
    return { said: said?.textContent, kind: said?.className };
  });

try {
  let page = watch(await context.newPage());
  await held(page, origin, args);
  const first = { ...(await status(page)), group: await page.evaluate(keptGroup) };
  let app = null;
  let saved = null;
  if (args.open) {
    await shown(page, `${args.open} is not saved here`);
    await fromList(page);
    app = page.url();
    saved = await page.evaluate(pieces);
    await page.goto(origin);
    await page.waitForSelector('#lent button', { timeout: 60_000 });
    first.ready = await page.evaluate(readiness, {
      label: new URL(app).hostname.split('.')[0],
      name: args.open,
    });
  }
  await close();
  writeFileSync(args.ready, 'ready');
  const input = createInterface({ input: process.stdin });
  await new Promise((done) => input.once('line', done));
  input.close();
  if (args.siteDown) {
    server.closeAllConnections();
    await new Promise((done) => server.close(done));
  }
  const built = args.rebuilt ? rebuild() : null;
  ({ context, close } = await launch(args.engine, { profile }));
  page = watch(context.pages()[0] ?? (await context.newPage()));
  await page.goto(origin);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  const again = await status(page);
  const reopened = {};
  if (app && args.siteDown) {
    await page.goto(app);
    await hi(page);
    Object.assign(reopened, await page.evaluate(answers, PIECES));
    await page.goto(origin);
    await page.waitForSelector('#lent button', { timeout: 90_000 });
    await fromList(page);
    reopened.fresh = true;
  }
  if (app && args.rebuilt) {
    // Waited for by the worker's cache: the page the old worker moves is
    // left, as Chromium under Playwright never finishes that navigation.
    await page.evaluate(async (name) => {
      await (await navigator.serviceWorker.getRegistration()).update();
      while (!(await caches.keys()).includes(name) || (await caches.keys()).length > 1)
        await new Promise((done) => setTimeout(done, 100));
    }, `svrngs-web-${built}`);
    await page.close();
    page = watch(await context.newPage());
    await page.goto(origin);
    await page.waitForSelector('#lent button', { timeout: 60_000 });
    await fromList(page);
    // The browser checks the worker's script itself after a navigation
    // here, so the new build's cache comes a moment after the app opens.
    await page.evaluate(async (name) => {
      const until = Date.now() + 20_000;
      while (Date.now() < until && ((await caches.keys()).length > 1 || !(await caches.has(name))))
        await new Promise((done) => setTimeout(done, 100));
    }, `svrngs-web-${built}`);
    reopened.saved = await page.evaluate(pieces);
    reopened.built = built;
  }
  console.log(JSON.stringify({ first, again, app, saved, reopened, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.closeAllConnections();
  server.close();
  rmSync(profile, { recursive: true, force: true });
  rmSync(site, { recursive: true, force: true });
}
