// Driver for HOLDER_APPROVAL.md §3. usage: node measure.js [webkit|chromium|chromium-part ...]
// A: opener handoff (new tab kept as opener; same-tab navigation; reload;
//    restart; a bookmarked visit; another app's partition).
// B: popup per approval (no gesture; a click in the frame; a click in the
//    app page relayed to the frame), with the harness's own popup control.
// D: __Host- cookies set by the group page, read by the framed signer
//    (reload, restart, cookie tossing by the app).
process.env.HTTPS_PORT ??= '9443';
process.env.PROXY_PORT ??= '9888';
const fs = require('fs');
const os = require('os');
const path = require('path');
const pw = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright');
const { GROUP } = require('./server.js');
const { PROXY_PORT } = require('./proxy.js');

const G = `https://${GROUP}`;
// Production's name shape: a 62-character label under the group's host (sign.js:41).
const L1 = 'a'.repeat(31) + '1'.repeat(31);
const L2 = 'b'.repeat(31) + '2'.repeat(31);
const APP1 = `https://${L1}.${GROUP}/`;
const APP2 = `https://${L2}.${GROUP}/`;

const { chromiumSwitches } = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright-core/lib/server/chromium/chromiumSwitches.js');
const PW_DISABLE = chromiumSwitches(false).find((a) => a.startsWith('--disable-features='));
// Shipped Chrome partitions third-party storage and blocks popups; Playwright turns both off.
const real = { ignoreDefaultArgs: [PW_DISABLE, '--disable-popup-blocking'], args: [PW_DISABLE.replace(/,?ThirdPartyStoragePartitioning/, '')] };
const ENGINES = {
  webkit: [pw.webkit, {}],
  chromium: [pw.chromium, {}],
  'chromium-part': [pw.chromium, real],
};
const base = { ignoreHTTPSErrors: true, proxy: { server: `http://127.0.0.1:${PROXY_PORT}` } };

const result = (page, ms = 8000) => page.waitForFunction(() => window.__result, null, { timeout: ms }).then((h) => h.jsonValue()).catch((e) => ({ error: String(e).split('\n')[0] }));
const pick = (r, keys) => Object.fromEntries(keys.filter((k) => k in r).map((k) => [k, r[k]]));
const log = (engine, step, r) => console.log(JSON.stringify({ engine, step, ...r }));

async function persistent(type, opts, dir) {
  return type.launchPersistentContext(dir, { ...opts, ...base });
}

async function opener(name, [type, opts]) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hd-'));
  const K = ['opener', 'reply', 'reply_ms', 'storedNow', 'stored', 'bc', 'ancestorTop', 'error'];
  let ctx = await persistent(type, opts, dir);
  const g = await ctx.newPage();
  await g.goto(`${G}/?mode=opener&app=${encodeURIComponent(APP1 + '?ask=opener')}`);
  const [tab] = await Promise.all([ctx.waitForEvent('page'), g.click('#open')]);
  log(name, 'A1 new tab via window.open, frame asks top.opener', pick(await result(tab), K));
  log(name, 'A1 group page saw', { asked: await g.evaluate(() => window.__asked) });
  await tab.goto(APP1); // reload, no ask
  log(name, 'A2 app reloaded: frame reads its own store', pick(await result(tab), K));
  await g.close();
  await tab.goto(APP1 + '?ask=opener');
  log(name, 'A3 group tab closed, frame asks again', pick(await result(tab), K));
  const other = await ctx.newPage();
  await other.goto(APP2 + '?ask=opener');
  log(name, 'A4 another app, typed/bookmarked, no opener', pick(await result(other), K));
  await ctx.close();
  ctx = await persistent(type, opts, dir);
  const back = await ctx.newPage();
  await back.goto(APP1 + '?ask=opener');
  log(name, 'A5 browser restarted, app opened from a bookmark', pick(await result(back), K));
  // Same tab: production's handing() navigates (web/page/index.js:69).
  const g2 = await ctx.newPage();
  await g2.goto(`${G}/?mode=nav&app=${encodeURIComponent(APP2 + '?ask=opener')}`);
  await g2.waitForURL((u) => u.host.startsWith(L2));
  log(name, 'A6 same tab: group page navigated to the app', pick(await result(g2), K));
  await ctx.close();
}

async function popups(name, [type, opts]) {
  const browser = await type.launch(opts);
  const ctx = await browser.newContext(base);
  const K = ['popup_load', 'popup_load_read', 'popup_click', 'popup_click_read', 'popup_click_ms', 'popup_msg', 'popup_msg_read', 'popup_msg_ms', 'error'];
  const g = await ctx.newPage();
  await g.goto(`${G}/?mode=write`);
  await g.waitForFunction(() => window.__done);
  const c = await ctx.newPage();
  await c.goto(`${G}/?mode=popctl`);
  log(name, 'B0 control: top-level window.open at load, no gesture', await c.waitForFunction(() => window.__done).then((h) => h.jsonValue()));
  const c2 = await ctx.newPage();
  await c2.goto(APP1 + '?popctl=1');
  log(name, 'B0 control: app page window.open at load, no gesture', { popup: await c2.evaluate(() => window.__popctl) });
  const a = await ctx.newPage();
  await a.goto(APP1 + '?popup=load');
  log(name, 'B1 frame opens popup at load, no gesture', pick(await result(a), K));
  const b = await ctx.newPage();
  await b.goto(APP1 + '?popup=click');
  await b.waitForFunction(() => window.__results.length > 0);
  await b.frameLocator('iframe').locator('#b').click();
  log(name, 'B2 frame opens popup on a tap in the frame', pick(await result(b), K));
  const m = await ctx.newPage();
  await m.goto(APP1 + '?popup=msg');
  await m.waitForFunction(() => window.__results.length > 0);
  await m.click('#top');
  log(name, 'B3 tap in the app page, relayed to the frame, which opens the popup', pick(await result(m), K));
  await browser.close();
}

async function cookies(name, [type, opts]) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hd-'));
  const K = ['cookie', 'ls', 'ed25519', 'error'];
  let ctx = await persistent(type, opts, dir);
  const p = await ctx.newPage();
  await p.goto(`${G}/?mode=cookie&go=1&app=${encodeURIComponent(APP1)}`);
  await p.waitForURL((u) => u.host.startsWith(L1));
  log(name, 'D1 group page set __Host- cookies, navigated (same tab) to the app; frame reads', pick(await result(p), K));
  log(name, 'D1 the app page itself sees', { appSees: await p.evaluate(() => document.cookie) });
  await p.goto(APP1 + '?toss=1');
  log(name, 'D2 app tries to toss cookies at the group host', await p.evaluate(() => window.__toss));
  log(name, 'D2 frame then reads', pick(await result(p), K));
  await ctx.close();
  ctx = await persistent(type, opts, dir);
  const q = await ctx.newPage();
  await q.goto(APP2);
  log(name, 'D3 browser restarted, another app opened from a bookmark; frame reads', pick(await result(q), K));
  await ctx.close();
}

(async () => {
  const want = process.argv.slice(2).length ? process.argv.slice(2) : ['webkit', 'chromium', 'chromium-part'];
  for (const name of want) {
    const [type, opts] = ENGINES[name];
    const b = await type.launch(opts);
    console.log(`# ${name} ${b.version()} (${new Date().toISOString()})`);
    await b.close();
    for (const f of [opener, popups, cookies]) {
      try { await f(name, ENGINES[name]); } catch (e) { log(name, f.name + ' THREW', { error: String(e).split('\n')[0] }); }
    }
  }
  process.exit(0);
})();
