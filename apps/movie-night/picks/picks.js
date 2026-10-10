// mn: own picks
// su: own picks — Tonight's Picks (demo/runs/su/2026-10-09/README.md, su-0;
// cut to what svrngs carries, sup-4, demo/runs/sup/2026-10-09/): the
// group's films, as Movie Night lists them, ranked by the person's taste
// once they say yes. Lent apart from Movie Night's search, `--reads
// jellyfin --reads taste`, so the yes it is given is its own. A static
// page: it keeps nothing, and never says the yes itself — it sends its
// person to the page no app runs at, the home page on a computer and the
// group's page on a phone, and the taste's door keeps what they say
// (DESIGN §3, "What an app gets"). Every line is marked by su-0's rule (§4).
import { copyOf, films } from './match.js';
import { PLANT } from './plant.js';

const said = document.getElementById('said');
const ask = document.getElementById('ask');
const taste = document.getElementById('taste');
const stop = document.getElementById('stop');
const picks = document.getElementById('picks');
const LIST = 'Items?Recursive=true&IncludeItemTypes=Movie&Fields=ProviderIds,ProductionYear&EnableImages=false';

// su: plumbing find — what svrngs tells this page at its own origin: each
// lend it reads, its kind, name, lender and address, a lend that asks its
// words and its `ask` address, and whose computer did not answer (DESIGN
// §3, "Lent apps", call 6). A stop is sent to `/svrngs-reads/<label>/
// svrngs-stop` here, the label the one `ask` names.
const told = () => fetch('/svrngs-reads').then((r) => (r.ok ? r.json() : null), () => null);
const stopAt = (t) => `/svrngs-reads/${new URL(t.ask).searchParams.get('lend')}/svrngs-stop`;

// su: plumbing refusal — svrngs's answers in a lend's place told from the
// lend's by `x-svrngs-refused` (DESIGN §3), the taste's by its `refused`.
// On a computer the taste is at another origin, and svrngs's own answer
// there lets no other origin read it: a fetch that fails is unread, never
// a kind guessed.
async function asked(url, body) {
  let res;
  try {
    res = await fetch(url, body === undefined ? {} : { method: 'POST', body: JSON.stringify(body) });
  } catch (e) {
    return { state: 'unread', why: String(e) };
  }
  const json = await res.json().catch(() => null);
  const kind = res.headers.get('x-svrngs-refused');
  if (kind === 'away') return { state: 'away' };
  if (kind) return { state: 'refused', kind, why: json?.error };
  if (res.ok && json) return { state: 'answered', body: json };
  return { state: 'refused', kind: json?.refused ?? 'lend', why: json?.error ?? `answered ${res.status}` };
}

// su: own picks — the group's films: every library asked at once, each
// said as it answers, one film a line.
const el = (tag, text, data = {}) => {
  const e = Object.assign(document.createElement(tag), { textContent: text });
  Object.assign(e.dataset, data);
  return e;
};

function libraryWords(l) {
  if (l.state === 'answered') return `${l.who}: ${l.n} ${l.n === 1 ? 'film' : 'films'}`;
  if (l.state === 'away') return `${l.who} did not answer: its computer may be asleep. Its films are not here.`;
  if (l.state === 'unanswered') return `${l.who} did not answer, so nothing of theirs is here.`;
  if (l.state === 'refused') return `${l.who} would not let you in.`;
  return `${l.who}: ${l.state === 'asking' ? 'asking…' : 'could not be read from here.'}`;
}

async function group(libs, unanswered) {
  const copies = [];
  const state = libs.map((l) => ({ ...l, state: 'asking' }));
  const away = unanswered.map((p) => ({ who: `${p}’s computer`, state: 'unanswered' }));
  const draw = () => said.replaceChildren(...[...state, ...away].map((l) => el('li', cap(libraryWords(l)), { state: l.state })));
  draw();
  await Promise.all(
    libs.map(async (lib, i) => {
      const got = await asked(`${lib.address}${LIST}`);
      const mine = got.state === 'answered' ? (got.body.Items ?? []).map((item) => copyOf(item, lib.who)) : [];
      copies.push(...mine);
      state[i] = { ...state[i], state: got.state, n: mine.length };
      draw();
    }),
  );
  return films(copies).map((f, n) => ({ n, title: f.title, year: f.year, ids: f.copies[0].ids }));
}

function list(order, how) {
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
  unasked: () => 'Not ranked: you have not let Tonight’s Picks use your taste.',
  stopped: (t) => `You stopped Tonight’s Picks using ${t.who}. These films are not ranked.`,
  unread: (t) => `Tonight’s Picks could not read whether ${t.who} lets it rank. These films are not ranked.`,
  lend: (t) => `${t.who} would not let you in. These films are not ranked.`,
};

function refused(t, got, films) {
  // su: plant empty — bar 6's plant: a refusal shown as an empty ranking
  if (PLANT === 'empty') return list([], 'ranked'), tell('ranked', 'No picks for you tonight.');
  // su: own consent
  list(films, 'unranked');
  if (got.state === 'away') return tell('away', `${t.who} did not answer: its computer may be asleep. These films are not ranked.`);
  const kind = got.state === 'unread' ? 'unread' : got.kind;
  tell(`refused-${kind}`, (REFUSED[kind] ?? REFUSED.lend)(t));
  // The person may say yes where svrngs asks them, when the taste asks.
  ask.hidden = !(t.ask && ['unasked', 'stopped', 'unread'].includes(kind));
}

// su: own picks — rank `films` by the first taste lend.
async function rank(t, films) {
  tell('asking', `Asking ${t.who}…`);
  const got = await asked(`${t.address}rank`, { films });
  if (got.state !== 'answered') return refused(t, got, films);
  list(got.body.films, 'ranked');
  tell('ranked', 'Ranked by your taste.');
  stop.hidden = !t.ask;
}

async function start() {
  const reads = await told();
  if (!reads) return said.replaceChildren(el('li', 'Tonight’s Picks could not learn what is open to it. Open it again from your group’s page.', { state: 'untold' }));
  const lends = reads.lends.map((l) => ({ ...l, who: `${l.lender}’s ${l.name}` }));
  const [libs, tastes] = [lends.filter((l) => l.kind !== 'taste'), lends.filter((l) => l.kind === 'taste')];
  const films = await group(libs, reads.unanswered);
  list(films, 'unranked');
  if (!tastes.length) return tell('none', 'No taste is open to Tonight’s Picks here. These films are not ranked.');
  const t = tastes[0];
  // su: own consent — the person's presses: a yes said where svrngs asks
  // them, then back here; a stop sent as this app.
  document.getElementById('yes').onclick = () => location.assign(`${t.ask}&back=${encodeURIComponent(location.pathname)}`);
  document.getElementById('no').onclick = () => {
    ask.hidden = true;
    tell('refused-unasked', 'Not ranked: you kept browsing.');
  };
  stop.onclick = async () => {
    stop.hidden = true;
    const got = await asked(stopAt(t), {});
    if (got.state !== 'answered') return refused(t, got, films);
    picks.dataset.order = 'ranked before the stop';
    tell('stopped', `Stopped. From now on ${t.who} refuses Tonight’s Picks. The list ranked before you stopped stays on this screen until you leave it.`);
  };
  // su: own picks
  await rank(t, films);
}

start();
