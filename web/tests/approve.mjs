// The browser half of the holder-approval test (crates/svrngs-node/tests/
// browser/carried.rs, `an_app_s_entry_is_approved_by_the_group_s_signer_and_carried`;
// svrn-docs rings/current/HOLDER_APPROVAL.md §3.D and §6, "the whole flow
// end to end"): on the group's page at its TLS name (page.mjs `PAGE_HOST`),
// a browser keeping its profile opens `args.apps[0]` from the list, and the
// group's signer, framed under it, adopts the open's approver and deletes
// the cookie the page left. The app saves (its page's `save`), then after a
// reload, then after the browser restarts, from its address alone. Opened
// again from the group's page, it saves under the newer open, and the
// signer refuses the first. `args.apps[1]`, opened too, is refused the
// first app's open, as it is and as its own. `args.apps[2]`, whose page
// keeps frames out in its own `<meta>`, is told why it can't save. The
// first app's open, replayed in a browser that never opened it, is refused
// at the app's door.
//
//   node web/tests/approve.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "profile": "<dir>", "apps": ["Money", "Notes", "Walled"]}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { appAt, held, inApp, launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = pageAt(server);
const logs = [];
const seen = {};
const watched = (b) => (
  b.context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`)),
  b.context.on('requestfailed', (r) =>
    logs.push(`requestfailed: ${r.url().slice(0, 160)} ${r.failure()?.errorText}`),
  ),
  b
);
// The handoff cookies the group's page left that no signer has taken,
// once the signer's deletion has reached the browser's cookie store: a
// frame's `document.cookie` write reaches it after the frame goes on, so
// read at once WebKit's store still held the deleted cookie (the Fedora gate
// image, demo/runs/flakes/2026-11-12). Asked again until none is left or
// 10 s have passed: a cookie never deleted is still counted.
const handoffs = async (b) => {
  const left = async () =>
    (await b.context.cookies(origin)).filter((c) => c.name.startsWith('__Host-svrngs-')).length;
  const until = Date.now() + 10_000;
  let n = await left();
  while (n > 0 && Date.now() < until) {
    await new Promise((r) => setTimeout(r, 100));
    n = await left();
  }
  return n;
};
const ready = (page) =>
  page.waitForFunction(() => document.title === 'ready', null, { timeout: 90_000 });
// From the group's page at `page`, `name` opened, its page drawn.
async function opening(page, name) {
  await page.goto(origin);
  await page.waitForSelector('#status.good', { timeout: 90_000 });
  await Promise.all([
    page.waitForURL((u) => inApp(u) && !u.pathname.endsWith('/open.html'), { timeout: 90_000 }),
    page.locator('#lent button', { hasText: name }).click(),
  ]);
  await ready(page);
}
const save = (page, what) => page.evaluate((w) => window.save(w), what);
// The open this app's origin keeps (store.js `lendKept`), as its signer reads it.
const kept = (page) =>
  page.evaluate(async () => {
    const v = new URL(document.querySelector('script[src*="shim.js"]').src).searchParams.get('v');
    const store = await import(`/store.js?v=${v}`);
    const {
      key: _key,
      line: _line,
      unsent: _unsent,
      ...open
    } = await store.lendKept(`pass ${store.labelOf(location.hostname)}`);
    return open;
  });
// The group's signer, framed at `page`, asked whether this browser's group
// page issued `opening`: what it answers.
const asked = (page, opening) =>
  page.evaluate(
    async ([opening, signer]) => {
      const frame = Object.assign(document.createElement('iframe'), { src: signer, hidden: true });
      const loaded = new Promise((done) => (frame.onload = done));
      document.body.append(frame);
      await loaded;
      const { port1, port2 } = new MessageChannel();
      const said = new Promise((done) => (port1.onmessage = (e) => done(e.data)));
      frame.contentWindow.postMessage({ opening }, new URL(signer).origin, [port2]);
      return said;
    },
    [opening, `${origin}sign.html`],
  );

let browser = watched(await launch(args.engine, { profile: args.profile }));
let other;
try {
  const [app, another, walled] = args.apps;
  let page = await browser.context.newPage();
  await held(page, origin, args);
  await opening(page, app);
  const at = page.url();
  seen.opened = { cookies: await handoffs(browser), saved: await save(page, 'one') };
  const first = await kept(page);
  await page.reload();
  await ready(page);
  seen.reloaded = await save(page, 'two');
  await browser.close();
  browser = watched(await launch(args.engine, { profile: args.profile }));
  page = await browser.context.newPage();
  await page.goto(at);
  await ready(page);
  seen.restarted = await save(page, 'three');
  const again = await browser.context.newPage();
  await opening(again, app);
  const second = await kept(again);
  seen.again = {
    cookies: await handoffs(browser),
    saved: await save(again, 'four'),
    older: await asked(again, first),
    newer: await asked(again, second),
    newKey: second.pass !== first.pass,
  };
  const theirs = await browser.context.newPage();
  await opening(theirs, another);
  seen.another = {
    as: await asked(theirs, second),
    relabelled: await asked(theirs, { ...second, label: (await kept(theirs)).label }),
  };
  const keptOut = await browser.context.newPage();
  await opening(keptOut, walled);
  seen.walled = await save(keptOut, 'five');
  other = await launch(args.engine, { unopened: true });
  const replayed = await other.context.newPage();
  const fragment = Buffer.from(JSON.stringify(second)).toString('base64url');
  await replayed.goto(`${appAt(second.label, server.address().port)}/open.html#${fragment}`);
  await replayed.waitForFunction(
    () => !document.getElementById('said').textContent.includes('Opening'),
    null,
    { timeout: 60_000 },
  );
  seen.replayed = await replayed.locator('#said').innerText();
  console.log(JSON.stringify({ ...seen, logs }));
} catch (e) {
  console.error(
    `the approval run did not finish: ${e}\n${JSON.stringify(seen)}\n${logs.join('\n')}`,
  );
  process.exitCode = 1;
} finally {
  await other?.close();
  await browser.close();
  server.close();
}
