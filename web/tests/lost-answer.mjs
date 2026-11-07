// The browser half of crates/svrngs-node/tests/browser/keep_going.rs's lost
// answer (KEEP_GOING_AT_HOME 19): a phone that opened an app posts an entry,
// and its way to the computer is cut once the app has it, before any answer.
// The page is told; the Rust half brings the way back, and the person checks,
// as a plain read of the same app. It prints what the post was answered and
// what the read said.
//
//   node web/tests/lost-answer.mjs '{"engine": ..., "site": ..., "key": ...,
//     "token": ..., "id": ..., "open": "<the lend's name>", "ready": "<file>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { driving, inApp } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const go = new Promise((done) =>
  createInterface({ input: process.stdin }).on('line', (l) => l.includes('go') && done()),
);
await driving(args, async ({ page }) => {
  const button = page.locator('#lent button', { hasText: args.open });
  await Promise.all([page.waitForURL(inApp, { timeout: 90_000 }), button.click()]);
  await page.waitForFunction(() => document.body.innerText.includes('hi'), null, {
    timeout: 60_000,
  });
  const posted = await Promise.race([
    page.evaluate(async () => {
      const res = await fetch('/took', { method: 'POST', body: 'an entry' });
      return {
        status: res.status,
        refused: res.headers.get('x-svrngs-refused'),
        words: await res.text(),
      };
    }),
    new Promise((_, no) => setTimeout(() => no(new Error('the post was not answered')), 150_000)),
  ]);
  writeFileSync(args.ready, 'told');
  await go;
  // The person looks: every try a read, none a post.
  const checked = await page.evaluate(async () => {
    const tries = [];
    for (const until = Date.now() + 60_000; Date.now() < until;) {
      const res = await Promise.race([
        fetch('/took'),
        new Promise((done) => setTimeout(() => done(null), 40_000)),
      ]);
      tries.push(res?.status ?? 'no answer in 40 s');
      if (res?.ok) return { text: await res.text(), tries };
      await new Promise((done) => setTimeout(done, 500));
    }
    return { text: null, tries };
  });
  return { posted, checked };
});
process.stdin.destroy();
