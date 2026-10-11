// The browser half of sup-3's test (crates/svrngs-node/tests/browser): a
// lent page asks its person through the group's page, and is stopped. The
// page holds a joined group, opens the link and then the lend `args.open`;
// at its origin the page asks `/svrngs-reads`, and the first lend it reads
// at its address, `args.probe` under it, before and after each step:
//   1. it sends its person to the `ask` address it was told, `&back=` its
//      `args.back`; the group's page asks, Yes is clicked, and the browser
//      lands wherever it is taken, which the test judges;
//   2. the group's page, opened again: "What apps use", then its Stop;
//   3. asked and said yes again, the page stops its app itself, `POST` at
//      the `stop` address it was told, `…/svrngs-reads/<label>/svrngs-stop`;
//   4. the page frames the group's page at `#ask=`, and the test reads
//      whether the group's page is in the frame;
//   5. what an app at the page's origin can read there, its pass and its
//      holder's key, handed to the test, which posts a yes with them.
//
//   node web/tests/ask.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the lend's name>",
//     "probe": "<path>", "back": "<path>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { held, launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
const out = {};
// A step that hangs is named, never waited on for good.
const watchdog = setTimeout(() => {
  console.error(`the page hung after ${JSON.stringify(out)}\n${logs.join('\n')}`);
  process.exit(1);
}, 300_000);
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(page, origin, args);
  const away = (url) => url.hostname.endsWith('.localhost') && !url.pathname.endsWith('/open.html');
  await Promise.all([page.waitForURL(away, { timeout: 90_000 }), page.locator('#lent button', { hasText: args.open }).click()]);
  await page.waitForLoadState('load');
  out.opened = page.url();
  const lendOrigin = new URL(out.opened).origin;
  const at = async (path) => {
    if (new URL(page.url()).origin !== lendOrigin) await page.goto(`${lendOrigin}/`, { waitUntil: 'load' });
    return path;
  };
  const read = async (url, init) =>
    page.evaluate(
      ([u, i]) => fetch(u, i).then(async (r) => ({ status: r.status, refused: r.headers.get('x-svrngs-refused'), body: await r.text() })),
      [url, init],
    );
  out.told = JSON.parse((await read('/svrngs-reads')).body);
  const shelf = out.told.lends?.[0] ?? {};
  const probe = async () => (await at(), read(`${shelf.address}${args.probe}`));
  out.before = await probe();

  // 1. Asked, and the person says yes.
  const ask = async () => {
    await page.evaluate((url) => location.assign(url), `${shelf.ask}&back=${encodeURIComponent(args.back)}`);
    await page.waitForSelector('#ask:not([hidden])', { timeout: 90_000 });
    const asked = await page.evaluate(() => ({ heading: document.getElementById('status').textContent, words: document.getElementById('words').textContent }));
    await Promise.all([page.waitForURL(away, { timeout: 90_000 }), page.locator('#ask button', { hasText: 'Yes' }).click()]);
    await page.waitForLoadState('load');
    return { ...asked, landed: page.url() };
  };
  out.asked = await ask();
  out.yes = await probe();

  // 2. The group's page: what apps use, and its Stop.
  await page.goto(origin);
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  await page.locator('#uses summary').click();
  await page.waitForSelector('#used li', { timeout: 90_000 });
  out.used = await page.locator('#used').innerText();
  await page.locator('#used button', { hasText: 'Stop' }).click();
  await page.waitForFunction(() => /stopped/.test(document.getElementById('used').innerText), null, { timeout: 90_000 }).catch(() => {});
  out.unused = await page.locator('#used').innerText();
  out.pageStopped = await probe();

  // 3. Said yes again; the app stops itself.
  out.again = await ask();
  out.yesAgain = await probe();
  await at();
  out.appStop = await read(shelf.stop, { method: 'POST' });
  out.appStopped = await probe();

  // 4. The group's page, framed by the lent page.
  await page.goto(`${lendOrigin}/`, { waitUntil: 'load' });
  await page.evaluate((src) => {
    const f = document.createElement('iframe');
    f.src = src;
    document.body.append(f);
    // A frame refused may never say it loaded.
    return new Promise((done) => ((f.onload = done), setTimeout(done, 5000)));
  }, `${origin}#ask=x&app=y&at=/`);
  await page.waitForTimeout(2000);
  const framed = await Promise.all(
    page.frames().filter((f) => f !== page.mainFrame()).map((f) => f.evaluate(() => document.getElementById('status')?.textContent ?? null).catch(() => null)),
  );
  out.framed = framed.filter((t) => t !== null);

  // 5. What an app at this origin can read: its pass and its holder's key.
  out.kept = await page.evaluate(
    (label) =>
      new Promise((done) => {
        const req = indexedDB.open('svrngs-lend');
        req.onsuccess = () => {
          const get = req.result.transaction('kept').objectStore('kept').get(`pass ${label}`);
          get.onsuccess = () => done({ pass: get.result?.pass, holder: get.result?.holder });
        };
      }),
    new URL(lendOrigin).hostname.split('.')[0],
  );
  console.log(JSON.stringify({ ...out, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${JSON.stringify(out)}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  clearTimeout(watchdog);
  await close();
  server.close();
}
