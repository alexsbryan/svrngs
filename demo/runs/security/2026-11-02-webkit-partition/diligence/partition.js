// Driver: per engine x layout, a fresh context; write at the group origin
// top-level, then open the app page (same context) which frames the group
// origin's frame.html and reports what it sees.
// usage: node partition.js [webkit|chromium ...]
const { webkit, chromium } = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright');
const { HTTPS_PORT, HTTP_PORT } = require('./server.js');
const { PROXY_PORT, seen } = require('./proxy.js');

const H = HTTP_PORT;
// [name, group host, app host, scheme, via proxy]
const LAYOUTS = [
  ['same-origin control (.sh)', 'group.svrnme.sh', 'group.svrnme.sh', 'https', true],
  ['same-site  .sh (prod shape)', 'group.svrnme.sh', 'app1.group.svrnme.sh', 'https', true],
  ['cross-site .sh control', 'group.svrnme.sh', 'app.other.sh', 'https', true],
  ['same-site  .co.uk', 'group.svrnme.co.uk', 'app1.group.svrnme.co.uk', 'https', true],
  ['same-site  .test', 'group.svrnme.test', 'app1.group.svrnme.test', 'https', true],
  ['cross-site .test control', 'group.svrnme.test', 'app.other.test', 'https', true],
  ['localhost: localhost / app.localhost', `localhost:${H}`, `app.localhost:${H}`, 'http', false],
  ['localhost: page.localhost / app.page.localhost', `page.localhost:${H}`, `app.page.localhost:${H}`, 'http', false],
];

// Playwright launches Chromium with --disable-features=...,ThirdPartyStoragePartitioning,...
// (playwright-core/lib/server/chromium/chromiumSwitches.js:42, issue #32230).
// The *-part engines drop that flag and pass the same list without it, so the
// browser partitions as shipped Chrome does.
const { chromiumSwitches } = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright-core/lib/server/chromium/chromiumSwitches.js');
const PW_DISABLE = chromiumSwitches(false).find((a) => a.startsWith('--disable-features='));
const OURS = PW_DISABLE.replace(/,?ThirdPartyStoragePartitioning/, '');
const part = { ignoreDefaultArgs: [PW_DISABLE], args: [OURS] };
const ENGINES = {
  webkit: () => webkit.launch(),
  chromium: () => chromium.launch(),
  'chromium-part': () => chromium.launch(part),
  'chrome-part': () => chromium.launch({ channel: 'chrome', ...part }),
};

(async () => {
  const want = process.argv.slice(2).length ? process.argv.slice(2) : ['webkit', 'chromium'];
  const rows = [];
  for (const name of want) {
    const browser = await ENGINES[name]();
    console.log(`# ${name} ${browser.version()}`);
    for (const [label, group, app, scheme, viaProxy] of LAYOUTS) {
      const ctx = await browser.newContext({
        ignoreHTTPSErrors: true,
        ...(viaProxy ? { proxy: { server: `http://127.0.0.1:${PROXY_PORT}` } } : {}),
      });
      let res;
      try {
        const g = await ctx.newPage();
        await g.goto(`${scheme}://${group}/`);
        const done = await g.waitForFunction(() => window.__done, null, { timeout: 10000 }).then((h) => h.jsonValue());
        if (done.error) throw new Error('write: ' + done.error);
        const a = await ctx.newPage();
        await a.goto(`${scheme}://${app}/?g=${group}`);
        res = await a.waitForFunction(() => window.__result, null, { timeout: 10000 }).then((h) => h.jsonValue());
      } catch (e) {
        res = { error: String(e).split('\n')[0] };
      }
      await ctx.close();
      const tag = 'written-' + group;
      const v = (x) => (x === tag ? 'YES' : x === null || x === undefined ? 'no' : String(x));
      const row = res.error
        ? { engine: name, layout: label, error: res.error }
        : {
            engine: name, layout: label,
            idb: v(res.idb) + (res.idbFresh ? ' (fresh db)' : ''),
            ls: v(res.ls),
            key: res.key ?? 'no',
            cookie: res.cookie && res.cookie.includes(tag) ? 'YES' : 'no',
            hasStorageAccess: res.hasStorageAccess,
          };
      rows.push(row);
      console.log(JSON.stringify(row));
    }
    await browser.close();
  }
  console.log('# proxy saw: ' + [...new Set(seen)].join(', '));
  process.exit(0);
})();
