// Demo A's three on this host (check.sh `trio`; u5-9, demo/a.md): each
// a browser of their own, entering as person.mjs drives one — Alex on a
// computer at the lend's origin the bare command printed, Bo and Cy on
// phones through the page by the group's link — each signing in once
// with the account the lender made and playing a film of their own. Then,
// once all are playing, `window_s` with all three at once: each seeks
// through jellyfin-web's own seek bar on a schedule of their own, spread
// over the window, while every page's <video> is watched. Demo A's seek
// bar is judged from this as a rate (u5-9's row): every seek's time to
// play again is kept, with the host's 1-minute load at that moment.
//
// With `mw` (check.sh trio, MW=1), through Jellyfin's middleware: nothing
// typed, each person signed in by the page it wrote (person.mjs).
//
//   node trio.mjs '{"server": "<Jellyfin's id>", "window_s": 600, "mw"?: true,
//     "people": [{"name", "user", "way", "engine", "at", "film", "seeks": <how many>}, ...]}'
//   env: SECRETS, JF, ADMIN, DUMP, as drive.mjs's
//
// Prints one JSON line. A step that did not happen within its time is
// null with what the page showed: never a number guessed.
import os from 'node:os';
import { enter, lender, seek } from './person.mjs';

const args = JSON.parse(process.argv[2]);
const { SECRETS, JF, ADMIN, DUMP } = process.env;
const T0 = Date.now();
const jf = lender(JF, ADMIN);
const out = { window_s: args.window_s, mw: !!args.mw, from: new Date(T0).toISOString(), people: {} };
const load = () => +os.loadavg()[0].toFixed(2);

// Where each seek goes, in percent of the film: n points between 5 and
// 85, each 37 on from the last (mod 81), so forward and back in turn and
// the same on every run; `k` starts each person somewhere else. 85 % of
// a 1,200 s film leaves 180 s, more than the widest gap between seeks.
const where = (n, k) => Array.from({ length: n }, (_, i) => 5 + ((i * 37 + k * 13) % 81));

// In a page: the watch from `since` to `until`, leaving out the person's
// own seeks (`own`, [start, end] in the page's clock, each with a second
// after it): how long the film played on, every still of 5 s or more,
// and every event that starts the film over or holds it.
function over([since, until, own]) {
  const w = window.__w;
  const mine = (t) => own.some(([a, b]) => t >= a && t <= b + 1000);
  const s = w.samples.filter((x) => x.t >= since && x.t <= until);
  let played = 0, run = 0, runFrom = null;
  const stills = [];
  const close = (t) => { if (run >= 5000) stills.push({ from_s: +((runFrom - since) / 1000).toFixed(1), ms: Math.round(run) }); run = 0; runFrom = null; };
  for (let i = 1; i < s.length; i++) {
    const a = s[i - 1], b = s[i], dt = b.t - a.t;
    const on = !a.none && !b.none && b.ct > a.ct && !b.paused;
    if (on) played += dt;
    if (on || mine(b.t)) close(b.t);
    else { runFrom ??= a.t; run += dt; }
  }
  close(until);
  const ev = w.events.filter((x) => x.t >= since && x.t <= until && !mine(x.t));
  const count = (e) => ev.filter((x) => x.e === e).length;
  return {
    samples: s.length, played_s: +(played / 1000).toFixed(1), stills_5s: stills,
    outside_own_seeks: { emptied: count('emptied'), loadstart: count('loadstart'), element: count('element'),
      waiting: count('waiting'), stalled: count('stalled'), error: count('error'), ended: count('ended') },
  };
}

// One person's seeks over the window: the i-th at start + i·gap, or as
// soon as the last is done if it ran over.
async function seeking(person, pcts, start, gap) {
  const done = [];
  for (let i = 0; i < pcts.length; i++) {
    const wait = start + (i + 1) * gap - Date.now();
    if (wait > 0) await person.page.waitForTimeout(wait);
    const from = await person.page.evaluate(() => performance.now());
    const at_s = Math.round((Date.now() - start) / 1000);
    const load1 = load();
    const s = await person.page.evaluate(seek, pcts[i]).catch((e) => ({ pct: pcts[i], ms: null, error: String(e).split('\n')[0] }));
    person.own.push([from, from + (s.ms ?? 30_000)]);
    done.push({ ...s, at_s, load1 });
  }
  return done;
}

// Every request for the film's bytes a person's browser makes — the file
// a range at a time, or a converted film's segments — with when it was
// asked, answered or given up, in ms from T0: where a seek waits.
function media(person) {
  const asked = (person.media = []);
  const film = (r) => /\/videos\//i.test(r.url());
  const short = (u) => u.replace(/^https?:\/\/[^/]+/, '').replace(/\?.*$/, '').replace(/^\/videos\/[^/]+\//i, '');
  const open = new Map();
  person.page.on('request', (r) => {
    if (!film(r)) return;
    const e = { path: short(r.url()), range: r.headers().range ?? null, asked: Date.now() - T0 };
    open.set(r, e);
    asked.push(e);
  });
  person.page.on('requestfinished', async (r) => {
    const e = open.get(r);
    if (!e) return;
    open.delete(r);
    e.done = Date.now() - T0;
    e.status = (await r.response().catch(() => null))?.status() ?? null;
  });
  person.page.on('requestfailed', (r) => {
    const e = open.get(r);
    if (!e) return;
    open.delete(r);
    e.failed = Date.now() - T0;
    e.why = r.failure()?.errorText ?? null;
  });
}

// A phone's lend worker keeps its last 2,000 lines at <scope>svrngs-trace
// (web/lend.js): what it did with each request it could not carry, and
// each range from asked to its end (u5-11), kept whole.
const workerTrace = (person) => person.page.evaluate(async () => {
  const r = await fetch('/svrngs-trace');
  return r.ok ? (await r.text()).split('\n') : `${r.status}`;
}).catch((e) => `could not read: ${e}`);

// What the lender's Jellyfin says each person's play is: how it plays
// (as it is, repackaged or converted) and at what bitrate.
async function streams() {
  const all = await jf('GET', '/Sessions');
  return all.filter((s) => s.NowPlayingItem).map((s) => ({
    user: s.UserName, device: s.DeviceId, item: s.NowPlayingItem.Name, method: s.PlayState?.PlayMethod ?? null,
    bitrate: s.TranscodingInfo?.Bitrate ?? null, video: s.TranscodingInfo?.VideoCodec ?? null,
    direct: s.TranscodingInfo ? { video: s.TranscodingInfo.IsVideoDirect, audio: s.TranscodingInfo.IsAudioDirect } : null,
    reasons: s.TranscodingInfo?.TranscodeReasons ?? null, paused: s.PlayState?.IsPaused ?? null,
  }));
}

const people = [];
try {
  const entered = await Promise.all(args.people.map((p, k) => enter(p, {
    args: { way: p.way, engine: p.engine, at: p.at, server: args.server, mw: args.mw }, SECRETS, DUMP, T0,
  }).then((person) => Object.assign(person, { way: p.way, engine: p.engine, own: [], pcts: where(p.seeks, k) }))));
  people.push(...entered);
  for (const p of people) if (p.page) media(p);
  const playing = people.filter((p) => p.play_ms != null);
  out.all_playing = playing.length === people.length;
  out.load_at_start = load();
  const start = Date.now();
  out.window_from_ms = start - T0;
  const since = await Promise.all(playing.map((p) => p.page.evaluate(() => performance.now())));
  const mid = new Promise((r) => setTimeout(r, (args.window_s * 1000) / 2)).then(streams).catch((e) => ({ error: String(e) }));
  await Promise.all(playing.map(async (p) => {
    p.seeks = await seeking(p, p.pcts, start, (args.window_s * 1000) / (p.pcts.length + 1));
  }));
  const rest = start + args.window_s * 1000 - Date.now();
  if (rest > 0) await new Promise((r) => setTimeout(r, rest));
  out.window_ms = Date.now() - start;
  out.load_at_end = load();
  out.streams_mid = await mid;
  out.streams_end = await streams().catch((e) => ({ error: String(e) }));
  for (const [i, p] of playing.entries()) {
    const until = await p.page.evaluate(() => performance.now());
    p.window = await p.page.evaluate(over, [since[i], until, p.own]);
    p.signed_in_after = await p.page.evaluate(async () => {
      try { const u = await window.ApiClient.getJSON(window.ApiClient.getUrl('Users/Me')); return u.Name; } catch (e) { return `refused: ${e?.status ?? e}`; }
    }).catch((e) => `could not ask: ${e}`);
    if (p.way === 'phone') p.worker_trace = await workerTrace(p);
  }
  out.devices_apart = new Set(people.map((p) => p.device)).size === people.length;
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  for (const p of people) {
    const { page, close, token, tally, own, pcts, ...rest } = p;
    out.people[p.user] = { ...rest, token: token ? 'held' : null, received: { ...tally }, logs: p.logs.slice(-30) };
    await close();
  }
  // A person's own token rides in the film's URLs (`ApiKey=`), and so in
  // the worker's trace: theirs, and kept out of what a run commits.
  out.until = new Date().toISOString();
  console.log(JSON.stringify(out).replace(/(api_?key=)[0-9a-f]{32}/gi, '$1<their token>'));
}
