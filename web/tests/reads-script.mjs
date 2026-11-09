// The browser half of crates/svrngs-node/tests/browser/main.rs
// `a_phone_page_is_handed_a_lend_s_script_refused_at_its_own_origin`: the
// page at a lend's origin on a phone (WebKit) reads a lend that answers
// a script, and says what it could read of the answer and whether it ran.
//
//   node web/tests/reads-script.mjs '{"engine": "webkit", "site": ..., "key": ..., "token": ..., "open": "<the lend's name>"}'
import { driving, inApp } from './page.mjs';

const args = JSON.parse(process.argv[2]);
await driving(args, async ({ page }) => {
  const away = (url) => inApp(url) && !url.pathname.endsWith('/open.html');
  const button = page.locator('#lent button', { hasText: args.open });
  await button.waitFor({ timeout: 90_000 });
  await Promise.all([page.waitForURL(away, { timeout: 90_000 }), button.click()]);
  await page.waitForLoadState('load');
  const told = await page.evaluate(async () => (await fetch('/svrngs-reads')).json());
  const address = told.lends?.[0]?.address;
  const script = await page.evaluate(async (url) => {
    const res = await fetch(url);
    const headers = [...res.headers].map(([n, v]) => `${n}: ${v}`);
    const body = await res.text();
    let ran = null;
    let error = null;
    try {
      ran = (await import(new URL(`${url}`, location.href))).ran;
    } catch (e) {
      error = String(e);
    }
    return { status: res.status, headers, body, ran, error, origin: location.origin };
  }, `${address}code.js`);
  return { told, address, script };
});
