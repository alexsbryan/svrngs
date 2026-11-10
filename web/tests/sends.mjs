// The browser half of crates/svrngs-node/tests/browser/sends.rs
// `the_send_tick_is_the_yes_to_a_send` (row 13; svrn-docs
// rings/current/CONTRACTS.md §6.6, §7): a carried app's page at `args.url`,
// whose sends.txt names the lend `args.label`'s kind, POSTs JSON to that
// lend's `args.path` through its own origin's `/svrngs-lend/`. First by its
// typed address, `before`; then the person's home page, `args.home`: the row
// of the index's button reading `args.words`, its fold opened, `open` what
// the send's tick says and what the Open would post; `ticked` with the send
// ticked; `unread` with its read unticked after; ticked again, the Open
// pressed, and the new tab's POST, `yes`; the home page reloaded, `again`,
// what the last yes left ticked.
//
//   node web/tests/sends.mjs '{"engine": "chromium"|"webkit", "url": "<the carried app's origin>",
//     "home": "<url>", "words": "<its button's words>", "label": "<the lend's label>", "path": "</v1/embeddings>"}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent.
import { arrived, launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
// The page's POST, and what it could read of the answer.
const sent = (page) =>
  page.evaluate(
    async ([label, path]) => {
      const r = await fetch(`/svrngs-lend/${label}${path}`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ input: ['a film'] }),
      });
      return { status: r.status, refused: r.headers.get('x-svrngs-refused'), body: await r.text() };
    },
    [args.label, args.path],
  );
// The row's ticks, the read's then the send's, and what its Open posts.
const ticks = async (row) => {
  const boxes = row.locator('details input[type=checkbox]');
  const send = row.locator('details label', { hasText: 'Sends to' });
  return {
    line: (await send.count()) ? await send.textContent() : null,
    checked: await Promise.all((await boxes.all()).map((b) => b.isChecked())),
    disabled: await Promise.all((await boxes.all()).map((b) => b.isDisabled())),
    reads: await row.locator('input[name=reads]').inputValue(),
    sends: await row.locator('input[name=sends]').inputValue(),
  };
};
const seen = {};
try {
  const page = await context.newPage();
  await page.goto(args.url);
  await arrived(page);
  seen.before = await sent(page);
  const home = await context.newPage();
  const row = home.locator('#home table.index tr', { hasText: args.words });
  const unfolded = async () => {
    await home.goto(args.home);
    await home.locator('#home table.index button').first().waitFor({ timeout: 60_000 });
    await row.locator('details summary').click({ timeout: 10_000 });
  };
  await unfolded();
  seen.open = await ticks(row);
  const read = row.locator('details input[type=checkbox]').first();
  const send = row.locator('details label', { hasText: 'Sends to' }).locator('input');
  await send.check({ timeout: 10_000 });
  seen.ticked = await ticks(row);
  await read.uncheck();
  seen.unread = await ticks(row);
  await read.check();
  const [tab] = await Promise.all([
    context.waitForEvent('page'),
    row.locator('button', { hasText: args.words }).click({ timeout: 10_000 }),
  ]);
  await arrived(tab);
  seen.yes = await sent(tab);
  await unfolded();
  seen.again = await ticks(row);
} catch (e) {
  seen.error = `the page did not finish: ${e}`;
}
await close();
console.log(JSON.stringify(seen));
