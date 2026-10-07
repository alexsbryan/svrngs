// Demo B's dry run on this host (u6-9; demo/b.md; demo/runs/b/2026-10-06/
// README.md, what it measures and how each is read, committed before
// this). The TV is jellyfin-web at the room address (tv.mjs, u6-2's); the
// five are phones through the page by the group's link (person.mjs, way
// `phone`), one at a time, all signed in as the room's one account.
//
//   node night.mjs tv|five '<json: {room, link, server, profile, account,
//     people?: [{name, user, engine, device?, film}]}>'
//   env: SECRETS (the passwords), JF (the lender's Jellyfin), ADMIN (the
//   lender's token, to read its sessions and nothing else), DUMP (where
//   every phone's responses are written, and the TV's requests)
//
//   tv    before the evening, not timed: the TV opens the room address and
//         asks Quick Connect; Dee's phone, by the link, signed in as
//         `account`, types the code
//   five  the TV opened again from `profile`, as the evening; then each of
//         `people` in turn, timed from opening the link to the TV playing
//         the film they picked (its name), then pausing and seeking it from
//         their phone; with `direct`, the baseline with no svrngs anywhere:
//         the TV (signed in typing `account`'s) and each phone (a browser
//         as a computer's) straight at Sam's Jellyfin, `room` its /web/
//
// Prints one JSON line. A step that did not happen within its time is
// null with what the page showed: never a number guessed.
import fs from 'node:fs';
import { enter, lender, shown, timed, watch } from './person.mjs';
import { kept, open, playOn, screen, session, until, video } from './tv.mjs';

const [part, raw] = process.argv.slice(2);
const args = JSON.parse(raw);
const { SECRETS, JF, ADMIN, DUMP } = process.env;
const T0 = Date.now();
const jf = lender(JF, ADMIN);
const out = { part, room: args.room, from: new Date(T0).toISOString() };
const pw = (who) => fs.readFileSync(`${SECRETS}/${who}_pw`, 'utf8').trim();
const phone = (p) => enter({ name: p.name, user: p.user, account: args.account },
  { args: { way: args.direct ? 'computer' : 'phone', engine: p.engine, device: p.device, at: args.direct ? args.room : args.link, server: args.server }, SECRETS, DUMP, T0 });
const opened = [];

// In the TV: a video started since `since` (the TV's own clock) — a new
// element, or a load on it — and playing: moving, not paused.
const fresh = (s, since) => s.page.evaluate((since) => {
  const v = document.querySelector('video');
  const w = window.__w;
  const started = !!w && w.events.some((e) => (e.e === 'element' || e.e === 'loadstart') && e.t > since);
  return { started, ct: v ? +v.currentTime.toFixed(2) : null, paused: v?.paused ?? null };
}, since).catch((e) => ({ error: String(e).split('\n')[0] }));
const playing = (f) => f.started && f.ct > 0.5 && !f.paused;
// Where the lender's Jellyfin has the TV's play, in s.
const position = async (device) => ((await jf('GET', '/Sessions')).find((x) => x.DeviceId === device)?.PlayState?.PositionTicks ?? NaN) / 1e7;

// One person's turn: from the link to their film on the TV, then pause
// and seek from their phone.
async function turn(s, tv, p) {
  const r = { name: p.name, engine: p.engine, device: p.device ?? null, film: p.film };
  const ph = await phone(p);
  opened.push(ph);
  for (const k of ['browser', 'opened_at', 'list_ms', 'listed', 'open_ms', 'at', 'shown_ms', 'asked', 'shown', 'signed_in_ms']) r[k] = ph[k] ?? null;
  const stop = () => ({ ...r, stopped: ph.stopped ?? null });
  if (ph.signed_in_ms == null) return stop();
  const page = ph.page;
  // Jellyfin's home, the film they pick tapped there.
  r.film_ms = await timed(ph, 'the film', async () => {
    await page.locator('.card', { hasText: p.film }).first().locator('.cardImageContainer, a').first().click();
    await page.waitForURL((u) => u.hash.includes('/details'), { timeout: 30_000 });
    await page.locator('.mainDetailButtons .btnPlay:visible').first().waitFor({ timeout: 30_000 });
  });
  if (r.film_ms === null) return stop();
  // "Play on": the TV by the name, client and account its session has.
  const { cast, header, menu } = playOn(page);
  r.tv_named = `${tv.device_name} - ${tv.client} ${tv.user}`;
  r.play_on_ms = await timed(ph, '"Play on"', () => cast.first().waitFor({ timeout: 15_000 }));
  if (r.play_on_ms === null) return stop();
  // Each item's words as the menu shows them, the TV's found among them.
  r.play_on_lists = await menu();
  const at = r.play_on_lists.indexOf(r.tv_named);
  r.tv_listed = at >= 0;
  if (!r.tv_listed) {
    await page.keyboard.press('Escape');
    return { ...r, stopped: `the TV, "${r.tv_named}", not in "Play on"; the page says: ${await shown(page)}` };
  }
  await page.locator('[role=menuitem]:visible').filter({ hasText: /\S/ }).nth(at).click();
  await page.waitForTimeout(3000);
  r.header_after_pick = await header();
  // Play; the clock stops when the TV plays it.
  const since = await s.page.evaluate(() => performance.now());
  r.play_at = Date.now() - T0;
  await page.locator('.mainDetailButtons .btnPlay:visible').first().click({ force: true });
  // Both: the TV's <video> playing one started since, and the lender's
  // Jellyfin naming their film as the TV's — each time kept.
  const t0 = Date.now();
  let f = null, now = null;
  while (Date.now() - t0 < 60_000) {
    f = await fresh(s, since);
    now = await session(jf, tv.device);
    if (playing(f)) r.tv_video_ms ??= Date.now() - t0;
    if (now?.playing === p.film) r.tv_session_ms ??= Date.now() - t0;
    if (r.tv_video_ms != null && r.tv_session_ms != null) break;
    await s.page.waitForTimeout(100);
  }
  r.tv = f;
  r.session = now;
  r.session_at_s = await position(tv.device);
  r.phone_video = await video(page);
  if (r.tv_video_ms == null || r.tv_session_ms == null) return { ...r, stopped: `the TV did not play it within 60 s of Play; the TV says: ${await shown(s.page)}` };
  r.tv_playing_at = Date.now() - T0;
  r.tv_playing_ms = r.tv_playing_at - r.play_at;
  r.clock_ms = r.tv_playing_at - r.opened_at;
  r.within_3_min = r.clock_ms <= 180_000;
  // Pause, seek half way, play on: the same button's "Remote Control".
  await page.waitForTimeout(3000);
  r.header_after_play = await header();
  r.menu_after_play = await menu();
  const rc = page.getByRole('menuitem', { name: 'Remote Control' });
  r.remote_control_offered = (await rc.count()) > 0;
  if (!r.remote_control_offered) {
    await page.keyboard.press('Escape');
    return r;
  }
  await rc.first().click();
  const pp = page.locator('.btnPlayPause:visible');
  await pp.first().waitFor({ timeout: 15_000 });
  await pp.first().click();
  r.paused = await until(s.page, (v) => v && v.paused, 30_000);
  // As a person drags it once it shows the film's length: jellyfin-web
  // seeks by a share of the length the phone holds, and asked before it
  // holds one sent the TV to 0 (Cy's phone, Three, trials t5 and t9).
  const t2 = Date.now();
  const length = () => [...document.querySelectorAll('.runtime')].filter((e) => e.offsetParent).map((e) => e.textContent.trim()).find((t) => /^(?!0:00$)\d+(:\d\d)+$/.test(t)) ?? null;
  r.length_shown_ms = await page.waitForFunction(length, null, { timeout: 15_000 }).then(() => Date.now() - t2, () => null);
  r.length_shown = await page.evaluate(length).catch(() => null);
  const dur = (await video(s.page))?.dur;
  r.seek_to_s = dur ? +(dur / 2).toFixed(1) : null;
  await page.evaluate(() => {
    const sl = document.querySelector('.nowPlayingPositionSlider');
    sl.value = '50';
    sl.dispatchEvent(new Event('input', { bubbles: true }));
    sl.dispatchEvent(new Event('change', { bubbles: true }));
  });
  // Sought when the TV stands half way: by its <video>, or, where the film
  // is sent from an offset and the <video> counts from 0, by where the
  // lender's Jellyfin has the TV's session.
  const t1 = Date.now();
  r.sought = { ms: null };
  while (Date.now() - t1 < 30_000) {
    const v = await video(s.page);
    const at = await position(tv.device);
    if (v && dur && (Math.abs(v.ct - dur / 2) < 5 || Math.abs(at - dur / 2) < 5)) { r.sought = { ms: Date.now() - t1, video: v, session_at_s: at }; break; }
    r.sought = { ms: null, video: v, session_at_s: at };
    await s.page.waitForTimeout(100);
  }
  await pp.first().click();
  const from = r.sought.video?.ct ?? 0;
  r.played_on = await until(s.page, (v) => v && !v.paused && v.ct > from + 0.3, 30_000);
  r.played_on.session_at_s = await position(tv.device);
  r.pause_seek_within_10s = [r.paused, r.sought, r.played_on].every((x) => x.ms !== null && x.ms <= 10_000);
  return r;
}

let s = null;
try {
  s = await screen(args.profile, { room: args.room, dir: DUMP, log: `screen-${part}.jsonl`, T0 });
  opened.push(s);
  // Every film request the TV makes: the range it asked and what came back.
  out.films = [];
  s.context.on('requestfinished', async (q) => {
    if (!/\/Videos\/[^/]+\/(stream|hls|master|main)/i.test(q.url())) return;
    const a = await q.response().catch(() => null);
    const h = a ? await a.allHeaders().catch(() => ({})) : {};
    out.films.push({ ms: Date.now() - T0, path: new URL(q.url()).pathname, range: (await q.allHeaders().catch(() => ({}))).range ?? null,
      status: a?.status() ?? null, content_range: h['content-range'] ?? null, length: h['content-length'] ?? null });
  });
  s.context.on('requestfailed', (q) => {
    if (/\/Videos\/[^/]+\/stream/i.test(q.url())) out.films.push({ ms: Date.now() - T0, path: new URL(q.url()).pathname, failed: q.failure()?.errorText ?? null });
  });
  if (part === 'tv') {
    out.tv = await open(s, null, args.room, pw);
    out.quick_connect_enabled = await jf('GET', '/QuickConnect/Enabled');
    if (out.tv.quick_connect_offered) {
      await s.page.locator('.btnQuick:visible').click();
      const code = await s.page.waitForFunction(() => {
        const t = document.body.innerText.match(/Enter code (\d[\d ]{5,})/);
        return t && t[1];
      }, null, { timeout: 15_000 }).then((h) => h.jsonValue(), () => null);
      out.code_shown = code !== null;
      if (code !== null) {
        const dee = await phone({ name: 'Dee', user: 'dee', engine: 'webkit' });
        opened.push(dee);
        out.dee = { listed: dee.listed, at: dee.at, signed_in_ms: dee.signed_in_ms, stopped: dee.stopped ?? null };
        if (dee.signed_in_ms != null) {
          out.approved_ms = await timed(dee, 'approved', async () => {
            await dee.page.evaluate(() => { location.hash = '#/quickconnect'; });
            await dee.page.fill('#txtQuickConnectCode', code.replace(/\s/g, ''));
            await dee.page.locator('button[type=submit]:visible').first().click();
            await dee.page.waitForFunction(() => document.body.innerText.includes('successfully authenticated'), null, { timeout: 15_000 });
          });
          const t0 = Date.now();
          out.tv_signed_in_ms = await s.page.waitForSelector('.card', { timeout: 30_000 }).then(() => Date.now() - t0, () => null);
          out.typed_on_tv = 0;
          out.kept = await kept(s.page);
          out.session = out.kept.device ? await session(jf, out.kept.device) : null;
        }
      }
    }
  } else if (part === 'five') {
    out.direct = !!args.direct;
    out.tv = await open(s, args.direct ? args.account : null, args.room, pw);
    if (out.tv.asked && !args.direct) throw new Error('the TV asks to sign in again: what it kept from before the evening is gone');
    out.kept = await kept(s.page);
    out.session = await session(jf, out.kept.device);
    if (!out.session) throw new Error('the TV has no session in the lender\'s Jellyfin');
    const tv = { ...out.session, device: out.kept.device };
    await s.page.evaluate(watch);
    out.people = [];
    for (const p of args.people) {
      const r = await turn(s, tv, p);
      out.people.push(r);
      console.error(`${p.name}: ${r.clock_ms ?? 'no'} ms${r.stopped ? `, stopped: ${r.stopped}` : ''}`);
    }
    out.within_3_min = out.people.filter((r) => r.within_3_min).length;
  } else {
    throw new Error(`no part ${part}: tv or five`);
  }
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  if (s?.asked) out.asked = s.asked;
  if (s?.logs) out.logs = s.logs.slice(-30);
  for (const p of opened.slice(1)) if (p.logs) out[`${p.user}_logs`] = p.logs.slice(-20);
  for (const p of opened) if (p?.close) await p.close().catch(() => {});
  console.log(JSON.stringify(out));
}
