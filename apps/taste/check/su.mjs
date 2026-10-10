// Scoped use's journey (check.sh; su-0's bars, demo/runs/su/2026-10-09/
// README.md §3, rerun on what svrngs carries since sup-1 to sup-3,
// demo/runs/sup/2026-10-10/README.md). Each person is a browser of their
// own, every response it receives written to DUMP/<user>/ for scan.py
// (apps/jellyfin/check/person.mjs `record`):
// - on a computer (`home`, the computer's home page,
//   http://svrngs.localhost:<port>/): Playwright's Chromium or WebKit as a
//   desktop, opening Tonight's Picks from the home page's list, and saying
//   yes where Tonight's Picks sends them, the home page's `/ask`;
// - on a phone (no `home`): Chromium, or WebKit as an iPhone
//   (web/tests/page.mjs), joining from the group's link, opening Tonight's
//   Picks from the group's list, and saying yes on the group's page at
//   `#ask=`.
// Ana is her computer, or her two phones; the other person is Alex's
// computer, or Bo's phone.
//
// `steps`, in order, each run when named:
//   join      each phone person joins from the group's link; after each of
//             Ana's phones, OWN adds its key to her taste's owners
//   yes       Ana's first opens Tonight's Picks, again until every library
//             and the taste answered past svrngs's doors (a phone is new to a
//             computer until its ring round); is invited, sent to svrngs's
//             question, reads it and says yes: back at the ranked list, timed
//             (bar 1)
//   again     Ana's first opens Tonight's Picks again: whether it asks; her
//             other phone opens it, is asked once and says yes, then opens it
//             again (bar 1)
//   other-person  Alex or Bo opens Tonight's Picks, says yes where it sends
//             them, and reads what the taste answers (bars 2, 6)
//   diary     Ana's first's Tonight's Picks asks the taste's /diary (bar 3)
//   other     Ana's first opens the other app: its ask, then a yes it says
//             itself and its ask again (bar 4)
//   stop      Ana's last presses "Stop using my taste" in Tonight's Picks,
//             then opens it again; AFTER_STOP, if set, runs between (the
//             plant); and her first phone opens it (bar 5)
//   home-stop on a computer: Ana says yes again, then Stop on her home
//             page's "What apps use", then opens Tonight's Picks (bar 5)
//   asleep    Ana's first lists the group, SLEEP puts Ana's computer to
//             sleep, then Tonight's Picks is opened (bar 6)
//
//   node su.mjs '{"tag", "link", "steps": [...], "people": [{"name", "user", "engine", "home"?}]}'
//   env: DUMP; OWN (adds the last key admitted to Ana's owners); SLEEP; AFTER_STOP
//
// Prints one JSON line. A step that did not happen within its time is null
// with what the page showed: never a number guessed.
import { execSync } from 'node:child_process';
import { launch } from '../../../web/tests/page.mjs';
import { record, shown, timed } from '../../jellyfin/check/person.mjs';

const args = JSON.parse(process.argv[2]);
const { DUMP, OWN, SLEEP, AFTER_STOP } = process.env;
const T0 = Date.now();
const out = { tag: args.tag, steps: args.steps, started: new Date(T0).toISOString(), people: {} };
const now = () => new Date().toISOString();
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const PICKS = 'tonights-picks';
// What a settled Tonight's Picks may say of the taste, past svrngs's doors.
const PAST = ['ranked', 'refused-unasked', 'refused-stopped', 'refused-not-yours', 'refused-unread', 'away'];

async function browser(p) {
  const person = { name: p.name, user: p.user, engine: p.engine, home: p.home ?? null, steps: [], logs: [] };
  const { context, close } = await launch(p.engine, p.home && p.engine === 'webkit' ? { device: 'Desktop Safari' } : {});
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

// ── the group's page, on a phone (mn.mjs's) ─────────────────────────────

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
}

// ── where a person opens an app: the home page, or the group's page ─────

// The home page, its list read: what it showed is kept the first time.
async function home(person) {
  return timed(person, 'the home page', async () => {
    await person.page.goto(person.home);
    await person.page.waitForFunction(() => !document.querySelector('#home .said'), null, { timeout: 60_000 });
    person.home_seen ??= await person.page.evaluate(() => document.getElementById('home').innerText);
  });
}

async function list(person) {
  if (person.home) return home(person);
  return timed(person, 'the group again', async () => {
    await person.page.goto(args.link);
    await person.page.waitForSelector('#status.good', { timeout: 120_000 });
  });
}

async function openFrom(person, what, ready) {
  return timed(person, `${what} from the list`, async () => {
    const link = person.home
      ? person.page.locator('#home a', { hasText: `'s ${what}` })
      : person.page.locator('#lent button', { hasText: what });
    await link.first().click();
    await person.page.waitForURL((u) => !/^svrngs\./.test(u.hostname) && !u.href.startsWith(args.link.split('#')[0]), { timeout: 90_000 });
    await person.page.waitForSelector(ready, { state: 'attached', timeout: 90_000 });
  });
}

// ── Tonight's Picks ─────────────────────────────────────────────────────

// What Tonight's Picks shows: each library's words, the taste's, whether
// it invites, the films in their order and whether they are ranked.
const showing = (page) => page.evaluate(() => ({
  at: location.href,
  said: [...document.querySelectorAll('#said li')].map((l) => ({ state: l.dataset.state, text: l.textContent })),
  taste: { state: document.getElementById('taste').dataset.state ?? null, text: document.getElementById('taste').textContent },
  asks: !document.getElementById('ask').hidden,
  invitation: document.getElementById('ask').hidden ? null : document.getElementById('ask').innerText,
  stop: !document.getElementById('stop').hidden,
  order: document.getElementById('picks').dataset.order ?? null,
  films: [...document.querySelectorAll('#picks li')].map((l) => l.dataset.title),
}));

// Settled: every library past `asking`, and the taste said, not asking.
const settled = () => {
  const said = [...document.querySelectorAll('#said li')];
  const t = document.getElementById('taste')?.dataset.state;
  return said.length > 0 && said.every((l) => l.dataset.state !== 'asking') && !!t && t !== 'asking';
};

async function picks(person, t0, limit = 120_000) {
  const ok = await person.page.waitForFunction(settled, null, { timeout: limit, polling: 100 }).then(() => true, () => false);
  const s = await showing(person.page).catch((e) => ({ error: String(e) }));
  return { at: now(), settled_ms: ok ? Date.now() - t0 : null, ...s, shown: ok ? undefined : await shown(person.page) };
}

// Past svrngs: one library answered at least, none refused, and the taste
// said in a way a door that knows this person gives.
const past = (r) => r.said?.some((l) => l.state === 'answered')
  && r.said.every((l) => ['answered', 'away', 'unanswered'].includes(l.state)) && PAST.includes(r.taste?.state);

// Open Tonight's Picks from the list, then again every 15 s until `past`,
// at most `tries`: each kept.
async function openPicks(person, label, tries = 12) {
  const r = { label, tries: [] };
  for (let i = 0; i < tries; i++) {
    if ((await list(person)) === null) break;
    const t0 = Date.now();
    if ((await openFrom(person, PICKS, '#taste')) === null) break;
    const got = await picks(person, t0);
    r.tries.push(got);
    if (past(got)) return { ...r, ...got, opened_ms: got.settled_ms };
    await wait(15_000);
  }
  return { ...r, past: false };
}

// The person invited by Tonight's Picks: sent to svrngs's question, which
// is read, and answered yes; then back at Tonight's Picks, its taste said.
async function sayYes(person) {
  const page = person.page;
  const r = { invited: await showing(page).catch(() => null) };
  if (!r.invited?.asks) return { ...r, asked: false };
  const t0 = Date.now();
  await page.click('#yes');
  const reached = await timed(person, 'svrngs\'s question', async () => {
    if (person.home) {
      await page.waitForURL((u) => /^svrngs\./.test(u.hostname) && u.pathname === '/ask', { timeout: 60_000 });
      await page.waitForSelector('#home section h2', { timeout: 60_000 });
    } else {
      await page.waitForURL((u) => u.href.startsWith(args.link.split('#')[0]) && u.hash.startsWith('#ask='), { timeout: 60_000 });
      await page.waitForSelector('#ask:not([hidden])', { timeout: 120_000 });
    }
  });
  if (reached === null) return { ...r, asked: false, question: null };
  r.question_at = page.url();
  r.question = await page.evaluate((computer) => computer
    ? document.querySelector('#home section').innerText
    : `${document.getElementById('status').textContent}\n${document.getElementById('words').textContent}`, !!person.home);
  r.invite_to_question_ms = Date.now() - t0;
  const t1 = Date.now();
  await (person.home ? page.getByRole('button', { name: 'Yes', exact: true }) : page.locator('#ask button:not(#not)')).click();
  const back = await timed(person, 'back at Tonight\'s Picks', async () => {
    await page.waitForURL((u) => !/^svrngs\./.test(u.hostname) && !u.href.startsWith(args.link.split('#')[0]), { timeout: 90_000 });
    await page.waitForSelector('#taste', { state: 'attached', timeout: 90_000 });
  });
  const after = back === null ? null : await picks(person, t1);
  return { ...r, asked: true, yes_to_list_ms: after?.settled_ms ?? null, after };
}

async function yes(ana) {
  const opened = await openPicks(ana, 'first open');
  ana.yes = { opened, ...(await sayYes(ana)) };
}

async function again(first, last) {
  first.again = await openPicks(first, 'opened again');
  if (last === first) return;
  const opened = await openPicks(last, 'the other phone');
  last.again = { opened, ...(await sayYes(last)) };
  last.again.reopened = await openPicks(last, 'the other phone again');
}

async function otherPerson(p) {
  const opened = await openPicks(p, p.name);
  p.picks = { opened, ...(await sayYes(p)) };
}

// The taste's /diary, asked from Tonight's Picks' page as it asks /rank.
async function diary(ana) {
  if (!(await ana.page.$('#taste'))) await openPicks(ana, 'for the diary');
  ana.diary = await ana.page.evaluate(async () => {
    const told = await (await fetch('/svrngs-reads')).json();
    const t = told.lends.find((l) => l.kind === 'taste');
    if (!t) return { status: null, body: 'no taste is read here' };
    try {
      const res = await fetch(`${t.address}diary`);
      return { status: res.status, refused: res.headers.get('x-svrngs-refused'), body: (await res.text()).slice(0, 400) };
    } catch (e) {
      return { status: null, body: `unread: ${e}` };
    }
  }).catch((e) => ({ error: String(e) }));
  ana.diary.at = now();
}

async function other(ana) {
  if ((await list(ana)) === null) return;
  const t0 = Date.now();
  const opened = await openFrom(ana, 'other', '#out');
  const asked = await ana.page.waitForFunction(() => document.getElementById('out').dataset.state, null, { timeout: 120_000 }).then(() => true, () => false);
  const read = () => ana.page.evaluate(() => {
    const o = document.getElementById('out');
    return { state: o.dataset.state ?? null, refused: o.dataset.refused || null, yes: o.dataset.yes ? JSON.parse(o.dataset.yes) : null, text: o.textContent };
  });
  ana.other = { opened_ms: opened, asked_ms: asked ? Date.now() - t0 : null, at: now(), ask: await read() };
  await ana.page.click('#unasked');
  await ana.page.waitForFunction(() => document.getElementById('out').dataset.state?.startsWith('unasked'), null, { timeout: 120_000 }).catch(() => {});
  ana.other.unasked = await read();
  ana.other.unasked_at = now();
}

// What the home page lists of what apps use.
const uses = (person) => home(person).then(() => person.page.waitForFunction(
  () => [...document.querySelectorAll('#home section')].some((s) => s.querySelector('h2')?.textContent === 'What apps use' && s.querySelectorAll('p').length),
  null, { timeout: 30_000 }).catch(() => {})).then(() => person.page.evaluate(
  () => [...document.querySelectorAll('#home section')].find((s) => s.querySelector('h2')?.textContent === 'What apps use')?.innerText ?? null));

async function stop(last, first) {
  if (!(await last.page.$('#stop'))) await openPicks(last, 'to stop');
  const before = await showing(last.page);
  if (!before.stop) return (last.stopped = { pressed: false, before });
  const t0 = Date.now();
  await last.page.click('#stop');
  await last.page.waitForFunction(() => document.getElementById('taste').dataset.state !== 'ranked', null, { timeout: 60_000 }).catch(() => {});
  last.stopped = { pressed: true, at: new Date(t0).toISOString(), said: await showing(last.page), said_ms: Date.now() - t0 };
  if (AFTER_STOP) last.stopped.planted = execSync(AFTER_STOP).toString().trim();
  const t1 = Date.now();
  await last.page.reload({ timeout: 60_000 }).catch((e) => last.logs.push(`reload: ${e}`));
  last.stopped.again = await picks(last, t1);
  last.stopped.stop_to_refused_ms = last.stopped.again.taste?.state?.startsWith('refused') ? Date.now() - t0 : null;
  if (last.home) last.stopped.uses = await uses(last);
  if (first !== last) first.after_stop = await openPicks(first, 'the other phone, after the stop');
}

// On a computer: yes again, then Stop on the home page, then Tonight's Picks.
async function homeStop(ana) {
  const opened = await openPicks(ana, 'to say yes again');
  ana.home_stop = { opened, yes: await sayYes(ana) };
  ana.home_stop.uses_before = await uses(ana);
  const button = ana.page.locator('#home section p', { hasText: 'tonights-picks uses' }).getByRole('button', { name: 'Stop' });
  if (!(await button.count())) return (ana.home_stop.pressed = false);
  const t0 = Date.now();
  ana.home_stop.at = new Date(t0).toISOString();
  const kept = ana.page.waitForResponse((r) => r.url().endsWith('/choices') && r.request().method() === 'POST', { timeout: 60_000 });
  await button.first().click();
  ana.home_stop.answer = await kept.then((r) => r.status(), (e) => String(e));
  ana.home_stop.uses_after = await uses(ana);
  await openFrom(ana, PICKS, '#taste');
  ana.home_stop.after = await picks(ana, Date.now());
  ana.home_stop.stop_to_refused_ms = ana.home_stop.after.taste?.state?.startsWith('refused') ? Date.now() - t0 : null;
}

async function asleep(ana) {
  if ((await list(ana)) === null) return;
  ana.asleep = { slept_at: now() };
  execSync(SLEEP);
  const t0 = Date.now();
  if ((await openFrom(ana, PICKS, '#taste')) === null) return;
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
  const others = people.filter((p) => p.name !== 'Ana');
  const [first, last] = [anas[0], anas.at(-1)];
  for (const step of args.steps) {
    first.steps.push({ step, at: now() });
    if (step === 'join') {
      for (const p of people.filter((q) => !q.home)) {
        await join(p);
        if (p.name === 'Ana') p.owner = execSync(OWN).toString().trim();
      }
    } else if (step === 'yes') await yes(first);
    else if (step === 'again') await again(first, last);
    else if (step === 'other-person') for (const p of others) await otherPerson(p);
    else if (step === 'diary') await diary(first);
    else if (step === 'other') await other(first);
    else if (step === 'stop') await stop(last, first);
    else if (step === 'home-stop') await homeStop(first);
    else if (step === 'asleep') await asleep(first);
    else throw new Error(`no step ${step}`);
  }
} catch (e) {
  out.stopped = String(e).split('\n')[0];
} finally {
  for (const p of Object.values(out.people)) {
    const { page, close, tally, ...rest } = p;
    out.people[p.user] = { ...rest, received: { ...tally }, logs: p.logs.slice(-60) };
    await close();
  }
  out.ended = now();
  console.log(JSON.stringify(out));
}
