// The browser half of capability-open's test (crates/svrngs-node/tests/
// browser, `opening_an_app_from_the_home_page_is_the_yes_to_what_it_reads`):
// a lend's page that reads another member's lend, on a computer. Opened
// first another way, its typed address as a bookmark opens it, it fetches
// `args.read` at `bookmark`; then the person opens it from their home page,
// `args.home`, by the index's button reading `args.words`, the yes, and the
// new tab fetches at `opened`, and the first tab again at `after`, and at
// the read lend's plain origin, `bare`, carrying no capability. Then a
// lend that reads nothing, `args.plain`, opened by the index's link. The
// Rust half reads what the read lend was asked; this half says what the
// page was answered, and every index cell's words.
//
//   node web/tests/opens.mjs '{"engine": "chromium"|"webkit", "home": "<url>",
//     "page": "<the reading lend's origin>", "read": "<the lend it reads' origin>",
//     "words": "<its button's words>", "plain": "<the other link's words>"}'
//
// Exit 0 with the JSON line, a step that did not finish said in it; 3,
// naming the install command, when Playwright or its browser is absent.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
// A fetch of the read lend from `page` at `path`, at the address the
// page's `/svrngs-reads` gives it, as an app builds on it (a capability in
// the path, give-session); `bare`, at its plain origin: svrngs's refusal is
// readable to the page, a lend's own answer is not, so the Rust half
// judges a reach by what the lend was asked.
const fetched = (page, path, bare) =>
  page.evaluate(
    async ([read, path, bare]) => {
      const reads = bare ? { lends: [] } : await fetch('/svrngs-reads').then((r) => r.json());
      const to = reads.lends.find((l) => l.address.startsWith(read))?.address ?? read;
      return fetch(`${to}${path}`).then(async (r) => ({ status: r.status, refused: r.headers.get('x-svrngs-refused'), error: (await r.json().catch(() => ({}))).error ?? null }), (e) => ({ status: `${e}` }));
    },
    [args.read, path, bare],
  );
const opened = async (click) => {
  const [tab] = await Promise.all([context.waitForEvent('page'), click]);
  await tab.waitForLoadState();
  return tab;
};
// What it saw, said whole even when a step does not finish, as `error`,
// so the Rust half judges each step it reached.
const seen = {};
try {
  const page = await context.newPage();
  await page.goto(args.page);
  seen.bookmark = await fetched(page, 'bookmark');
  const home = await context.newPage();
  await home.goto(args.home);
  await home.locator('#home table.index a, #home table.index button').first().waitFor({ timeout: 60_000 });
  seen.cells = await home.locator('#home table.index tr td:first-child').allTextContents();
  const tab = await opened(home.locator('#home table.index button', { hasText: args.words }).click({ timeout: 10_000 }));
  seen.there = { url: tab.url(), reached: await fetched(tab, 'opened') };
  seen.after = await fetched(page, 'after');
  seen.bare = await fetched(page, 'bare', true);
  seen.plain = (await opened(home.locator('#home table.index a', { hasText: args.plain }).click())).url();
} catch (e) {
  seen.error = `the page did not finish: ${e}`;
}
await close();
console.log(JSON.stringify(seen));
