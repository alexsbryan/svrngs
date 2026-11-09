// The browser half of crates/svrngs-node/tests/browser/opens.rs
// `a_carried_app_reads_the_lends_it_declares_at_its_own_origin`: a carried
// app's page at `args.url`, whose reads.txt names a kind, reaches lends at
// its own origin's `/svrngs-lend/<label>/`, the only address its policy
// lets it ask. At each step the page asks `/svrngs-reads`, then each lend
// of `args.tries` by GET, its `films` by POST and for a script, and says
// what it could read of each answer, its headers and body, and of its own
// cookies: `before`, opened by its typed address; `yes`, opened from the
// home page, `args.home`, by the index's button reading `args.words`, with
// the index's cells; `stopped`, after that yes's Stop under "What acts as
// you".
//
//   node web/tests/carried-reads.mjs '{"engine": "chromium"|"webkit", "url": "<the carried app's origin>",
//     "home": "<url>", "words": "<its button's words>", "tries": {"<name>": "<a lend's label>", …}}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent.
import { arrived, launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
// Everything the page can read of what it asked, by name.
const tried = (page) =>
  page.evaluate(async (tries) => {
    const read = async (r) => ({
      status: r.status,
      refused: r.headers.get('x-svrngs-refused'),
      headers: [...r.headers].map(([name, value]) => `${name}: ${value}`),
      body: await r.text(),
    });
    const at = (label, path) => `/svrngs-lend/${label}${path}`;
    const said = { reads: await read(await fetch('/svrngs-reads')) };
    for (const [name, label] of Object.entries(tries))
      said[name] = await read(await fetch(at(label, '/films?n=1')));
    said.post = await read(
      await fetch(at(tries.films, '/films'), { method: 'POST', body: 'a film' }),
    );
    said.script = await read(await fetch(at(tries.films, '/code.js')));
    said.cookie = document.cookie;
    return said;
  }, args.tries);
const seen = {};
try {
  const page = await context.newPage();
  await page.goto(args.url);
  await arrived(page);
  seen.before = await tried(page);
  const home = await context.newPage();
  const loaded = () => home.locator('#home p.address').waitFor({ timeout: 60_000 });
  await home.goto(args.home);
  await loaded();
  await home.locator('#home table.index button').first().waitFor({ timeout: 60_000 });
  seen.cells = await home.locator('#home table.index tr td:first-child').allTextContents();
  const [tab] = await Promise.all([
    context.waitForEvent('page'),
    home.locator('#home table.index button', { hasText: args.words }).click({ timeout: 10_000 }),
  ]);
  await arrived(tab);
  seen.yes = await tried(tab);
  await home.reload();
  await loaded();
  await home.locator('#acts summary').click({ timeout: 60_000 });
  await Promise.all([
    home.waitForEvent('load'),
    home
      .locator('#acts p', { hasText: 'as you. Since' })
      .locator('button', { hasText: 'Stop' })
      .click({ timeout: 10_000 }),
  ]);
  seen.stopped = await tried(tab);
} catch (e) {
  seen.error = `the page did not finish: ${e}`;
}
await close();
console.log(JSON.stringify(seen));
