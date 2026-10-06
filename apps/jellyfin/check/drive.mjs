// Two people through one lend (check.sh `pair`; forked from u5-3's, at
// demo/runs/u5/2026-10-05-jellyfin/drive.mjs, its bars (a)-(c), u5-4's
// row's check). Each person is a browser of their own, Playwright's headless
// Chromium or its WebKit as an iPhone (web/tests/page.mjs): on `computer`
// they open the lend's origin on Alex's loopback, as the bare command
// prints it; on `phone` they open the group's link, give a name, and open
// the lend from the list. Each signs in to jellyfin-web with the account
// the lender made, typing it, and plays their film. Then the first seeks
// through the player's own seek bar while the second is watched, and the
// other way round; then the lender's Jellyfin sends each person's session
// a message, which only the live socket carries; then each asks who they
// are signed in as.
//
//   node drive.mjs '{"way": "computer"|"phone", "engine": "chromium"|"webkit",
//     "at": "<the lend's origin, or the group's link>", "server": "<Jellyfin's id>",
//     "people": [{"name", "user", "film", "seeks": [percent, ...]}, {...}]}'
//   env: SECRETS (the passwords), JF (the lender's Jellyfin, for its
//   message to each session), ADMIN (the lender's token, for that alone),
//   DUMP (where every response each browser receives is written, for scan.py)
//   A person may sign in with an `"account"` not their own (`user`, by
//   which they are keyed): check.sh's one-device plant, one account for
//   everyone, as U3's dry run's lend had. With `"seed": "<a Jellyfin user
//   id>"`, the web client's stored sign-in names that user with the
//   placeholder token `door` before the page loads, as U3's dry run's
//   driver wrote it (demo/runs/a/dry-2026-10-04/drive.mjs:41-47): what no
//   person can write, and what a lend keeping a token needs to play at all
//   (u5-3's probe); the plants alone use it.
//
// Prints one JSON line. A step that did not happen within its time is
// null with what the page showed: never a number guessed.
import fs from 'node:fs';
import { launch } from '../../../web/tests/page.mjs';

const args = JSON.parse(process.argv[2]);
const { SECRETS, JF, ADMIN, DUMP } = process.env;
const out = { way: args.way, engine: args.engine, people: {} };
// Every time this run reports is in ms from here.
const T0 = Date.now();

// In a page: watch its <video> — whichever element is the video now —
// every 200 ms, and every event that starts it over or holds it.
function watch() {
  const w = (window.__w = { samples: [], events: [], seen: new WeakSet() });
  const bind = (v) => {
    if (w.seen.has(v)) return;
    w.seen.add(v);
    w.events.push({ e: 'element', t: performance.now() });
    for (const e of ['emptied', 'loadstart', 'waiting', 'stalled', 'error', 'ended', 'seeking', 'seeked'])
      v.addEventListener(e, () => w.events.push({ e, t: performance.now(), ct: v.currentTime }));
  };
  const sample = () => {
    const v = document.querySelector('video');
    if (!v) return w.samples.push({ t: performance.now(), none: true });
    bind(v);
    w.samples.push({ t: performance.now(), ct: v.currentTime, paused: v.paused, seeking: v.seeking, rs: v.readyState });
  };
  // The element playing now is bound before any phase begins: only one
  // put in its place later counts.
  sample();
  setInterval(sample, 200);
}

// In a page: the watch from `since` on, judged by README.md's bar — no
// `emptied` or `loadstart`, never back more than 2 s, never still 5 s.
function judge(since) {
  const w = window.__w;
  const s = w.samples.filter((x) => x.t >= since);
  const ev = w.events.filter((x) => x.t >= since);
  let back = 0, still = 0, run = 0;
  for (let i = 1; i < s.length; i++) {
    const a = s[i - 1], b = s[i];
    if (a.none || b.none) { run += b.t - a.t; still = Math.max(still, run); continue; }
    back = Math.max(back, a.ct - b.ct);
    if (b.ct > a.ct) run = 0; else { run += b.t - a.t; still = Math.max(still, run); }
  }
  const count = (e) => ev.filter((x) => x.e === e).length;
  return {
    samples: s.length, emptied: count('emptied'), loadstart: count('loadstart'), element: count('element'),
    waiting: count('waiting'), error: count('error'), back_s: +back.toFixed(2), still_ms: Math.round(still),
    left_alone: count('emptied') === 0 && count('loadstart') === 0 && count('element') === 0 && back <= 2 && still < 5000,
  };
}

// In a page: seek through jellyfin-web's own seek bar, as a person drags
// it, to `pct` of the film; answer how long until it plays on from there.
async function seek(pct) {
  const s = document.querySelector('.osdPositionSlider');
  const v0 = document.querySelector('video');
  if (!s || !v0) return { ms: null, error: `no seek bar or no video; the page says ${document.body.innerText.slice(0, 160)}` };
  const target = (v0.duration * pct) / 100;
  const t0 = performance.now();
  s.value = String(pct);
  s.dispatchEvent(new Event('input', { bubbles: true }));
  s.dispatchEvent(new Event('change', { bubbles: true }));
  return new Promise((done) => {
    let from = null;
    const check = () => {
      const v = document.querySelector('video');
      const t = performance.now() - t0;
      const moving = v && !v.paused && !v.seeking && v.readyState >= 3;
      if (moving && from === null) from = v.currentTime;
      if (!moving) from = null;
      if (moving && from !== null && v.currentTime > from + 0.3) {
        return done({ pct, ms: Math.round(t), at: +v.currentTime.toFixed(2), target: +target.toFixed(2) });
      }
      if (t > 30_000) return done({ pct, ms: null, at: v?.currentTime ?? null, paused: v?.paused, seeking: v?.seeking, rs: v?.readyState, error: v?.error?.message ?? null });
      setTimeout(check, 20);
    };
    setTimeout(check, 50);
  });
}

const shown = (page) => page.evaluate(() => document.body.innerText.slice(0, 200).replace(/\s+/g, ' ')).catch(() => '?');
async function timed(person, what, step) {
  const t0 = Date.now();
  try {
    await step();
    return Date.now() - t0;
  } catch (e) {
    person.stopped = `${what}: ${String(e.message).split('\n')[0]}; the page says: ${await shown(person.page)}`;
    return null;
  }
}

// Every response a browser receives, and on a computer every socket frame,
// written to DUMP/<user>/ for scan.py; counted here.
function record(context, page, user) {
  const dir = `${DUMP}/${user}`;
  fs.mkdirSync(dir, { recursive: true });
  const index = fs.openSync(`${dir}/index.jsonl`, 'w');
  let n = 0;
  const tally = { responses: 0, bodies: 0, body_bytes: 0, no_body: 0, from_worker: 0, sockets: [], frames: 0 };
  context.on('response', async (r) => {
    const i = n++;
    tally.responses++;
    if (r.fromServiceWorker()) tally.from_worker++;
    let file = null;
    try {
      const body = await r.body();
      file = `${i}.bin`;
      fs.writeFileSync(`${dir}/${file}`, body);
      tally.bodies++;
      tally.body_bytes += body.length;
    } catch {
      tally.no_body++;
    }
    const headers = await r.allHeaders().catch(() => r.headers());
    fs.writeSync(index, JSON.stringify({ i, ms: Date.now() - T0, url: r.url(), status: r.status(), headers, file }) + '\n');
  });
  page.on('websocket', (ws) => {
    tally.sockets.push(ws.url().replace(/ApiKey=[^&]+/, 'ApiKey=<its token>'));
    ws.on('framereceived', (f) => {
      tally.frames++;
      fs.appendFileSync(`${dir}/frames.txt`, `${String(f.payload)}\n`);
    });
  });
  return tally;
}

async function enter(p) {
  const person = { user: p.user, name: p.name, account: p.account ?? p.user };
  const { context, close } = await launch(args.engine);
  person.close = close;
  person.page = await context.newPage();
  const page = person.page;
  person.tally = record(context, page, p.user);
  const logs = (person.logs = []);
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  page.on('console', (m) => { if (/^play\(\)/.test(m.text())) logs.push(`console.${m.type()}: ${m.text().slice(0, 200)}`); });
  page.on('requestfailed', (r) => logs.push(`failed: ${r.url().slice(0, 120)}: ${r.failure()?.errorText}`));
  page.on('response', (r) => { if (r.status() >= 400) logs.push(`http ${r.status()} ${r.request().method()} ${r.url().slice(0, 140)}`); });
  person.browser = context.browser()?.version() ?? null;
  // Every play() the page asks, and how the browser answers it: WebKit
  // refuses one asked too long after a tap, and jellyfin-web says nothing.
  // PLAY_WATCH=0 leaves play() alone, to see the watch change nothing.
  if (process.env.PLAY_WATCH !== '0') await context.addInitScript(() => {
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () {
      const p = play.call(this);
      p?.then(() => console.log('play() played'), (e) => console.log(`play() refused: ${e?.name}: ${e?.message}`));
      return p;
    };
  });
  if (p.seed) {
    await context.addInitScript(([user, server]) => {
      if (localStorage.getItem('jellyfin_credentials')) return;
      localStorage.setItem('jellyfin_credentials', JSON.stringify({ Servers: [{
        ManualAddress: location.origin, LocalAddress: location.origin, LastConnectionMode: 2, ManualAddressOnly: true,
        Name: 'jellyfin', Id: server, DateLastAccessed: Date.now(), UserId: user, AccessToken: 'door',
      }] }));
    }, [p.seed, args.server]);
  }

  if (args.way === 'phone') {
    person.list_ms = await timed(person, 'the list', async () => {
      await page.goto(args.at);
      await page.waitForFunction(
        () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
        null, { timeout: 120_000 });
      await page.fill('#name', p.name);
      await page.click('#join button');
      await page.waitForSelector('#status.good', { timeout: 120_000 });
    });
    if (person.list_ms === null) return person;
    person.listed = await page.evaluate(() => [...document.querySelectorAll('#lent button')].map((b) => b.textContent));
    person.open_ms = await timed(person, 'the lend', async () => {
      await page.locator('#lent button', { hasText: 'jellyfin' }).click();
      await page.waitForURL((u) => u.pathname.startsWith('/web/'), { timeout: 90_000 });
    });
  } else {
    person.open_ms = await timed(person, 'the lend', () => page.goto(args.at));
  }
  if (person.open_ms === null) return person;
  person.at = new URL(page.url()).host.replace(/^[a-z0-9]{20,}\./, '<label>.');

  // What jellyfin-web shows someone who never used it: a sign-in, or home.
  person.shown_ms = await timed(person, 'a sign-in or home', () => page.waitForSelector('#txtManualName, .manualLoginForm, .btnManual, .card', { timeout: 60_000 }));
  if (person.shown_ms === null) return person;
  await page.waitForTimeout(1500);
  person.asked = await page.locator('#txtManualName:visible, .manualLoginForm:visible, .btnManual:visible').count() > 0;
  person.shown = await shown(page);
  person.signed_in_ms = person.asked ? await timed(person, 'signed in', async () => {
    const pw = fs.readFileSync(`${SECRETS}/${person.account}_pw`, 'utf8').trim();
    const manual = page.locator('.btnManual:visible');
    if (await manual.count()) await manual.first().click();
    await page.fill('#txtManualName', person.account);
    await page.fill('#txtManualPassword', pw);
    await page.locator('.manualLoginForm button[type=submit]').click();
    await page.waitForSelector('.card', { timeout: 30_000 });
  }) : 0;
  if (person.signed_in_ms === null) return person;
  const stored = JSON.parse((await page.evaluate(() => localStorage.getItem('jellyfin_credentials'))) ?? '{}');
  person.token = stored.Servers?.[0]?.AccessToken ?? null;
  person.device = await page.evaluate(() => localStorage.getItem('_deviceId2'));
  // A token Jellyfin handed this browser is theirs; the placeholder is none.
  if (person.token && person.token !== 'door') fs.writeFileSync(`${DUMP}/${p.user}/token`, person.token);

  await page.evaluate(([id, s]) => { location.hash = `#/details?id=${id}&serverId=${s}`; }, [p.film, args.server]);
  person.play_from = Date.now() - T0;
  person.play_ms = await timed(person, 'playing', async () => {
    const play = page.locator('.mainDetailButtons .btnPlay:visible');
    await play.first().waitFor({ timeout: 30_000 });
    await play.first().click({ force: true });
    // Playing; or held — paused at the start with enough to play, 3 s on
    // end — when a person taps the screen for the player's buttons and
    // taps play: a second tap, counted, never hidden.
    const t0 = Date.now();
    let heldSince = null;
    while (Date.now() - t0 < 30_000) {
      const v = await page.evaluate(() => { const v = document.querySelector('video'); return v && { ct: v.currentTime, paused: v.paused, rs: v.readyState }; });
      if (v && v.ct > 0.5 && !v.paused) return;
      const held = v && v.paused && v.rs >= 3 && v.ct < 0.5;
      heldSince = held ? heldSince ?? Date.now() : null;
      if (held && !person.second_tap && Date.now() - heldSince >= 3000) {
        person.second_tap = Date.now() - t0;
        await page.tap('video').catch(() => page.mouse.click(200, 300));
        await page.locator('.videoOsdBottom .btnPause').click({ timeout: 5000 }).catch((e) => logs.push(`the second tap missed: ${String(e).split('\n')[0]}`));
      }
      await page.waitForTimeout(250);
    }
    throw new Error(`not playing 30 s after play${person.second_tap ? `, a second tap given at ${person.second_tap} ms` : ''}`);
  });
  if (person.play_ms === null) {
    person.video = await page.evaluate(() => {
      const v = document.querySelector('video');
      if (!v) return null;
      const b = [...Array(v.buffered.length).keys()].map((i) => [+v.buffered.start(i).toFixed(1), +v.buffered.end(i).toFixed(1)]);
      return { ct: v.currentTime, paused: v.paused, rs: v.readyState, ns: v.networkState, buffered: b, error: v.error?.message ?? null };
    }).catch((e) => String(e));
  }
  if (person.play_ms !== null) {
    person.playing = await page.evaluate(() => { const v = document.querySelector('video'); return { src: v.currentSrc.replace(/ApiKey=[^&]+/, 'ApiKey=<its token>').slice(0, 220), w: v.videoWidth, h: v.videoHeight }; });
    await page.evaluate(watch);
  }
  return person;
}

// One seeks through `seeks` while the other plays, watched.
// With nobody else, the seeks alone.
async function phase(seeker, watched, seeks) {
  const since = watched && (await watched.page.evaluate(() => performance.now()));
  const done = [];
  for (const pct of seeks) {
    done.push(await seeker.page.evaluate(seek, pct));
    await seeker.page.waitForTimeout(3000);
  }
  return { seeker: seeker.user, seeks: done, within_5s: done.filter((s) => s.ms !== null && s.ms <= 5000).length,
    watched: watched?.user ?? null, watch: watched ? await watched.page.evaluate(judge, since) : null };
}

// The lender's Jellyfin, which the driver asks as the lender: each person's
// sessions, and a message to one, which jellyfin-web shows only if its
// live socket carried it.
const jf = async (method, path, body) => {
  const r = await fetch(`${JF}${path}`, {
    method, body: body && JSON.stringify(body),
    headers: { 'content-type': 'application/json', authorization: `MediaBrowser Client="u5-4 check", Device="Sam computer", DeviceId="u5-4-setup", Version="1", Token="${ADMIN}"` },
  });
  return r.status === 204 ? null : r.json();
};
async function socket(person) {
  const all = await jf('GET', '/Sessions');
  const sessions = all.filter((s) => s.DeviceId === person.device);
  // None of this browser's own: Jellyfin sees the device whoever signed in
  // with, as on a lend carrying a token.
  if (!sessions.length) return { session: false, theirs_by_device: all.filter((s) => s.UserName === person.account).map((s) => s.DeviceId) };
  const text = `a message for ${person.user} ${Math.random().toString(36).slice(2, 8)}`;
  const t0 = Date.now();
  await jf('POST', `/Sessions/${sessions[0].Id}/Message`, { Header: 'Sam', Text: text, TimeoutMs: 15000 });
  const shownIn = await person.page.waitForFunction((t) => document.body.innerText.includes(t), text, { timeout: 10_000 }).then(() => Date.now() - t0, () => null);
  return { session: true, remote_control: sessions[0].SupportsRemoteControl, message_shown_ms: shownIn };
}

const people = [];
try {
  for (const p of args.people) people.push(await enter(p));
  const [a, b] = people;
  const playing = people.every((p) => p.play_ms != null);
  if (playing) {
    out.first_seeks = await phase(a, b, args.people[0].seeks);
    if (b) out.then_seeks = await phase(b, a, args.people[1].seeks);
  }
  const all = await jf('GET', '/Sessions');
  out.devices_apart = new Set(people.map((p) => p.device)).size === people.length;
  for (const p of people) {
    if (p.signed_in_ms == null || p.stopped) continue;
    p.socket = await socket(p).catch((e) => ({ error: String(e) }));
    p.signed_in_after = await p.page.evaluate(async () => {
      try { const u = await window.ApiClient.getJSON(window.ApiClient.getUrl('Users/Me')); return u.Name; } catch (e) { return `refused: ${e?.status ?? e}`; }
    }).catch((e) => `could not ask: ${e}`);
    p.sessions = all.filter((s) => s.DeviceId === p.device).map((s) => ({ user: s.UserName, playing: !!s.NowPlayingItem, client: s.Client }));
  }
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  for (const p of people) {
    const { page, close, token, tally, ...rest } = p;
    out.people[p.user] = { ...rest, token: token ? 'held' : null, received: { ...tally }, logs: p.logs.slice(-30) };
    await close();
  }
  console.log(JSON.stringify(out));
}
