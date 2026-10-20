// The browser half of V1.1.3's phone tests (crates/svrngs-node/tests/
// browser): the page holds a joined group, opens the lend `args.open` from
// its list and waits for its page's title, `args.title`. Given
// `args.ready`, it then writes that file and waits for a line `go` on
// stdin, which the Rust half sends once it has stopped the lend's
// computer; then it goes to the lend's address again, through the lend's
// worker, timed from the address given to the page shown.
//
// Given `args.socket`, a path, it asks for a socket there instead, through
// the shim: once the lend's page is open, once its computer is stopped,
// and then, having written a second file, `args.again`, on which the Rust
// half starts the computer again, once a second until one opens or 60 s
// pass.
//
//   node web/tests/night.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the lend's name>",
//     "title": "<its page's title>", "ready"?: "<file>",
//     "socket"?: "<path>", "again"?: "<file>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { driving } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const told = new Promise((done) => createInterface({ input: process.stdin }).on('line', (l) => l.includes('go') && done()));
const go = () => told;
// In the lend's page: a socket at `path`, opened or not within 10 s.
function opens(path) {
  return new Promise((done) => {
    const s = new WebSocket(`${location.origin.replace(/^http/, 'ws')}${path}`);
    const t = setTimeout(() => (s.close(), done('no answer in 10 s')), 10_000);
    s.onopen = () => (clearTimeout(t), s.close(), done('open'));
    s.onclose = (e) => (clearTimeout(t), done(`closed ${e.code}`));
  });
}
const shown = async (page) => ({ url: page.url(), title: await page.title(), text: await page.evaluate(() => document.body.innerText) });
await driving(args, async ({ page }) => {
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  await page.locator('#lent button', { hasText: args.open }).click();
  await page.waitForFunction((t) => document.title === t, args.title, { timeout: 60_000 }).catch(() => {});
  const opened = await shown(page);
  if (!args.ready) return { opened };
  if (args.socket) {
    const first = await page.evaluate(opens, args.socket);
    writeFileSync(args.ready, '');
    await go();
    const stopped = await page.evaluate(opens, args.socket);
    writeFileSync(args.again, '');
    const t0 = Date.now();
    let [tries, last] = [0, null];
    while (Date.now() - t0 < 60_000 && last !== 'open') {
      tries += 1;
      last = await page.evaluate(opens, args.socket);
      if (last !== 'open') await new Promise((r) => setTimeout(r, 1000));
    }
    return { opened, first, stopped, back: { last, tries, ms: Date.now() - t0 } };
  }
  writeFileSync(args.ready, '');
  await go();
  const t0 = Date.now();
  const res = await page.goto(opened.url, { timeout: 90_000 });
  const again = { ms: Date.now() - t0, status: res?.status() ?? null, refused: (await res?.headerValue('x-svrngs-refused')) ?? null, ...(await shown(page)) };
  return { opened, again };
});
process.stdin.destroy();
