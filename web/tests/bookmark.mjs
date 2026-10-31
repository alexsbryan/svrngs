// The browser half of the computer's cold bookmark's test (crates/
// svrngs-node/tests/browser/navigations.rs, `a_lends_bookmark_opens_cold_
// on_a_computer`; svrn-docs rings/current/IN_THE_BROWSER.md §4): a browser
// svrngs opened its home page in, `args.home`, and no lend, goes to the
// lend `args.label`'s address at `args.at` on `args.port`, as a bookmark
// opens it, and says where it lands, every navigation answered on the way;
// then, from there, frames the lend `args.other`, opened nowhere, and says
// each navigation the frame made; and a browser svrngs opened nothing in
// goes to the same address.
//
//   node web/tests/bookmark.mjs '{"engine": ..., "home": "<url>", "port": N,
//     "label": "<label>", "other": "<label>", "at": "/a/path?n=1"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const at = (label) => `http://${label}.localhost:${args.port}`;
// Where `context` lands at the lend's address: its URL, title and words,
// and each navigation answered on the way.
async function landed(context) {
  const answered = [];
  context.on('response', (r) => r.request().isNavigationRequest() && answered.push(`${r.request().method()} ${r.url()} ${r.status()}`));
  const page = await context.newPage();
  await page.goto(`${at(args.label)}${args.at}`);
  return [page, { url: page.url(), title: await page.title(), text: await page.locator('body').innerText(), answered }];
}
const seen = {};
try {
  const opened = await launch(args.engine);
  let page;
  [page, seen.opened] = await landed(opened.context);
  const [answered, from] = [seen.opened.answered, seen.opened.answered.length];
  await page.evaluate((to) => new Promise((done) => (Object.assign(document.body.appendChild(document.createElement('iframe')), { src: `${to}/framed`, onload: done }), setTimeout(done, 10_000))), at(args.other));
  seen.framed = answered.splice(from);
  await opened.close();
  const unopened = await launch(args.engine, { unopened: true });
  [, seen.unopened] = await landed(unopened.context);
  await unopened.close();
} catch (e) {
  seen.error = `the page did not finish: ${e}`;
}
console.log(JSON.stringify(seen));
