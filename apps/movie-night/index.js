// mn: own cards — Movie Night: one search over every Jellyfin the group
// lends, as the person searching may see each, one card a film with its
// copies under it, each opening in its own Jellyfin
// (demo/runs/mn/2026-10-08/README.md). A static page: no server, no
// credential, nothing kept but its own query and results.
import { closest, copyOf, distance, films, itemAt } from './match.js';
// mn: plant switch
import { PLANT, TOKEN } from './plant.js';

// mn: own cards
const form = document.getElementById('search');
const box = document.getElementById('q');
const said = document.getElementById('said');
const results = document.getElementById('results');
const KEPT = 'movie-night';

// mn: plumbing find — which libraries there are, whose each is, where to
// ask each and whose computer did not answer: svrngs tells a page at its
// own origin (DESIGN §3, "Lent apps", call 6), or answers no.
const told = () =>
  fetch('/svrngs-reads').then(
    (r) => (r.ok ? r.json() : null),
    () => null,
  );

// mn: own cards — each library by its lender, in the person's words: "Sam’s
// library", and its name beside that only where Sam lends more than one.
async function libraries() {
  const t = await told();
  if (!t) return null;
  const many = (l) => t.lends.filter((o) => o.lender === l.lender).length > 1;
  return {
    libs: t.lends.map((l) => ({
      library: l.lender,
      who: many(l) ? `${l.lender}’s library “${l.name}”` : `${l.lender}’s library`,
      base: l.address,
      // Carried, a lender's own page opens by svrngs's `to`; lent, under
      // the address it reads.
      open: l.to ?? `${l.address}web/`,
    })),
    unanswered: t.unanswered,
    // Libraries this page may read once its person opens it from their
    // home page and leaves them ticked: no address yet.
    fresh: [...new Set((t.new ?? []).map((l) => l.lender))],
  };
}

// mn: plumbing ask — a library asked with no credential: the door lets
// a reading app in as the account its lender keeps for readers (`svrngs
// share --readers`, DESIGN §3 "Lent apps, as they come" 4), on a GET.
// What it must tell apart is svrngs's own answers from the library's: the
// lend's worker answers 502 when the lender's computer did not answer and
// 403 when the pass does not open the lend (web/lend.js:268, :300-309), as
// origin.rs answers 502 on a computer. A fetch that fails outright was not
// let read here, which is neither.
async function ask(url) {
  let res;
  try {
    res = await fetch(url, { headers: { accept: 'application/json', ...planted() } });
  } catch (e) {
    return { state: 'failed', why: String(e) };
  }
  if (res.status === 502 || res.status === 504)
    return { state: 'away', why: `answered ${res.status}` };
  // The library itself asked for a sign-in: its lender keeps no account for
  // apps that read it (`svrngs share --readers`), which only they can add.
  if (res.status === 401) return { state: 'closed', why: 'answered 401' };
  if (!res.ok) return { state: 'refused', why: `answered ${res.status}` };
  return { state: 'answered', body: await res.json() };
}

// mn: plant lender-token — bar 5's plant: a lender's token written here
// and sent on each request
const planted = () => (TOKEN ? { authorization: `MediaBrowser Token="${TOKEN}"` } : {});

// mn: own match — each library's films as the person may see them, the
// whole list, matched here: Jellyfin's own search matches by substring
// and does not forgive a slip.
const LIST =
  'Items?Recursive=true&IncludeItemTypes=Movie&Fields=ProviderIds,ProductionYear&EnableImages=false';

async function listed(lib) {
  const got = await ask(`${lib.base}${LIST}`);
  // mn: plant asleep-empty — bar 4's plant: a library that did not answer
  // shown as one with nothing
  if (PLANT === 'asleep-empty' && got.state === 'away') return { state: 'answered', copies: [] };
  // mn: own match
  if (got.state !== 'answered') return got;
  return {
    state: 'answered',
    copies: (got.body.Items ?? []).map((item) => copyOf(item, lib.library)),
  };
}

// mn: own cards — a search: every library asked at once, each said as it
// answers, the cards drawn again with each answer, and the whole kept for
// this tab so the browser's Back finds it as it was.
let asked = 0;

async function search(query) {
  const n = ++asked;
  const found = await libraries();
  const libs = found?.libs ?? [];
  const asking = libs.map((l) => ({ library: l.library, who: l.who, state: 'asking', matched: 0 }));
  const away = (found?.unanswered ?? []).map((p) => ({
    library: p,
    who: `${p}’s computer`,
    state: 'unanswered',
  }));
  const fresh = (found?.fresh ?? []).map((library) => ({ library, state: 'new' }));
  const state = {
    query,
    films: [],
    libs: found ? [...asking, ...away, ...fresh] : [{ state: 'untold' }],
  };
  const copies = [];
  render(state);
  if (!libs.length) return;
  await Promise.all(
    libs.map(async (lib, i) => {
      const got = await listed(lib);
      if (n !== asked) return;
      const near = (got.copies ?? []).map((c) => ({ ...c, d: distance(query, c.title) }));
      const mine = near.filter((c) => c.d !== null);
      // mn: own choose — each copy's address in its own Jellyfin
      for (const c of mine) c.href = itemAt(lib.open, c);
      // mn: own cards — each library's count is of the films shown, the
      // nearest of every library's, so it is counted again with each answer
      copies.push(...mine.map((c) => ({ ...c, who: lib.who, at: i })));
      state.libs[i] = { ...state.libs[i], state: got.state, why: got.why };
      state.films = closest(films(copies));
      const shown = state.films.flatMap((f) => f.copies);
      for (const [j, l] of state.libs.entries())
        if (l.state === 'answered') l.matched = shown.filter((c) => c.at === j).length;
      render(state);
    }),
  );
}

const el = (tag, cls, text) => {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text !== undefined) e.textContent = text;
  return e;
};

// What a library said, in the person's words.
const WORDS = {
  asking: (l) => `Asking ${l.who}…`,
  answered: (l) =>
    l.matched
      ? `${l.who}: ${l.matched} ${l.matched === 1 ? 'film' : 'films'}`
      : `${l.who}: nothing by that name`,
  away: (l) => `${l.who} did not answer: its computer may be asleep. Its films are not here.`,
  closed: (l) =>
    `${l.who} doesn’t let Movie Night search it yet. Only ${l.library} can change that.`,
  refused: (l) => `${l.who} would not let you in.`,
  failed: (l) => `${l.who} could not be read from here.`,
  unanswered: (l) => `${l.who} did not answer, so no library of theirs is here.`,
  new: (l) => `${l.library}’s films are new. Open Movie Night from your home page to search them.`,
  untold: () =>
    'Movie Night could not learn which libraries are open to it. Open it again from your group’s page.',
};

function card(film) {
  const li = el('li', 'card');
  li.append(el('h2', null, film.year ? `${film.title} (${film.year})` : film.title));
  const list = el('ul', 'copies');
  // mn: own choose — the first copy offered first, the others under it
  film.copies.forEach((c, i) => {
    const a = el('a', i ? 'copy' : 'copy first', i ? `or from ${c.who}` : `Watch from ${c.who}`);
    a.href = c.href;
    a.dataset.library = c.library;
    const item = el('li');
    item.append(a);
    list.append(item);
  });
  // mn: own cards
  li.append(list);
  return li;
}

function render(state) {
  box.value = state.query;
  said.replaceChildren(
    ...(state.libs.length ? state.libs : [{ state: 'none' }]).map((l) => {
      const li = el(
        'li',
        null,
        l.state === 'none' ? 'No library is open to Movie Night here.' : WORDS[l.state](l),
      );
      li.dataset.state = l.state;
      if (l.library) li.dataset.library = l.library;
      return li;
    }),
  );
  results.replaceChildren(...state.films.map(card));
  sessionStorage.setItem(KEPT, JSON.stringify(state));
}

form.addEventListener('submit', (e) => {
  e.preventDefault();
  search(box.value.trim());
});

// Back from a Jellyfin, the last search as it was.
const last = JSON.parse(sessionStorage.getItem(KEPT) ?? 'null');
if (last) render(last);
// A cold page can draw before this module arrives. Enable only the bound form.
box.disabled = false;
const submit = form.querySelector('button');
submit.textContent = 'Search';
submit.disabled = false;
console.debug('movie-night: ready');
