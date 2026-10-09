// mn: own picks
// su: own picks — Tonight's Picks (demo/runs/su/2026-10-09/README.md, su-0):
// the group's films, as Movie Night lists them, ranked by the person's
// taste once they say yes. Lent apart from Movie Night's search, `--reads
// jellyfin --reads taste`, so a pass this page holds names it first. A
// static page: it asks the taste lend as the person and keeps nothing; the
// yes and the stop are kept where the taste is. Every line is marked by
// su-0's rule (§4).
import { copyOf, films } from './match.js';
import { PLANT } from './plant.js';

const said = document.getElementById('said');
const ask = document.getElementById('ask');
const taste = document.getElementById('taste');
const stop = document.getElementById('stop');
const picks = document.getElementById('picks');
const LIST = 'Items?Recursive=true&IncludeItemTypes=Movie&Fields=ProviderIds,ProductionYear&EnableImages=false';

// su: plumbing find — what svrngs handed this origin: the lends its pass
// opens, each {label, key, name}, in IndexedDB `svrngs-lend` as `pass <this
// origin's label>` (web/open.js), as Movie Night reads them. No kind comes
// with a lend, so a lend named `taste` is a taste and any other a library.
// Each lend is at `<label>.<this host>` (web/lend.js `baseOf`).
const OWN = location.hostname.split('.')[0];
const baseOf = (label) => `${location.protocol}//${label}${location.host.slice(OWN.length)}/`;

function kept() {
  return new Promise((done) => {
    const req = indexedDB.open('svrngs-lend');
    req.onupgradeneeded = () => req.transaction.abort();
    req.onerror = () => done(null);
    req.onsuccess = () => {
      const db = req.result;
      try {
        const get = db.transaction('kept').objectStore('kept').get(`pass ${OWN}`);
        get.onsuccess = () => (db.close(), done(get.result ?? null));
        get.onerror = () => (db.close(), done(null));
      } catch {
        db.close();
        done(null);
      }
    };
  });
}

async function lends() {
  const all = ((await kept())?.lends ?? [])
    .filter((l) => l.label !== OWN)
    .map((l) => ({ label: l.label, name: l.name, who: `the ${l.name} at ${l.key.slice(0, 8)}`, base: baseOf(l.label) }));
  return { libs: all.filter((l) => l.name !== 'taste'), tastes: all.filter((l) => l.name === 'taste') };
}

// su: plumbing refusal — a lend asked as the person, and svrngs's answers
// told from the lend's: the lend's worker answers 502 when the lender's
// computer did not answer (web/lend.js), the door 403 with its own words;
// the taste refuses with `refused`, its kind. A fetch that fails outright
// was not let read here.
async function asked(url, body) {
  let res;
  try {
    res = await fetch(url, body === undefined ? { headers: { accept: 'application/json' } } : { method: 'POST', body: JSON.stringify(body) });
  } catch (e) {
    return { state: 'failed', why: String(e) };
  }
  if (res.status === 502 || res.status === 504) return { state: 'away', why: `answered ${res.status}` };
  const json = await res.json().catch(() => null);
  if (res.ok && json) return { state: 'answered', body: json };
  if (json?.refused) return { state: 'refused', kind: json.refused, why: json.error };
  return { state: 'refused', kind: 'svrngs', why: json?.error ?? `answered ${res.status}` };
}

// su: own picks — the group's films: every library asked at once, each
// said as it answers, one film a line.
let shown = [];
const el = (tag, text, data = {}) => {
  const e = Object.assign(document.createElement(tag), { textContent: text });
  Object.assign(e.dataset, data);
  return e;
};

function libraryWords(l) {
  if (l.state === 'answered') return `${l.who}: ${l.n} ${l.n === 1 ? 'film' : 'films'}`;
  if (l.state === 'away') return `${l.who} did not answer: its computer may be asleep. Its films are not here.`;
  if (l.state === 'refused') return `${l.who} would not let you in.`;
  return `${l.who}: ${l.state === 'asking' ? 'asking…' : 'could not be read from here.'}`;
}

async function group(libs) {
  const copies = [];
  const state = libs.map((l) => ({ ...l, state: 'asking' }));
  const draw = () => said.replaceChildren(...state.map((l) => el('li', cap(libraryWords(l)), { state: l.state })));
  draw();
  await Promise.all(
    libs.map(async (lib, i) => {
      const got = await asked(`${lib.base}${LIST}`);
      const mine = got.state === 'answered' ? (got.body.Items ?? []).map((item) => copyOf(item, lib.label)) : [];
      copies.push(...mine);
      state[i] = { ...state[i], state: got.state, n: mine.length };
      draw();
    }),
  );
  return films(copies).map((f, n) => ({ n, title: f.title, year: f.year, ids: f.copies[0].ids }));
}

function list(order, how) {
  shown = order;
  picks.dataset.order = how;
  picks.replaceChildren(...order.map((f) => el('li', f.year ? `${f.title} (${f.year})` : f.title, { title: f.title })));
}

// su: own consent — what the person reads of the taste, each refusal said
// as what it is, the films beside it unranked.
const cap = (words) => words.charAt(0).toUpperCase() + words.slice(1);

function tell(state, words) {
  taste.dataset.state = state;
  taste.textContent = cap(words);
}

const REFUSED = {
  'not-yours': (t) => `${t.who} is someone else’s. These films are not ranked.`,
  'no-app': (t) => `${t.who} was not told which app asked, so it ranked nothing. These films are not ranked.`,
  'not-asked': () => 'Not ranked: you kept browsing.',
  stopped: () => 'You stopped Tonight’s Picks using your taste. These films are not ranked.',
  svrngs: (t) => `${t.who} would not let you in. These films are not ranked.`,
};

function refused(t, got, films) {
  // su: plant empty — bar 6's plant: a refusal shown as an empty ranking
  if (PLANT === 'empty') return list([], 'ranked'), tell('ranked', 'No picks for you tonight.');
  // su: own consent
  list(films, 'unranked');
  if (got.state === 'away') return tell('away', `${t.who} did not answer: its computer may be asleep. These films are not ranked.`);
  if (got.state === 'failed') return tell('failed', `${t.who} could not be read from here. These films are not ranked.`);
  tell(`refused-${got.kind}`, (REFUSED[got.kind] ?? REFUSED.svrngs)(t));
}

// su: own picks — rank `films` by the first taste lend, asking the person
// first when it has no yes for this app, or a stop since.
async function rank(t, films) {
  tell('asking', `Asking ${t.who}…`);
  const got = await asked(`${t.base}rank`, { films });
  if (got.state === 'answered') {
    list(got.body.films, 'ranked');
    tell('ranked', 'Ranked by your taste.');
    stop.hidden = false;
    return;
  }
  if (got.state === 'refused' && (got.kind === 'not-asked' || got.kind === 'stopped') && !ask.dataset.answered) {
    list(films, 'unranked');
    // su: own consent
    if (got.kind === 'stopped') tell('refused-stopped', REFUSED.stopped());
    else tell('asked', '');
    ask.hidden = false;
    return;
  }
  // su: own picks
  refused(t, got, films);
}

async function start() {
  const { libs, tastes } = await lends();
  if (!libs.length) said.replaceChildren(el('li', 'No library is open to Tonight’s Picks here.'));
  const films = await group(libs);
  list(films, 'unranked');
  if (!tastes.length) return tell('none', 'No taste is open to Tonight’s Picks here. These films are not ranked.');
  const t = tastes[0];
  // su: own consent — the person's presses: a yes or a stop sent as them,
  // kept by the taste for the app the door names.
  document.getElementById('yes').onclick = async () => {
    ask.hidden = true;
    ask.dataset.answered = 'yes';
    const got = await asked(`${t.base}yes`, {});
    if (got.state !== 'answered') return refused(t, got, films);
    await rank(t, films);
  };
  document.getElementById('no').onclick = () => {
    ask.hidden = true;
    ask.dataset.answered = 'no';
    tell('refused-not-asked', REFUSED['not-asked']());
  };
  stop.onclick = async () => {
    stop.hidden = true;
    const got = await asked(`${t.base}stop`, {});
    if (got.state !== 'answered') return refused(t, got, films);
    picks.dataset.order = 'ranked before the stop';
    tell('stopped', `Stopped. From now on ${t.who} refuses Tonight’s Picks. The list ranked before you stopped stays on this screen until you leave it.`);
  };
  // su: own picks
  await rank(t, films);
}

start();
