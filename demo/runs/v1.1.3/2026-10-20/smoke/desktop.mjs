// V1.1.3's smoke: a computer's tab at `url`, a lend's origin on a computer
// whose lender did not answer, as Desktop Chrome; its answer, page and a
// screenshot at `shot`.
//   node desktop.mjs '{"url": ..., "shot": ...}'
import { chromium, devices } from '/Users/alexsbryan/dev/svrngs/web/node_modules/playwright/index.mjs';
const args = JSON.parse(process.argv[2]);
const browser = await chromium.launch();
const page = await (await browser.newContext(devices['Desktop Chrome'])).newPage();
const t0 = Date.now();
const res = await page.goto(args.url, { timeout: 60_000 }).catch((e) => ({ error: String(e) }));
const out = { ms: Date.now() - t0, status: res?.status?.() ?? null, refused: (await res?.headerValue?.('x-svrngs-refused')) ?? null, title: await page.title(), text: await page.evaluate(() => document.body?.innerText ?? '') };
await page.screenshot({ path: args.shot });
console.log(JSON.stringify(out));
await browser.close();
