// Ana's two devices for run.py here (PREDICTIONS.md): her laptop,
// Playwright's Chromium at the laptop's own address for Sam's jellyfin,
// and her phone, its WebKit as an iPhone (web/tests/page.mjs) opening her
// device link, never typing a name, then Sam's jellyfin from the list.
// Each plays a film of its own by its page's play button and is watched
// as apps/jellyfin/check/person.mjs watches and seeks one; over a window
// both seek at once while Sam's Jellyfin is asked for its sessions every
// 2 s. Then the laptop's film is stopped, the phone plays on alone, the
// phone's is stopped, and Jellyfin is asked where each film was left.
//
//   node two.mjs '{"laptop": <origin>, "link": <device link>, "server",
//     "films": {"laptop": <id>, "phone": <id>}, "window_s": 180, "seeks": 4}'
//   env: JF (Sam's Jellyfin), ADMIN (Sam's token there)
//
// Prints one JSON line. A step that did not happen within its time is null
// with what the page showed: never a number guessed.
import { launch } from '../../../../web/tests/page.mjs';
import { lender, seek, watch } from '../../../../apps/jellyfin/check/person.mjs';

const a = JSON.parse(process.argv[2]);
const T0 = Date.now();
const at = () => Date.now() - T0;
const jf = lender(process.env.JF, process.env.ADMIN);
const out = { window_s: a.window_s, films: a.films, devices: {}, sessions: [] };
const words = (page) => page.evaluate(() => document.body?.innerText.slice(0, 300).replace(/\s+/g, ' ') ?? '').catch(() => '?');
const pause = (ms) => new Promise((r) => setTimeout(r, ms));

// trio.mjs's `over`, as it is there: the watch from `since` to `until`,
// leaving out the page's own seeks.
function over([since, until, own]) {
  const w = window.__w;
  const mine = (t) => own.some(([a, b]) => t >= a && t <= b + 1000);
  const s = w.samples.filter((x) => x.t >= since && x.t <= until);
  let played = 0, run = 0, runFrom = null;
  const stills = [];
  const close = () => { if (run >= 5000) stills.push({ from_s: +((runFrom - since) / 1000).toFixed(1), ms: Math.round(run) }); run = 0; runFrom = null; };
  for (let i = 1; i < s.length; i++) {
    const a = s[i - 1], b = s[i], dt = b.t - a.t;
    const on = !a.none && !b.none && b.ct > a.ct && !b.paused;
    if (on) played += dt;
    if (on || mine(b.t)) close();
    else { runFrom ??= a.t; run += dt; }
  }
  close();
  const ev = w.events.filter((x) => x.t >= since && x.t <= until && !mine(x.t));
  const count = (e) => ev.filter((x) => x.e === e).length;
  return {
    samples: s.length, played_s: +(played / 1000).toFixed(1), stills_5s: stills,
    outside_own_seeks: { emptied: count('emptied'), loadstart: count('loadstart'), element: count('element'),
      waiting: count('waiting'), stalled: count('stalled'), error: count('error'), ended: count('ended') },
  };
}

// What the page's video is now: where, playing or not, and which film.
const video = (page) => page.evaluate(() => {
  const v = document.querySelector('video');
  if (!v) return null;
  const item = (v.currentSrc.match(/\/videos\/([0-9a-f]{32})\//i) ?? [])[1] ?? null;
  return { ct: +v.currentTime.toFixed(2), paused: v.paused, rs: v.readyState, item, error: v.error?.message ?? null };
}).catch((e) => `could not ask: ${String(e).split('\n')[0]}`);

// One device: in, signed in or asked, its film playing and watched.
async function device(d) {
  const r = { engine: d.engine, reports: { start: [], progress: 0, stopped: [] }, logs: [] };
  const { context, close } = await launch(d.engine);
  r.close = close;
  const page = (r.page = await context.newPage());
  page.on('pageerror', (e) => r.logs.push(`pageerror: ${String(e).slice(0, 200)}`));
  page.on('response', (x) => { if (x.status() >= 400) r.logs.push(`${at()} http ${x.status()} ${x.request().method()} ${x.url().replace(/ApiKey=[^&]+/i, 'ApiKey=<its token>').slice(0, 160)}`); });
  // What jellyfin-web tells Jellyfin of this device's play.
  page.on('request', (q) => {
    const m = q.url().match(/\/Sessions\/Playing(\/Progress|\/Stopped)?(\?|$)/i);
    if (!m || q.method() !== 'POST') return;
    let b = {};
    try { b = JSON.parse(q.postData() ?? '{}'); } catch {}
    const said = { at: at(), item: b.ItemId ?? null, pos_s: b.PositionTicks == null ? null : +(b.PositionTicks / 1e7).toFixed(1), play_session: b.PlaySessionId ?? null };
    if (!m[1]) r.reports.start.push(said);
    else if (/Stopped/i.test(m[1])) r.reports.stopped.push(said);
    else r.reports.progress++;
  });
  const t0 = Date.now();
  if (d.way === 'phone') {
    await page.goto(d.at);
    await page.waitForFunction(
      () => !document.getElementById('join')?.hidden || /good|bad/.test(document.getElementById('status')?.className),
      null, { timeout: 90_000 }).catch(() => {});
    r.joined = await page.evaluate(() => ({ asked_name: !document.getElementById('join').hidden, status: document.getElementById('status').textContent })).catch((e) => String(e));
    r.joined_ms = Date.now() - t0;
    if (r.joined?.asked_name || !/You’re in/.test(r.joined?.status ?? '')) { r.stopped = `not in: ${JSON.stringify(r.joined)}`; return r; }
    try {
      await page.locator('#lent button', { hasText: 'jellyfin' }).click({ timeout: 60_000 });
      await page.waitForURL((u) => u.pathname.startsWith('/web/'), { timeout: 90_000 });
    } catch (e) { r.stopped = `the lend: ${String(e).split('\n')[0]}; ${await words(page)}`; return r; }
  } else {
    await page.goto(d.at).catch((e) => { r.stopped = `the lend: ${e}`; });
    if (r.stopped) return r;
  }
  r.open_ms = Date.now() - t0;
  r.shown_ms = await page.waitForSelector('#txtManualName, .manualLoginForm, .btnManual, .card', { timeout: 60_000 }).then(() => Date.now() - t0, () => null);
  if (r.shown_ms === null) { r.stopped = `no sign-in and no home: ${await words(page)}`; return r; }
  await page.waitForTimeout(1500);
  r.asked_sign_in = (await page.locator('#txtManualName:visible, .manualLoginForm:visible, .btnManual:visible').count()) > 0;
  if (r.asked_sign_in) { r.stopped = `asked to sign in: ${await words(page)}`; return r; }
  const stored = JSON.parse((await page.evaluate(() => localStorage.getItem('jellyfin_credentials'))) ?? '{}').Servers?.[0] ?? {};
  r.stored = { user: stored.UserId ?? null, token: stored.AccessToken === 'svrngs-stand-in' ? 'the stand-in' : stored.AccessToken == null ? null : 'not the stand-in' };
  r.device = await page.evaluate(() => localStorage.getItem('_deviceId2'));
  await page.evaluate(([id, s]) => { location.hash = `#/details?id=${id}&serverId=${s}`; }, [d.film, a.server]);
  const p0 = Date.now();
  try {
    const play = page.locator('.mainDetailButtons .btnPlay:visible');
    await play.first().waitFor({ timeout: 30_000 });
    await play.first().click({ force: true });
  } catch (e) { r.stopped = `no play button: ${String(e).split('\n')[0]}; ${await words(page)}`; return r; }
  // person.mjs's second tap: WebKit may hold a play asked too long after a tap.
  let heldSince = null;
  while (Date.now() - p0 < 30_000) {
    const v = await video(page);
    if (v?.ct > 0.5 && !v.paused) { r.play_ms = Date.now() - p0; break; }
    const held = v && v.paused && v.rs >= 3 && v.ct < 0.5;
    heldSince = held ? heldSince ?? Date.now() : null;
    if (held && !r.second_tap && Date.now() - heldSince >= 3000) {
      r.second_tap = Date.now() - p0;
      await page.tap('video').catch(() => page.mouse.click(200, 300));
      await page.locator('.videoOsdBottom .btnPause').click({ timeout: 5000 }).catch(() => {});
    }
    await page.waitForTimeout(250);
  }
  r.play_ms ??= null;
  if (r.play_ms === null) { r.stopped = `not playing 30 s after play: ${JSON.stringify(await video(page))}`; return r; }
  r.playing = await video(page);
  await page.evaluate(watch);
  return r;
}

// Every session of Ana's account, as Jellyfin lists it to Sam.
let ana = null;
async function sessions(phase) {
  const all = await jf('GET', '/Sessions').catch((e) => ({ error: String(e) }));
  if (!Array.isArray(all)) return out.sessions.push({ at: at(), phase, error: all });
  const hers = all.filter((s) => s.UserName?.startsWith('Ana-')).map((s) => ({
    id: s.Id, device: s.DeviceId, client: s.Client, name: s.DeviceName,
    now: s.NowPlayingItem?.Id ?? null, pos_s: s.PlayState?.PositionTicks == null ? null : +(s.PlayState.PositionTicks / 1e7).toFixed(1),
    paused: s.PlayState?.IsPaused ?? null, method: s.PlayState?.PlayMethod ?? null,
  }));
  ana ??= all.find((s) => s.UserName?.startsWith('Ana-'))?.UserId ?? null;
  out.sessions.push({ at: at(), phase, hers });
}

// One page's seeks over the window, the i-th at start + offset + i·gap.
async function seeking(r, pcts, start, gap, offset) {
  const done = [];
  for (let i = 0; i < pcts.length; i++) {
    const wait = start + offset + i * gap - Date.now();
    if (wait > 0) await pause(wait);
    const from = await r.page.evaluate(() => performance.now());
    const s = await r.page.evaluate(seek, pcts[i]).catch((e) => ({ pct: pcts[i], ms: null, error: String(e).split('\n')[0] }));
    r.own.push([from, from + (s.ms ?? 30_000)]);
    done.push({ ...s, at_s: Math.round((Date.now() - start) / 1000) });
  }
  return done;
}

// Stop a page's film as a person does: back from the player.
async function stop(r) {
  const before = await video(r.page);
  const t = at();
  await r.page.evaluate(() => history.back());
  const gone = await r.page.waitForFunction(() => { const v = document.querySelector('video'); return !v || v.paused; }, null, { timeout: 15_000 }).then(() => at() - t, () => null);
  await pause(3000);
  return { at: t, before, gone_ms: gone, stop_reported: r.reports.stopped.at(-1) ?? null };
}

// Where Jellyfin keeps each film for Ana's account.
async function kept(film) {
  for (const path of [`/UserItems/${film}/UserData?userId=${ana}`, `/Items/${film}?userId=${ana}`, `/Users/${ana}/Items/${film}`]) {
    const b = await jf('GET', path).catch(() => null);
    const u = b?.UserData ?? (b && 'PlaybackPositionTicks' in b ? b : null);
    if (u) return { asked: path.split('?')[0].replace(/[0-9a-f]{32}/g, '<id>'), pos_s: +(u.PlaybackPositionTicks / 1e7).toFixed(1), played: u.Played ?? null };
  }
  return null;
}

const devs = {};
let poll = null;
try {
  out.from = new Date(T0).toISOString();
  [devs.laptop, devs.phone] = await Promise.all([
    device({ way: 'computer', engine: 'chromium', at: a.laptop, film: a.films.laptop }),
    device({ way: 'phone', engine: 'webkit', at: a.link, film: a.films.phone }),
  ]);
  for (const r of Object.values(devs)) r.own = [];
  out.both_playing = devs.laptop.play_ms != null && devs.phone.play_ms != null;
  if (!out.both_playing) throw new Error('not both playing');
  poll = setInterval(() => sessions(out.phase).catch(() => {}), 2000);
  out.phase = 'window';
  const start = Date.now();
  out.window_from = at();
  const since = { laptop: await devs.laptop.page.evaluate(() => performance.now()), phone: await devs.phone.page.evaluate(() => performance.now()) };
  const gap = (a.window_s * 1000) / (a.seeks + 1);
  // Forward and back in turn, each page apart from the other's by half a gap.
  const [lp, pp] = [[20, 60, 35, 75], [70, 30, 55, 15]].map((p) => p.slice(0, a.seeks));
  [devs.laptop.seeks, devs.phone.seeks] = await Promise.all([
    seeking(devs.laptop, lp, start, gap, gap), seeking(devs.phone, pp, start, gap, gap * 1.5)]);
  const rest = start + a.window_s * 1000 - Date.now();
  if (rest > 0) await pause(rest);
  for (const k of ['laptop', 'phone']) {
    const r = devs[k];
    const until = await r.page.evaluate(() => performance.now());
    r.window = await r.page.evaluate(over, [since[k], until, r.own]);
    r.at_window_end = await video(r.page);
  }
  // The laptop's film stopped; the phone plays on 20 s alone.
  out.phase = 'laptop stopped';
  const phoneSince = await devs.phone.page.evaluate(() => performance.now());
  devs.laptop.stop = await stop(devs.laptop);
  await pause(20_000 - 3000);
  devs.phone.alone = await devs.phone.page.evaluate(over, [phoneSince, await devs.phone.page.evaluate(() => performance.now()), []]);
  devs.phone.alone.at_end = await video(devs.phone.page);
  out.phase = 'phone stopped';
  devs.phone.stop = await stop(devs.phone);
  clearInterval(poll);
  await sessions('after');
  out.kept = { laptop: await kept(a.films.laptop), phone: await kept(a.films.phone) };
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  if (poll) clearInterval(poll);
  for (const [k, r] of Object.entries(devs)) {
    const { page, close, own, ...rest } = r;
    out.devices[k] = { ...rest, logs: r.logs.slice(-30) };
    await close().catch(() => {});
  }
  out.ana_user = ana;
  out.until = new Date().toISOString();
  console.log(JSON.stringify(out).replace(/(api_?key=)[0-9a-f]{32}/gi, '$1<their token>'));
}
