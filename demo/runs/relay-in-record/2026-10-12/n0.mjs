// relay-n0-off's run in a browser (run.py): a computer's home page read, and
// Cy's phone. The phone's steps are Demo C's rerun's
// (demo/runs/c/dry-2026-10-12-rerun/c.mjs, `phone`), its film a lent page's
// <video> rather than Jellyfin's.
//
//   node n0.mjs home  <json>    a home page at `url`, loaded again until its words hold
//                               every one of `until` or any of `or`, or `within` ms pass
//   node n0.mjs phone <json>    Cy: the link on a phone, named, the group's lends
//                               listed, `film` opened and playing
//
// Prints one JSON line. Every time is ms from `t0`; a step that did not
// happen within its time is null with what the page said, never a number
// guessed.
import { launch } from '../../../../web/tests/page.mjs';

const [mode, raw] = process.argv.slice(2);
const a = JSON.parse(raw);
const T0 = a.t0 ?? Date.now();
const at = () => Date.now() - T0;
const out = { mode };
const words = (page) => page.evaluate(() => document.body?.innerText.slice(0, 400).replace(/\s+/g, ' ') ?? '').catch(() => '?');

async function until(fn, ms, every = 250) {
  const end = Date.now() + ms;
  for (;;) {
    const v = await fn().catch(() => null);
    if (v) return v;
    if (Date.now() > end) return null;
    await new Promise((r) => setTimeout(r, every));
  }
}

const { context, close } = await launch(mode === 'phone' ? 'webkit' : 'chromium');
const page = await context.newPage();
const logs = [];
page.on('console', (m) => logs.push(`console: ${m.text().slice(0, 300)}`));
page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
page.on('response', (r) => { if (r.status() >= 400) logs.push(`${r.status()} ${r.request().method()} ${r.url().slice(0, 120)}`); });
try {
  if (mode === 'home') {
    const end = Date.now() + a.within;
    for (;;) {
      await page.goto(a.url).catch(() => {});
      await page.waitForFunction(() => !/Reading your groups/.test(document.body.innerText), null, { timeout: 10_000 }).catch(() => {});
      out.said = await page.locator('main').innerText().catch(() => '');
      const held = (a.until ?? []).every((w) => out.said.includes(w)) || (a.or ?? []).some((w) => out.said.includes(w));
      if (held) { out.at = at(); break; }
      if (Date.now() > end) { out.at = null; break; }
      await page.waitForTimeout(2000);
    }
  } else if (mode === 'phone') {
    await page.goto(a.link);
    out.opened = at();
    await page.waitForFunction(
      () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
      null, { timeout: 90_000 });
    out.asked_name = await page.evaluate(() => !document.getElementById('join').hidden);
    if (out.asked_name) {
      out.name_asked_at = at();
      await page.fill('#name', a.name);
      await page.click('#join button');
      out.named_at = at();
    }
    await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    out.status = { at: at(), said: await page.locator('#status').textContent() };
    out.lent = await until(async () => {
      const lent = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
      return lent.some((l) => l.includes(a.lend)) && lent;
    }, 90_000, 500);
    out.lent_at = out.lent ? at() : null;
    if (!out.lent) out.lent_said = await words(page);
    if (out.lent) {
      out.film_opened = at();
      await page.locator('#lent button', { hasText: a.lend }).first().click();
      await page.waitForURL((u) => u.hostname !== new URL(a.link).hostname, { timeout: 90_000 }).catch(() => {});
      out.film_at = page.url();
      await page.waitForSelector('video', { timeout: 60_000 }).catch(() => {});
      await page.evaluate(() => document.querySelector('video')?.play()).catch(() => {});
      out.playing = await until(() => page.evaluate(() => { const v = document.querySelector('video'); return v && v.currentTime > 1 && !v.paused; }), 60_000)
        .then((ok) => (ok ? at() : null));
      if (out.playing === null) out.film_said = await words(page);
    }
  } else {
    throw new Error(`no mode ${mode}`);
  }
} catch (e) {
  out.stopped = `${String(e).split('\n')[0]}; the page says: ${await words(page)}`;
} finally {
  out.logs = logs.slice(-60);
  await close();
  console.log(JSON.stringify(out));
}
