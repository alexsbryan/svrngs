// Scoped use's journey (check.sh; the bars in demo/runs/su/2026-10-09/
// README.md §3). Each person is a browser of their own, Playwright's
// headless Chromium or its WebKit as an iPhone (web/tests/page.mjs), every
// response it receives written to DUMP/<user>/ for scan.py
// (apps/jellyfin/check/person.mjs `record`). Ana is two phones, one an
// engine each, both her taste's owners; Bo is another person of the group.
//
// `steps`, in order, each run when named:
//   join    each person new to the group opens its link and joins; after
//           each Ana, OWN adds her phone's key to her taste's owners
//   yes     Ana's first phone opens Tonight's Picks from the list, again
//           until every library and the taste answered past svrngs's doors
//           (a phone's person is new to a computer until its ring round);
//           reads the question, says yes, the ranked list timed (bar 1)
//   again   that phone opens Tonight's Picks again; then her second phone
//           opens it, until answered: whether either is asked (bar 1)
//   bo      Bo opens Tonight's Picks, until answered: what he reads (bars 2, 6)
//   diary   Ana's first phone's Tonight's Picks asks the taste's /diary (bar 3)
//   other   Ana's first phone opens the other app: its ask, then a yes it
//           sends unasked and its ask again (bar 4; the measured gap)
//   stop    Ana's last phone presses "Stop using my taste" in Tonight's
//           Picks, then opens it again, and her first phone too (bar 5)
//   asleep  Ana's first phone lists the group, SLEEP puts Ana's computer to
//           sleep, then Tonight's Picks is opened (bar 6)
//
//   node su.mjs '{"tag", "link", "steps": [...], "people": [{"name", "user", "engine"}]}'
//   env: DUMP; OWN (adds the last key admitted to Ana's owners); SLEEP
//
// Prints one JSON line. A step that did not happen within its time is null
// with what the page showed: never a number guessed.
import { execSync } from 'node:child_process';
import { launch } from '../../../web/tests/page.mjs';
import { record, shown, timed } from '../../jellyfin/check/person.mjs';

const args = JSON.parse(process.argv[2]);
const { DUMP, OWN, SLEEP } = process.env;
const T0 = Date.now();
const out = { tag: args.tag, steps: args.steps, started: new Date(T0).toISOString(), people: {} };
const now = () => new Date().toISOString();
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const PICKS = 'tonights-picks';

async function browser(p) {
  const person = { name: p.name, user: p.user, engine: p.engine, steps: [], logs: [] };
  const { context, close } = await launch(p.engine);
  person.close = close;
  person.page = await context.newPage();
  person.tally = record(context, person.page, p.user, { DUMP, T0 });
  const page = person.page;
  page.on('pageerror', (e) => person.logs.push(`pageerror: ${e}`));
  page.on('console', (m) => { if (m.type() === 'error' || m.type() === 'warning') person.logs.push(`console.${m.type()}: ${m.text().slice(0, 240)}`); });
  page.on('requestfailed', (r) => person.logs.push(`failed: ${r.url().slice(0, 120)}: ${r.failure()?.errorText}`));
  page.on('response', (r) => { if (r.status() >= 400) person.logs.push(`http ${r.status()} ${r.request().method()} ${r.url().slice(0, 140)}`); });
  person.browser = context.browser()?.version() ?? null;
  return person;
}

// ── the group's page (mn.mjs's) ─────────────────────────────────────────

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
  })).catch((e) => ({ error: String(e) }));
  return person.listed;
}

async function group(person) {
  return timed(person, 'the group again', async () => {
    await person.page.goto(args.link);
    await person.page.waitForSelector('#status.good', { timeout: 120_000 });
    await listed(person);
  });
}

async function openFromList(person, what, ready) {
  return timed(person, `${what} from the list`, async () => {
    await person.page.locator('#lent button', { hasText: what }).click();
    await person.page.waitForURL((u) => !u.href.startsWith(args.link.split('#')[0]), { timeout: 90_000 });
    await person.page.waitForSelector(ready, { state: 'attached', timeout: 90_000 });
  });
}

// ── Tonight's Picks ─────────────────────────────────────────────────────

// What Tonight's Picks shows: each library's words, the taste's, whether
// it asks, the films in their order and whether they are ranked.
const showing = (page) => page.evaluate(() => ({
  said: [...document.querySelectorAll('#said li')].map((l) => ({ state: l.dataset.state, text: l.textContent })),
  taste: { state: document.getElementById('taste').dataset.state ?? null, text: document.getElementById('taste').textContent },
  asks: !document.getElementById('ask').hidden,
  question: document.getElementById('ask').hidden ? null : document.getElementById('ask').innerText,
  stop: !document.getElementById('stop').hidden,
  order: document.getElementById('picks').dataset.order ?? null,
  films: [...document.querySelectorAll('#picks li')].map((l) => l.dataset.title),
}));

// Settled: every library past `asking`, and the taste said, or asking.
const settled = () => {
  const said = [...document.querySelectorAll('#said li')];
  const t = document.getElementById('taste').dataset.state;
  return said.length > 0 && said.every((l) => l.dataset.state !== 'asking') && !!t && t !== 'asking';
};

// Tonight's Picks from the time `t0`: settled, what it shows and when.
async function picks(person, t0, limit = 120_000) {
  const ok = await person.page.waitForFunction(settled, null, { timeout: limit, polling: 100 }).then(() => true, () => false);
  const s = await showing(person.page).catch((e) => ({ error: String(e) }));
  return { at: now(), settled_ms: ok ? Date.now() - t0 : null, ...s, shown: ok ? undefined : await shown(person.page) };
}

// Past svrngs: no library refused (a door that does not know this phone
// yet), one answered at least, Cy's asleep one said away; and the taste
// answered or refused in its own words.
const past = (r) => r.said?.some((l) => l.state === 'answered') && r.said.every((l) => ['answered', 'away'].includes(l.state))
  && r.taste?.state && r.taste.state !== 'refused-svrngs';

// Open Tonight's Picks from the list, then again every 15 s until `past`,
// at most `tries`: each kept.
async function openPicks(person, label, tries = 12) {
  const r = { label, tries: [] };
  for (let i = 0; i < tries; i++) {
    if ((await group(person)) === null) break;
    const t0 = Date.now();
    if ((await openFromList(person, PICKS, '#taste')) === null) break;
    const got = await picks(person, t0);
    r.tries.push(got);
    if (past(got)) return { ...r, ...got, opened_ms: got.settled_ms };
    await wait(15_000);
  }
  return { ...r, past: false };
}

// The same page again, by the browser's reload.
async function reload(person) {
  const t0 = Date.now();
  await person.page.reload({ timeout: 60_000 }).catch((e) => person.logs.push(`reload: ${e}`));
  return picks(person, t0);
}

async function yes(ana) {
  const r = await openPicks(ana, 'first open');
  ana.steps.push({ yes_opened: r });
  if (!r.asks) return (ana.yes = { asked: false, before: r });
  const t0 = Date.now();
  await ana.page.click('#yes');
  const ok = await ana.page.waitForFunction(() => ['ranked', 'refused-svrngs', 'refused-not-yours', 'refused-no-app', 'away', 'failed'].includes(document.getElementById('taste').dataset.state), null, { timeout: 60_000, polling: 50 }).then(() => true, () => false);
  const after = await showing(ana.page);
  ana.yes = { asked: true, question: r.question, opened: r, yes_to_list_ms: ok ? Date.now() - t0 : null, after };
}

// The taste lend's /diary, asked from Tonight's Picks' page as it asks /rank.
async function diary(ana) {
  if (!(await ana.page.$('#taste'))) await openPicks(ana, 'for the diary');
  ana.diary = await ana.page.evaluate(async () => {
    const OWN = location.hostname.split('.')[0];
    const pass = await new Promise((done) => {
      const req = indexedDB.open('svrngs-lend');
      req.onsuccess = () => { const g = req.result.transaction('kept').objectStore('kept').get(`pass ${OWN}`); g.onsuccess = () => done(g.result); g.onerror = () => done(null); };
      req.onerror = () => done(null);
    });
    const t = pass?.lends.find((l) => l.name === 'taste');
    if (!t) return { status: null, body: 'no taste in the pass' };
    const res = await fetch(`${location.protocol}//${t.label}${location.host.slice(OWN.length)}/diary`);
    return { status: res.status, body: (await res.text()).slice(0, 400) };
  }).catch((e) => ({ error: String(e) }));
}

async function other(ana) {
  if ((await group(ana)) === null) return;
  const t0 = Date.now();
  const opened = await openFromList(ana, 'other', '#out');
  const asked = await ana.page.waitForFunction(() => document.getElementById('out').dataset.state, null, { timeout: 120_000 }).then(() => true, () => false);
  const read = () => ana.page.evaluate(() => ({ state: document.getElementById('out').dataset.state ?? null, yes: document.getElementById('out').dataset.yes ?? null, text: document.getElementById('out').textContent }));
  ana.other = { opened_ms: opened, asked_ms: asked ? Date.now() - t0 : null, ask: await read() };
  await ana.page.click('#unasked');
  await ana.page.waitForFunction(() => document.getElementById('out').dataset.state?.startsWith('unasked'), null, { timeout: 60_000 }).catch(() => {});
  ana.other.unasked = await read();
}

async function stop(last, first) {
  const before = await showing(last.page);
  if (!before.stop) {
    last.stopped = { pressed: false, before };
    return;
  }
  const t0 = Date.now();
  await last.page.click('#stop');
  await last.page.waitForFunction(() => document.getElementById('taste').dataset.state !== 'ranked', null, { timeout: 60_000 }).catch(() => {});
  last.stopped = { pressed: true, at: now(), said: await showing(last.page), said_ms: Date.now() - t0 };
  last.stopped.again = await reload(last);
  last.stopped.stop_to_refused_ms = last.stopped.again.taste?.state === 'refused-stopped' ? Date.now() - t0 : null;
  if (first !== last) {
    await group(first);
    const t1 = Date.now();
    await openFromList(first, PICKS, '#taste');
    first.after_stop = await picks(first, t1);
  }
}

async function asleep(ana) {
  if ((await group(ana)) === null) return;
  ana.asleep = { listed: ana.listed, slept_at: now() };
  execSync(SLEEP);
  const t0 = Date.now();
  if ((await openFromList(ana, PICKS, '#taste')) === null) return;
  ana.asleep.picks = await picks(ana, t0, 180_000);
}

try {
  const people = [];
  for (const p of args.people) {
    const person = await browser(p);
    out.people[person.user] = person;
    people.push(person);
  }
  const anas = people.filter((p) => p.name === 'Ana');
  const bo = people.find((p) => p.name === 'Bo');
  const [first, last] = [anas[0], anas.at(-1)];
  for (const step of args.steps) {
    out.people[first.user].steps.push({ step, at: now() });
    if (step === 'join') {
      for (const p of people) {
        await join(p);
        if (p.name === 'Ana') p.owner = execSync(OWN).toString().trim();
      }
    } else if (step === 'yes') await yes(first);
    else if (step === 'again') {
      first.again = await openPicks(first, 'opened again');
      if (last !== first) last.again = await openPicks(last, 'the other phone');
    } else if (step === 'bo') bo.picks = await openPicks(bo, 'Bo');
    else if (step === 'diary') await diary(first);
    else if (step === 'other') await other(first);
    else if (step === 'stop') await stop(last, first);
    else if (step === 'asleep') await asleep(first);
    else throw new Error(`no step ${step}`);
  }
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
