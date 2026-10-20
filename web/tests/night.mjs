// The browser half of V1.1.3's two phone tests (crates/svrngs-node/tests/
// browser): the page holds a joined group, opens the lend `args.open` from
// its list and waits for its page's title, `args.title`. Given
// `args.ready`, it then writes that file and waits for a line `go` on
// stdin, which the Rust half sends once it has stopped the lend's
// computer; then it goes to the lend's address again, through the lend's
// worker, timed from the address given to the page shown.
//
//   node web/tests/night.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the lend's name>",
//     "title": "<its page's title>", "ready"?: "<file>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { driving } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const go = new Promise((done) => createInterface({ input: process.stdin }).on('line', (l) => l.includes('go') && done()));
const shown = async (page) => ({ url: page.url(), title: await page.title(), text: await page.evaluate(() => document.body.innerText) });
await driving(args, async ({ page }) => {
  await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
  await page.locator('#lent button', { hasText: args.open }).click();
  await page.waitForFunction((t) => document.title === t, args.title, { timeout: 60_000 }).catch(() => {});
  const opened = await shown(page);
  if (!args.ready) return { opened };
  writeFileSync(args.ready, '');
  await go;
  const t0 = Date.now();
  const res = await page.goto(opened.url, { timeout: 90_000 });
  const again = { ms: Date.now() - t0, status: res?.status() ?? null, refused: (await res?.headerValue('x-svrngs-refused')) ?? null, ...(await shown(page)) };
  return { opened, again };
});
process.stdin.destroy();
