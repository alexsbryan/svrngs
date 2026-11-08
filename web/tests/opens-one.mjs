// The browser half of crates/svrngs-node/tests/browser/opens.rs
// `a_carried_app_opens_one_lend_of_its_group_and_no_more`
// (docs/APP_ACCESS_AND_STOP.md A4, "Consent/scope preserved"): a carried
// app's page at `args.url`, which asks svrngs's opener at its own origin
// whether each of three lends opens, and writes what it was told under
// `#said`, then draws a link to one by it, "Open notes". The link is
// followed as a person follows it, in a tab of its own: where the opener
// sent it, `sent`, its status and `Location` as the browser saw them, and
// there the opened lend's page reads the lend of `args.read` it declared it
// reads, as an app builds on /svrngs-reads.
//
//   node web/tests/opens-one.mjs '{"engine": "chromium"|"webkit", "url": "<the carried app's origin>",
//     "read": "<the read lend's origin>"}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent.
import { arrived, launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
const seen = { sent: [] };
context.on('response', (r) => {
  if (new URL(r.url()).pathname === '/svrngs-to' && !r.url().includes('whether='))
    seen.sent.push({ status: r.status(), location: r.headers().location ?? null });
});
try {
  const page = await context.newPage();
  await page.goto(args.url);
  await arrived(page);
  await page.waitForFunction(() => document.title === 'tried', null, { timeout: 60_000 });
  seen.said = await page.locator('#said').innerText();
  const [tab] = await Promise.all([
    context.waitForEvent('page'),
    page.getByRole('link', { name: 'Open notes', exact: true }).click(),
  ]);
  await arrived(tab);
  seen.url = tab.url();
  seen.read = await tab.evaluate(async (read) => {
    const reads = await fetch('/svrngs-reads').then((r) => r.json());
    const to = reads.lends.find((l) => l.address.startsWith(read))?.address ?? read;
    const r = await fetch(to);
    const error = (await r.json().catch(() => ({}))).error ?? null;
    return { status: r.status, refused: r.headers.get('x-svrngs-refused'), error };
  }, args.read);
} catch (e) {
  seen.error = `the page did not finish: ${e}`;
}
await close();
console.log(JSON.stringify(seen));
