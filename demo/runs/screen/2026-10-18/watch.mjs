// The screen as a projector shows it, watched (rehearse.py): each group's
// screen it is told to open, `/?screen=<group>` (home.js `screen`), in
// headless Chromium at 1920x1080, read every 250 ms for what it shows; a
// line to `out` each time a page's reading changes, and a picture of a
// page when one of its moments first holds:
//   opened   its door first judged, no window "asking…";
//   mid      at least half of `expect` people in its door;
//   settled  its count reads `expect` people, all on now, `expect` lent;
//   finale   `founder`'s window drawn off, "did not answer".
// Pages to open, and when to stop, come from `control`, a JSON file
// rehearse.py rewrites: {"pages": {"<name>": "<url>"}, "stop": bool}.
//
//   node watch.mjs '{"web": "<repo>/web", "control": "<file>", "out": "<file>", "shots": "<dir>", "expect": 25, "founder": "Priya"}'
import { appendFileSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';

const args = JSON.parse(process.argv[2]);
const { chromium } = createRequire(join(args.web, 'package.json'))('playwright');
const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1920, height: 1080 } });
const pages = {};
const last = {};
const shot = {};
const line = (o) => appendFileSync(args.out, `${JSON.stringify({ t: Date.now() / 1000, ...o })}\n`);
const read = (page) =>
  page.evaluate(() => ({
    count: document.querySelector('section > p')?.textContent ?? null,
    windows: [...document.querySelectorAll('.door li')].map((w) => ({ name: w.querySelector('strong')?.textContent, class: w.className, lends: [...w.querySelectorAll('a')].map((a) => a.textContent), words: w.lastChild?.textContent })),
    since: [...document.querySelectorAll('aside ol li')].map((li) => li.textContent),
  }));
const moments = (s) => ({
  opened: s.windows.length > 0 && !s.windows.some((w) => w.words === 'asking…'),
  mid: s.windows.length >= Math.ceil(args.expect / 2),
  settled: s.count === `${args.expect} people · ${args.expect} on now · ${args.expect} things lent`,
  finale: s.windows.some((w) => w.name === args.founder && w.class === 'off'),
});
for (;;) {
  let control = {};
  try {
    control = JSON.parse(readFileSync(args.control, 'utf8'));
  } catch {}
  if (control.stop) break;
  for (const [name, url] of Object.entries(control.pages ?? {})) {
    if (pages[name]) continue;
    pages[name] = await context.newPage();
    pages[name].on('pageerror', (e) => line({ page: name, error: String(e) }));
    await pages[name].goto(url);
    line({ page: name, opened_url: url });
  }
  for (const [name, page] of Object.entries(pages)) {
    const s = await read(page).catch((e) => ({ error: String(e) }));
    const said = JSON.stringify(s);
    if (said !== last[name]) {
      last[name] = said;
      line({ page: name, ...s });
    }
    if (s.error) continue;
    for (const [m, holds] of Object.entries(moments(s))) {
      if (!holds || shot[`${name}-${m}`]) continue;
      shot[`${name}-${m}`] = true;
      const path = join(args.shots, `${name}-${m}.png`);
      await page.screenshot({ path });
      line({ page: name, moment: m, shot: path });
    }
  }
  await new Promise((r) => setTimeout(r, 250));
}
await browser.close();
