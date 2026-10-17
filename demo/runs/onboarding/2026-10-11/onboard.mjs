// v1-onboarding (f)'s two looks in a browser (run.sh): Chromium on a
// computer, as the group's page offers the line only there (web/app.js).
//
//   node onboard.mjs line <link>                 the line the group's page shows, as pasted
//   node onboard.mjs home <url> <group> <dir> <t0 ms>
//                                                 the home page at <url> until it lists <group>
//                                                 and says its folder is at <dir>: ms from t0
//
// Prints one JSON line. What did not show within its time is null, with
// what the page showed: never a number guessed.
import { launch } from '../../../../web/tests/page.mjs';

const [mode, ...args] = process.argv.slice(2);
const { context, close } = await launch('chromium');
const page = await context.newPage();
const out = { mode };
try {
  if (mode === 'line') {
    const t = Date.now();
    await page.goto(args[0]);
    await page.waitForFunction(() => document.getElementById('line')?.textContent?.startsWith('curl '), null, { timeout: 60_000 });
    out.shown_ms = Date.now() - t;
    out.line = await page.locator('#line').textContent();
  } else if (mode === 'home') {
    const [url, group, dir, t0] = args;
    out.url = url;
    const deadline = +t0 + 300_000;
    // The home page reads /v1/groups as it loads: until the group and its
    // folder are there, it is loaded again, as a person would.
    for (;;) {
      await page.goto(url).catch((e) => (out.last_error = String(e).split('\n')[0]));
      const listed = await page.waitForFunction(([g, d]) => {
        const h = [...document.querySelectorAll('h2')].some((e) => e.textContent === g);
        const f = [...document.querySelectorAll('p.folder')].some((e) => e.textContent.includes(d));
        return h && f;
      }, [group, dir], { timeout: 5_000 }).then(() => true, () => false);
      if (listed) { out.listed_ms = Date.now() - +t0; break; }
      if (Date.now() > deadline) { out.listed_ms = null; break; }
    }
    out.shown = (await page.locator('main').innerText().catch(() => '')).slice(0, 600);
  } else {
    throw new Error(`no mode ${mode}: line or home`);
  }
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  await close();
  console.log(JSON.stringify(out));
}
