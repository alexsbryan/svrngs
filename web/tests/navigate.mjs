// The browser half of capability-navigation's test (crates/svrngs-node/
// tests/browser, `a_navigation_from_another_lend_s_page_is_judged_as_its_fetch`):
// a lend's page on a computer, `args.url`, typed into the address bar, then
// moving within itself by a link. From it, into each lend of `args.into`,
// the page reaches four ways: a fetch, a frame, a form posted as the whole
// tab, and a link the person follows, each at the address its
// `/svrngs-reads` gives it, as an app builds on it (a capability in the
// path, give-session), or at the plain origin of a lend it does not read;
// and, a fifth, a fetch of the plain origin, `bare`, which carries none.
// The Rust half reads what each lend was asked; this half says what the
// browser was answered.
//
//   node web/tests/navigate.mjs '{"engine": "chromium"|"webkit", "url": "<the page's origin>",
//     "into": ["<another lend's origin>", ...]}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const { context, close } = await launch(args.engine);
const answered = [];
context.on('response', (r) => r.request().isNavigationRequest() && answered.push(`${r.request().method()} ${r.url()} ${r.status()}`));
const follow = (page, href) =>
  Promise.all([
    page.waitForEvent('load'),
    page.evaluate((href) => Object.assign(document.body.appendChild(document.createElement('a')), { href }).click(), href),
  ]);
try {
  const page = await context.newPage();
  await page.goto(args.url);
  await follow(page, `${args.url}own/next`);
  const within = page.url();
  const into = [];
  const reads = await page.evaluate(() => fetch('/svrngs-reads').then((r) => r.json()));
  for (const plain of args.into) {
    const to = reads.lends.find((l) => l.address.startsWith(plain))?.address ?? plain;
    await page.goto(args.url);
    const bare = await page.evaluate((to) => fetch(`${to}bare`).then((r) => r.status, (e) => `${e}`), plain);
    const fetched = await page.evaluate((to) => fetch(`${to}fetched`).then((r) => r.status, (e) => `${e}`), to);
    const framed = await page.evaluate(
      (to) =>
        new Promise((done) => {
          const f = Object.assign(document.createElement('iframe'), { src: `${to}framed` });
          f.onload = () => done('loaded');
          document.body.append(f);
          setTimeout(() => done('no load in 10 s'), 10_000);
        }),
      to,
    );
    await Promise.all([
      page.waitForEvent('load'),
      page.evaluate((to) => {
        const form = Object.assign(document.body.appendChild(document.createElement('form')), { method: 'post', action: `${to}posted` });
        form.submit();
      }, to),
    ]);
    const posted = page.url();
    await page.goto(args.url);
    await follow(page, `${to}linked`);
    into.push({ to, bare, fetched, framed, posted, linked: page.url() });
  }
  console.log(JSON.stringify({ within, into, answered }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${answered.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
}
