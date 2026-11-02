// Does the Storage Access API (requestStorageAccess after a click in the
// frame) give WebKit's framed group origin the first-party IndexedDB?
// usage: node saa.js
const { webkit } = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright');
require('./server.js');
const { PROXY_PORT } = require('./proxy.js');
(async () => {
  const browser = await webkit.launch();
  console.log('# webkit ' + browser.version());
  for (const [group, app] of [['group.svrnme.sh', 'app1.group.svrnme.sh'], ['group.svrnme.sh', 'app.other.sh']]) {
    const ctx = await browser.newContext({ ignoreHTTPSErrors: true, proxy: { server: `http://127.0.0.1:${PROXY_PORT}` } });
    const g = await ctx.newPage();
    await g.goto(`https://${group}/`);
    await g.mouse.click(5, 5); // a first-party user interaction with the group site
    await g.waitForFunction(() => window.__done);
    const a = await ctx.newPage();
    await a.goto(`https://${app}/?g=${group}&saa`);
    await a.frameLocator('iframe').locator('#b').click();
    const r = await a.waitForFunction(() => window.__result, null, { timeout: 10000 }).then((h) => h.jsonValue()).catch((e) => ({ error: String(e).split('\n')[0] }));
    console.log(app, JSON.stringify({ saa: r.saa, idb: r.idb, idbFresh: r.idbFresh, key: r.key, ls: r.ls, hasStorageAccess: r.hasStorageAccess, error: r.error }));
    await ctx.close();
  }
  await browser.close();
  process.exit(0);
})();
