// The browser half of crates/svrngs-node/tests/browser/carried.rs
// `a_phone_s_line_past_the_budget_is_let_go_and_its_chain_rewound`: a phone
// opens an app whose budget of entries is spent on its computer and writes
// twice; it prints what each write was answered and the worker's trace,
// whose lines say which place in the chain each write was signed at.
//
//   node web/tests/refused-line.mjs '{"engine": ..., "site": ..., "key": ...,
//     "token": ..., "open": "<the app>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { held, inApp, launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const profile = mkdtempSync(join(tmpdir(), 'svrngs-refused-line-'));
const server = await serve(args.site);
const origin = pageAt(server);
const { context, close } = await launch(args.engine, { profile });
const logs = [];
const write = (page, description) =>
  page.evaluate(async (description) => {
    const res = await fetch('svrngs/acts', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ op: { kind: 'expense', amount_cents: 1, description } }),
    });
    return { status: res.status, refused: res.headers.get('x-svrngs-refused') };
  }, description);
try {
  const page = await context.newPage();
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await held(page, origin, args);
  const button = page.locator('#lent button', { hasText: args.open });
  await Promise.all([page.waitForURL(inApp, { timeout: 90_000 }), button.click()]);
  await page.waitForFunction((title) => document.title === title, args.open, { timeout: 60_000 });
  const first = await write(page, 'the bins');
  const second = await write(page, 'the milk');
  const trace = await page.evaluate(async () => await (await fetch('svrngs-trace')).text());
  console.log(JSON.stringify({ first, second, trace, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.closeAllConnections();
  server.close();
  rmSync(profile, { recursive: true, force: true });
}
