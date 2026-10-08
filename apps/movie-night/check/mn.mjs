// Movie Night's journeys (check.sh; the bars in demo/runs/mn/2026-10-08/
// README.md §3). Each person is a browser of their own, Playwright's
// headless Chromium or its WebKit as an iPhone (web/tests/page.mjs), every
// response it receives written to DUMP/<person>/ for scan.py
// (apps/jellyfin/check/person.mjs `record`).
//
//   phone     each person new to the group opens its link, joins and opens
//             Movie Night from the list; searches `bladruner` cold, then
//             again until every library answered (a phone's person is new
//             to Dee's computer until its ring sync round), `brazl` after the
//             check kept Sam's Brazil from those `kept_from`, `bladruner`
//             warm; chooses their copy (`choose`, its lender) before their
//             phone ever opened that Jellyfin, and back; opens that Jellyfin
//             from the group's list; opens Movie Night again, searches,
//             chooses, presses Play, and Back until Movie Night.
//   asleep    Cy's computer awake while the person lists the group; asleep
//             (SLEEP) before Movie Night asks it; each library's every word
//             recorded as it changed.
//   computer  Alex opens Movie Night at his computer's origin for it: what
//             it finds with nothing handed; then with `lends` in its address,
//             the run's stand-in for discovery; chooses, plays, Back.
//   limit     two people: the first searches once, the second at each of
//             `after_s` seconds after the first's search ended; the doors'
//             logs say whom each counted (doors.py).
//   once      a person joins, opens Movie Night and searches `bladruner`
//             until every library answered (the plants' run).
//
//   node mn.mjs '{"journey", "engine": "chromium"|"webkit", "link"?, "at"?, "lends"?,
//     "labels": {"Sam": <his jellyfin's label>, "Dee": <hers>},
//     "people": [{"name", "choose"?: "Sam"|"Dee", "kept_from"?}], "after_s"?}'
//   env: DUMP; JF_SAM, SAM_ADMIN, MOVIES (Sam's Jellyfin, his token and his
//   Movies library, to keep Brazil from a person); SLEEP (puts Cy to sleep)
//
// Prints one JSON line. A step that did not happen within its time is null
// with what the page showed: never a number guessed.
import { execSync } from 'node:child_process';
import { launch } from '../../../web/tests/page.mjs';
import { record, shown, timed } from '../../jellyfin/check/person.mjs';

const args = JSON.parse(process.argv[2]);
const { DUMP, JF_SAM, SAM_ADMIN, MOVIES, SLEEP } = process.env;
const T0 = Date.now();
const out = { journey: args.journey, engine: args.engine, labels: args.labels, started: new Date(T0).toISOString(), people: {} };
const now = () => new Date().toISOString();
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

async function browser(p) {
  const user = p.name.toLowerCase();
  const person = { name: p.name, user, steps: [], logs: [] };
  const { context, close } = await launch(args.engine);
  person.close = close;
  person.page = await context.newPage();
  person.tally = record(context, person.page, user, { DUMP, T0 });
  const page = person.page;
  page.on('pageerror', (e) => person.logs.push(`pageerror: ${e}`));
  page.on('console', (m) => { if (m.type() === 'error' || m.type() === 'warning') person.logs.push(`console.${m.type()}: ${m.text().slice(0, 240)}`); });
  page.on('requestfailed', (r) => person.logs.push(`failed: ${r.url().slice(0, 120)}: ${r.failure()?.errorText}`));
  page.on('response', (r) => { if (r.status() >= 400) person.logs.push(`http ${r.status()} ${r.request().method()} ${r.url().slice(0, 140)}`); });
  person.browser = context.browser()?.version() ?? null;
  return person;
}

// ── the group's page ────────────────────────────────────────────────────

async function join(person) {
  const page = person.page;
  person.join_ms = await timed(person, 'the list', async () => {
    await page.goto(args.link);
    await page.waitForFunction(
      () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
      null, { timeout: 120_000 });
    await page.fill('#name', person.name);
    await page.click('#join button');
    await page.waitForSelector('#status.good', { timeout: 120_000 });
  });
  await listed(person);
}

async function listed(person) {
  person.listed = await person.page.evaluate(() => ({
    lent: [...document.querySelectorAll('#lent button')].map((b) => b.textContent),
    said: document.getElementById('response')?.textContent ?? '',
  })).catch((e) => ({ error: String(e) }));
  return person.listed;
}

// Back at the group's page, already joined: its list again.
async function group(person) {
  return timed(person, 'the group again', async () => {
    await person.page.goto(args.link);
    await person.page.waitForSelector('#status.good', { timeout: 120_000 });
    await listed(person);
  });
}

// From the group's list, the lend whose button names `what`.
async function openFromList(person, what, ready) {
  return timed(person, `${what} from the list`, async () => {
    await person.page.locator('#lent button', { hasText: what }).click();
    await person.page.waitForURL((u) => !u.href.startsWith(args.link.split('#')[0]), { timeout: 90_000 });
    await person.page.waitForSelector(ready, { timeout: 90_000 });
    // Movie Night's module runs before the document is complete; a query
    // typed before then is the browser's form, not Movie Night's.
    await person.page.waitForFunction(() => document.readyState === 'complete', null, { timeout: 60_000 });
  });
}

// ── Movie Night ─────────────────────────────────────────────────────────

// What Movie Night shows: its query, each library's words, the cards.
const showing = (page) => page.evaluate(() => ({
  url: location.href.replace(/#lends=.*/, '#lends=…'),
  query: document.getElementById('q')?.value ?? null,
  said: [...document.querySelectorAll('#said li')].map((l) => ({ library: l.dataset.library ?? null, state: l.dataset.state, text: l.textContent })),
  cards: [...document.querySelectorAll('#results .card')].map((c) => ({
    title: c.querySelector('h2').textContent,
    copies: [...c.querySelectorAll('a.copy')].map((a) => ({ library: a.dataset.library, text: a.textContent, first: a.classList.contains('first'), href: a.getAttribute('href') })),
  })),
}));

// Search `q`: the time from Enter to the first card, to each library's
// word changing and to the last library's answer, watched in the page
// itself, every change of every library's word kept.
async function search(person, q, limit = 180_000) {
  const page = person.page;
  await page.fill('#q', q);
  await page.evaluate(() => {
    const w = (window.__mn = { t0: performance.now(), first: null, seen: [], at: {} });
    const look = () => {
      const t = Math.round(performance.now() - w.t0);
      if (w.first === null && document.querySelector('#results .card')) w.first = t;
      for (const li of document.querySelectorAll('#said li')) {
        const k = li.dataset.library ?? 'none';
        if (w.at[k]?.text !== li.textContent) {
          w.at[k] = { state: li.dataset.state, text: li.textContent, t };
          w.seen.push({ library: k, state: li.dataset.state, text: li.textContent, t });
        }
      }
    };
    w.observer = new MutationObserver(look);
    w.observer.observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true });
  });
  const started = now();
  await page.press('#q', 'Enter');
  const settled = await page.waitForFunction(() => {
    const said = [...document.querySelectorAll('#said li')];
    return said.length && said.every((l) => l.dataset.state !== 'asking') && window.__mn.seen.length > 0;
  }, null, { timeout: limit, polling: 100 }).then(() => true, () => false);
  const w = await page.evaluate(() => { window.__mn.observer.disconnect(); const { t0, observer, ...w } = window.__mn; return w; });
  const s = await showing(page);
  const last = Math.max(...w.seen.filter((x) => x.state !== 'asking').map((x) => x.t), -1);
  const r = { query: q, started, ended: now(), first_card_ms: w.first, all_said_ms: settled ? last : null, seen: w.seen, ...s };
  person.steps.push({ search: r });
  return r;
}

const everyAnswered = (r) => r.said.length > 0 && r.said.every((l) => l.state === 'answered');
// With a computer asleep: every library answered but the one said away.
const awayOrAnswered = (r) => r.said.length > 0 && r.said.every((l) => l.state === 'answered' || l.state === 'away');

// Search `q` until `done` says so of it (else every library answered), at
// most `tries` times, `gap` ms apart: each search is kept.
async function settle(person, q, tries = 10, gap = 15_000, done = everyAnswered) {
  for (let i = 0; i < tries; i++) {
    const r = await search(person, q);
    if (done(r)) return { searches: i + 1, ...r };
    await wait(gap);
  }
  return null;
}

// Keep Sam's Brazil from `person`: their account, made by the
// middleware on their first request, sees Sam's Movies alone.
async function keepFrom(person) {
  const h = { 'content-type': 'application/json', authorization: `MediaBrowser Client="mn check", Device="sam computer", DeviceId="mn-sam-setup", Version="1", Token="${SAM_ADMIN}"` };
  const users = await (await fetch(`${JF_SAM}/Users`, { headers: h })).json();
  const u = users.find((x) => x.Name.startsWith(`${person.name}-`));
  if (!u) return { kept: false, why: `no account of ${person.name}'s in Sam's Jellyfin: ${users.map((x) => x.Name)}` };
  const policy = { ...u.Policy, EnableAllFolders: false, EnabledFolders: [MOVIES] };
  const res = await fetch(`${JF_SAM}/Users/${u.Id}/Policy`, { method: 'POST', headers: h, body: JSON.stringify(policy) });
  return { kept: res.ok, account: u.Name, status: res.status, at: now() };
}

// ── a Jellyfin ──────────────────────────────────────────────────────────

// Choose `who`'s copy of the first card: where the browser lands, and
// whether it is jellyfin-web at that item, signed in as the person.
async function choose(person, card, who) {
  const page = person.page;
  // Movie Night names no lender: their lend's label, from the run, picks it.
  const copy = card?.copies.find((c) => c.library === args.labels[who]);
  if (!copy) return { chose: null, why: 'no copy to choose' };
  const t0 = Date.now();
  const [nav] = await Promise.all([
    page.waitForNavigation({ timeout: 60_000 }).catch(() => null),
    page.click(`a.copy[data-library="${copy.library}"]`),
  ]);
  const r = { chose: copy.text, library: copy.library, status: nav?.status() ?? null, from_worker: nav?.fromServiceWorker?.() ?? null, at: page.url().replace(/^https?:\/\/[a-z0-9]{20,}\./, '<label>.') };
  const landed = await Promise.race([
    page.waitForSelector('.mainDetailButtons .btnPlay', { state: 'visible', timeout: 60_000 }).then(() => 'item'),
    page.waitForSelector('#txtManualName, .manualLoginForm, .btnManual', { state: 'visible', timeout: 60_000 }).then(() => 'asked to sign in'),
    r.status && r.status >= 400 ? wait(3000).then(() => `answered ${r.status}`) : new Promise(() => {}),
  ]).catch(() => null);
  r.landed = landed ?? 'nothing within 60 s';
  r.landed_ms = landed ? Date.now() - t0 : null;
  r.shown = await shown(page);
  if (landed === 'item') {
    Object.assign(r, await page.evaluate(() => {
      const s = JSON.parse(localStorage.getItem('jellyfin_credentials') ?? '{}').Servers?.[0] ?? {};
      return { title: document.querySelector('.itemName, h1, h3.itemName')?.textContent ?? null,
        token: s.AccessToken == null ? null : s.AccessToken === 'svrngs-stand-in' ? 'the stand-in' : 'not the stand-in', user: s.UserId ?? null };
    }));
    r.signed_in = r.token === 'the stand-in';
  }
  person.steps.push({ choose: r });
  return r;
}

// One press of jellyfin-web's Play: how long until its <video> moves, a
// second tap given and counted if it is held at the start (person.mjs's).
async function play(person) {
  const page = person.page;
  const r = {};
  r.play_ms = await timed(person, 'playing', async () => {
    await page.locator('.mainDetailButtons .btnPlay:visible').first().click({ force: true });
    const t0 = Date.now();
    let heldSince = null;
    while (Date.now() - t0 < 30_000) {
      const v = await page.evaluate(() => { const v = document.querySelector('video'); return v && { ct: v.currentTime, paused: v.paused, rs: v.readyState }; });
      if (v && v.ct > 0.5 && !v.paused) return;
      const held = v && v.paused && v.rs >= 3 && v.ct < 0.5;
      heldSince = held ? heldSince ?? Date.now() : null;
      if (held && !r.second_tap && Date.now() - heldSince >= 3000) {
        r.second_tap = Date.now() - t0;
        await page.tap('video').catch(() => page.mouse.click(200, 300));
        await page.locator('.videoOsdBottom .btnPause').click({ timeout: 5000 }).catch(() => {});
      }
      await page.waitForTimeout(250);
    }
    throw new Error('not playing 30 s after Play');
  });
  if (r.play_ms !== null) r.video = await page.evaluate(() => { const v = document.querySelector('video'); return { ct: +v.currentTime.toFixed(1), w: v.videoWidth, h: v.videoHeight }; });
  person.steps.push({ play: r });
  return r;
}

// The browser's Back, pressed until Movie Night shows again (at most 6):
// the presses, and whether its query and cards are those of `before`.
async function back(person, before, origin) {
  const page = person.page;
  const r = { presses: 0, back: false };
  for (let i = 0; i < 6 && !r.back; i++) {
    r.presses++;
    await page.goBack({ waitUntil: 'commit', timeout: 30_000 }).catch((e) => (r.error = String(e).split('\n')[0]));
    await page.waitForTimeout(1500);
    if (page.url().startsWith(origin)) {
      await page.waitForSelector('#q', { timeout: 30_000 }).catch(() => {});
      await page.waitForTimeout(500);
      r.back = true;
    }
  }
  if (r.back) {
    const s = await showing(page);
    r.query = s.query;
    r.same_query = s.query === before.query;
    r.same_cards = JSON.stringify(s.cards) === JSON.stringify(before.cards);
    r.cards = s.cards;
  }
  r.at = page.url().replace(/^https?:\/\/[a-z0-9]{20,}\./, '<label>.');
  person.steps.push({ back: r });
  return r;
}

const origin = (page) => new URL(page.url()).origin + '/';
const film = (r, title) => r?.cards.find((c) => c.title.startsWith(title));

// ── the journeys ────────────────────────────────────────────────────────

async function phone(p) {
  const person = await browser(p);
  out.people[person.user] = person;
  await join(person);
  if (person.join_ms === null) return;
  person.open_ms = await openFromList(person, 'movie-night', '#q');
  if (person.open_ms === null) return;
  const mn = origin(person.page);
  person.cold = await search(person, 'bladruner');
  person.settled = await settle(person, 'bladruner');
  if (p.kept_from) person.kept = await keepFrom(person);
  person.kept_film = await search(person, 'brazl');
  person.warm = await search(person, 'bladruner');
  // Before this phone ever opened that Jellyfin from the list.
  person.never_opened = await choose(person, film(person.warm, 'Blade Runner'), p.choose);
  person.never_opened_back = await back(person, person.warm, mn);
  // That Jellyfin from the group's list, then Movie Night again.
  if ((await group(person)) === null) return;
  person.opened_jellyfin_ms = await openFromList(person, `${p.choose}’s jellyfin`, '.card, #txtManualName, .btnManual');
  person.opened_jellyfin_asked = await person.page.locator('#txtManualName:visible, .btnManual:visible').count() > 0;
  if ((await group(person)) === null) return;
  person.reopen_ms = await openFromList(person, 'movie-night', '#q');
  if (person.reopen_ms === null) return;
  person.reopened_showing = await showing(person.page);
  person.again = await search(person, 'bladruner');
  person.opened = await choose(person, film(person.again, 'Blade Runner'), p.choose);
  if (person.opened.landed === 'item') person.played = await play(person);
  person.opened_back = await back(person, person.again, mn);
}

async function asleep(p) {
  const person = await browser(p);
  out.people[person.user] = person;
  await join(person);
  if (person.join_ms === null) return;
  // The list must name Cy's jellyfin: Cy's computer answered Sam's. A
  // computer woken just after it slept was not answered for minutes
  // (22:06 here, 2026-10-08): how long it took is kept.
  const t0 = Date.now();
  for (let i = 0; i < 18 && !person.listed.lent?.some((l) => l.startsWith('Cy')); i++) {
    await wait(10_000);
    await group(person);
  }
  person.cy_listed = !!person.listed.lent?.some((l) => l.startsWith('Cy'));
  person.cy_listed_after_ms = Date.now() - t0;
  person.slept_at = now();
  execSync(SLEEP);
  person.open_ms = await openFromList(person, 'movie-night', '#q');
  if (person.open_ms === null) return;
  person.first = await search(person, 'bladruner');
  person.settled = await settle(person, 'bladruner', 4, 15_000, awayOrAnswered);
}

async function computer(p) {
  const person = await browser(p);
  out.people[person.user] = person;
  const page = person.page;
  person.open_ms = await timed(person, 'Movie Night', async () => { await page.goto(args.at, { waitUntil: 'load' }); await page.waitForSelector('#q'); });
  if (person.open_ms === null) return;
  person.nothing_handed = await search(person, 'bladruner', 30_000);
  person.store = await page.evaluate(() => indexedDB.databases?.().then((d) => d.map((x) => `${x.name} v${x.version}`)) ?? 'not listed').catch((e) => String(e));
  await page.goto(`${args.at}#lends=${args.lends}`);
  person.handed = await search(person, 'bladruner');
  person.opened = await choose(person, film(person.handed, 'Blade Runner'), p.choose);
  if (person.opened.landed === 'item') person.played = await play(person);
  person.opened_back = await back(person, person.handed, args.at);
}

async function limit() {
  const [a, b] = await Promise.all(args.people.map(browser));
  for (const p of [a, b]) out.people[p.user] = p;
  for (const p of [a, b]) {
    await join(p);
    p.open_ms = p.join_ms === null ? null : await openFromList(p, 'movie-night', '#q');
  }
  if (a.open_ms === null || b.open_ms === null) return;
  // Both known to every computer before the count is read.
  b.settled = await settle(b, 'bladruner');
  await wait(35_000);
  a.settled = await settle(a, 'bladruner');
  const ended = Date.parse(a.settled?.ended ?? now());
  b.after = [];
  for (const s of args.after_s) {
    await wait(Math.max(0, ended + s * 1000 - Date.now()));
    b.after.push({ after_s: s, ...(await search(b, 'bladruner')) });
  }
}

async function once(p) {
  const person = await browser(p);
  out.people[person.user] = person;
  await join(person);
  if (person.join_ms === null) return;
  person.open_ms = await openFromList(person, 'movie-night', '#q');
  if (person.open_ms === null) return;
  person.settled = await settle(person, 'bladruner');
}

try {
  if (args.journey === 'limit') await limit();
  else for (const p of args.people) await { phone, asleep, computer, once }[args.journey](p);
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  for (const p of Object.values(out.people)) {
    const { page, close, tally, ...rest } = p;
    out.people[p.user] = { ...rest, received: { ...tally }, logs: p.logs.slice(-40) };
    await close();
  }
  out.ended = now();
  console.log(JSON.stringify(out));
}
