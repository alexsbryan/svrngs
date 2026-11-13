// The browser half of crates/svrngs-node/tests/browser/acts.rs
// `a_phone_s_page_reading_past_the_rate_is_refused_with_the_words`
// (EXTENSION_POINT §7, G2, on a phone): the group's page, holding the group
// `args.token` joins, opens the lend `args.open` from its list, the yes;
// at that lend's origin its page asks `/svrngs-reads`, then reads the lend
// `args.read` at the address it is told, as fast as it can, until svrngs
// refuses a read or `args.most` are taken. It says how many were taken,
// in how many ms from the first read's asking to the last answer (`ms`, so
// the rate's refill over the run is judged by its length, not assumed
// fast), and what the refusal said: its status, `x-svrngs-refused`,
// `Retry-After`, and its `error`.
//
//   node web/tests/read-rate.mjs '{"engine": "chromium"|"webkit", "site": ...,
//     "key": ..., "token": ..., "open": "<the lend's name>", "read": "<the read lend's name>", "most": <reads>}'
//
// Exit 0 with the JSON line, a step that did not finish said in it as
// `error`; 3, naming the install command, when Playwright or its browser
// is absent; 1 on anything else.
import { driving, inApp } from './page.mjs';

const args = JSON.parse(process.argv[2]);
await driving(args, async ({ page }) => {
  const seen = {};
  try {
    await page.locator('#lent button', { hasText: args.open }).click({ timeout: 90_000 });
    await page.waitForURL((u) => inApp(u) && !u.pathname.endsWith('/open.html'), {
      timeout: 90_000,
    });
    await page.waitForLoadState('load');
    Object.assign(
      seen,
      await page.evaluate(
        async ([read, most]) => {
          const reads = await fetch('/svrngs-reads').then((r) => r.json());
          const at = reads.lends.find((l) => l.name === read)?.address;
          if (!at) return { error: `no ${read} in ${JSON.stringify(reads)}` };
          const began = performance.now();
          const ms = () => Math.round(performance.now() - began);
          for (let taken = 0; taken < most; taken += 1) {
            const r = await fetch(at);
            if (r.status !== 200)
              return {
                taken,
                ms: ms(),
                status: r.status,
                refused: r.headers.get('x-svrngs-refused'),
                retry: r.headers.get('retry-after'),
                words: (await r.json().catch(() => ({}))).error ?? null,
              };
            await r.text();
          }
          return { taken: most, ms: ms() };
        },
        [args.read, args.most],
      ),
    );
  } catch (e) {
    seen.error = `the page did not finish: ${e}`;
  }
  return seen;
});
