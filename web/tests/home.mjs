// The browser half of lean-4 (c)'s test (crates/svrngs-node/tests/browser):
// the home page on a computer asks svrngs's question. Opened at
// `args.ask`, its `/ask?lend=<label>&app=<label>`, it is read for its
// heading, which the test holds to the one source (question.js).
//
//   node web/tests/home.mjs '{"engine": "chromium"|"webkit", "ask": "<url>", "read"?: "<selector>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await page.goto(args.ask);
  const heading = await page.locator('#home h2').first().textContent({ timeout: 30_000 });
  // With `read`, a selector, its first match's words too (U7: a removed
  // person's line).
  const read = args.read ? await page.locator(args.read).first().textContent({ timeout: 30_000 }) : null;
  console.log(JSON.stringify({ heading, read, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
