// The browser half of page-open (c)'s test (crates/svrngs/tests/
// pages/open.rs): the group's page, holding the group `args.token` joins
// as `args.key`, opened and read; then hidden, as a phone's page is when its
// person turns to another app: the file `args.ready` is written, and on a
// line `go` (the Rust half has lent something meanwhile) it is shown again.
// What #response said before and after, and the trace.
//
//   node web/tests/pages/open-shown.mjs '{"engine": "webkit", "site": ..., "key": ..., "token": ..., "ready": "<file>"}'
//
// Exit 0 with the JSON line; 1 on anything else.
import { writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { driving } from '../page.mjs';

const args = JSON.parse(process.argv[2]);
const go = new Promise((done) => createInterface({ input: process.stdin }).on('line', (l) => l.includes('go') && done()));
// The page hidden or shown, as the browser says it when a tab or an app is switched.
const seen = (page, state) => page.evaluate((v) => (Object.defineProperty(document, 'visibilityState', { value: v, configurable: true }), document.dispatchEvent(new Event('visibilitychange'))), state);
const said = (page) => page.evaluate(() => ({ status: document.getElementById('status').textContent, response: document.getElementById('response').textContent }));
await driving(args, async ({ page }) => {
  const before = await said(page);
  await seen(page, 'hidden');
  writeFileSync(args.ready, '');
  await go;
  await seen(page, 'visible');
  await page.waitForFunction((was) => document.getElementById('response').textContent !== was, before.response, { timeout: 30_000 }).catch(() => {});
  return { before, after: await said(page), lines: await page.evaluate(() => window.__lines) };
});
process.stdin.destroy();
