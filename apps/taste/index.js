// Taste, carried in a group's folder (svrn-docs rings/current/CONTRACTS.md
// §4, §7): no server of its own. It reads each Jellyfin its person said yes
// to (reads.txt), sends the whole list of films, described, to the first
// llama-server it may send to (sends.txt) at /v1/embeddings, in runs of 64,
// and guesses from the ratings kept in this browser. Nothing it rates
// leaves this browser: the list sent is every film, in one order, whatever
// was rated.
const $ = (id) => document.getElementById(id);
const LIST =
  'Items?Recursive=true&IncludeItemTypes=Movie&Fields=Overview,Genres,People,ProviderIds,ProductionYear&EnableImages=false';
const RUN = 64;
// The first time is a short exercise: this many films rated before a
// guess, since fewer say too little; in a smaller library, all but one.
const FIRST = 5;
// Kept in this browser alone: the ratings, and the film its person left to
// watch, until they say how it was.
const DIARY = 'taste-diary';
const WATCHING = 'taste-watching';
// Each rating's words and what it keeps, in Letterboxd's stars (0.5 to 5),
// so a rating made here and one brought from Letterboxd read alike.
const RATINGS = [
  ['Loved it', 5],
  ['Liked it', 4],
  ['Not for me', 1],
];
// A rating's pull on a taste: 5 stars is 1, 3 is nothing, 1 is -1.
const pull = (stars) => (stars - 3) / 2;

const el = (tag, text, cls) => {
  const e = document.createElement(tag);
  if (text !== undefined) e.textContent = text;
  if (cls) e.className = cls;
  return e;
};
const say = (words) => {
  $('said').textContent = words;
};

// A browser may refuse its storage (a private window, blocked site data): a
// read refused is nothing kept, a write refused is said.
const kept = (key) => {
  try {
    return JSON.parse(localStorage.getItem(key) ?? 'null');
  } catch {
    return null;
  }
};
const keep = (key, value) => {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, JSON.stringify(value));
    return true;
  } catch {
    return false;
  }
};

const plain = (s) =>
  String(s ?? '')
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '');

// Whether a film and a rating are one film: by a provider id both name, else
// by title and year. A rating keeps no library, so a film is its person's
// wherever they found it.
function same(film, row) {
  const ids = Object.keys(film.ids ?? {}).filter((k) => k in (row.ids ?? {}));
  if (ids.length) return ids.some((k) => String(film.ids[k]) === String(row.ids[k]));
  return plain(film.title) === plain(row.title) && Number(film.year) === Number(row.year);
}

let rows = kept(DIARY) ?? [];
let all = [];
let vectors = null;
let model = null;

function saved() {
  if (!keep(DIARY, rows))
    $('guessed').textContent =
      'This browser won’t keep your ratings, so Taste forgets them when you close it.';
}

// `film` rated `stars`, replacing any rating of it; `null` takes it back.
function rate(film, stars) {
  rows = rows.filter((r) => !same(film, r));
  if (stars !== null) {
    const { title, year, ids } = film;
    rows.push({ title, year, ids, rating: stars, at: new Date().toISOString() });
  }
  saved();
  draw();
}

// What a model reads of a film: its name and year, genres, overview, and
// the first six people in it.
const describe = (i) =>
  [
    i.ProductionYear ? `${i.Name} (${i.ProductionYear}).` : `${i.Name}.`,
    i.Genres?.length ? `${i.Genres.join(', ')}.` : '',
    i.Overview ?? '',
    i.People?.length
      ? `With ${i.People.slice(0, 6)
          .map((p) => p.Name)
          .join(', ')}.`
      : '',
  ]
    .filter(Boolean)
    .join(' ');

// A library's films, or what kept them from here, told apart as Movie
// Night tells them: 502 and 504 are the lender's computer not answering;
// 401, no account kept for apps.
async function films(lib) {
  let res;
  try {
    res = await fetch(`${lib.address}${LIST}`, { headers: { accept: 'application/json' } });
  } catch {
    return { state: 'failed' };
  }
  if (res.status === 502 || res.status === 504) return { state: 'away' };
  if (res.status === 401) return { state: 'closed' };
  if (!res.ok) return { state: 'refused' };
  const items = (await res.json()).Items ?? [];
  return {
    state: 'answered',
    films: items.map((i) => ({
      library: lib.address,
      id: i.Id,
      title: i.Name,
      year: i.ProductionYear ?? null,
      ids: i.ProviderIds ?? {},
      whose: lib.lender,
      // Its own page in its lender's Jellyfin, opened by svrngs's `to`, as
      // Movie Night opens one (apps/movie-night/match.js `itemAt`).
      href: lib.to
        ? `${lib.to}#/details?id=${i.Id}${i.ServerId ? `&serverId=${i.ServerId}` : ''}`
        : null,
      text: describe(i),
    })),
  };
}

const WORDS = {
  answered: (l, n) => `${l.lender}’s library: ${n} ${n === 1 ? 'film' : 'films'}`,
  away: (l) => `${l.lender}’s computer didn’t answer. It may be asleep, so its films aren’t here.`,
  closed: (l) =>
    `${l.lender}’s library doesn’t let apps read it yet. Only ${l.lender} can change that.`,
  refused: (l) => `${l.lender}’s library wouldn’t let Taste in.`,
  failed: (l) => `${l.lender}’s library couldn’t be read from here.`,
};

// What a refused send says to its person: svrngs's own words when it held
// the line (its JSON `error`, which says what to do), else the service's
// status and answer as given.
async function refusal(res) {
  const text = await res.text();
  let said;
  try {
    said = JSON.parse(text).error;
  } catch {
    said = undefined;
  }
  return said ?? `${model.lender}’s ${model.kind} answered ${res.status}: ${text}`;
}

// Each film's numbers from the model, in the list's order, the whole list.
async function numbers(list) {
  const out = [];
  for (let i = 0; i < list.length; i += RUN) {
    const input = list.slice(i, i + RUN).map((f) => f.text);
    const res = await fetch(`${model.address}v1/embeddings`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input, encoding_format: 'float' }),
    });
    if (!res.ok) throw new Error(await refusal(res));
    const data = [...((await res.json()).data ?? [])].sort((a, b) => a.index - b.index);
    out.push(...data.map((d) => d.embedding));
  }
  return out;
}

const norm = (v) => Math.sqrt(v.reduce((s, x) => s + x * x, 0));
const unit = (v) => {
  const n = norm(v);
  return n ? v.map((x) => x / n) : v;
};
const dot = (a, b) => a.reduce((s, x, i) => s + x * b[i], 0);

// Each film scored against the ratings: its own rating if rated (`mine`),
// else how near it is to the taste (`score`, cosine, -1 to 1) and, when
// that is above nothing, the rated film that pulls it nearest (`because`).
// The taste is the sum of the rated films' numbers, each weighted by its
// rating's pull. A rating no library here holds counts for nothing: it has
// no numbers. In order: rated films by pull, others by score, one scale;
// a film with no score last; ties by title.
function scored() {
  const units = all.map((_, i) => (vectors?.[i] ? unit(vectors[i]) : null));
  const rated = [];
  for (const row of rows) {
    const copies = all.flatMap((f, i) => (same(f, row) && units[i] ? [units[i]] : []));
    if (!copies.length) continue;
    const v = unit(copies[0].map((_, k) => copies.reduce((s, c) => s + c[k], 0)));
    rated.push({ row, v, pull: pull(row.rating) });
  }
  const taste = (rated[0]?.v ?? []).map((_, k) => rated.reduce((s, r) => s + r.pull * r.v[k], 0));
  const toward = norm(taste) ? unit(taste) : null;
  const out = all.map((f, i) => {
    const mine = rows.find((r) => same(f, r));
    if (mine) return { ...f, mine: mine.rating, score: pull(mine.rating) };
    if (!units[i] || !toward) return { ...f, mine: null, score: null };
    const score = dot(toward, units[i]);
    const near = rated
      .filter((r) => r.pull > 0)
      .map((r) => ({ row: r.row, by: r.pull * dot(r.v, units[i]) }))
      .sort((a, b) => b.by - a.by)[0];
    const because = score > 0 && near?.by > 0 ? near.row : null;
    return { ...f, mine: null, score, because };
  });
  return out.sort(
    (a, b) => (b.score ?? -Infinity) - (a.score ?? -Infinity) || a.title.localeCompare(b.title),
  );
}

function row(f) {
  const li = el('li');
  li.dataset.title = f.title;
  li.append(el('strong', f.year ? `${f.title} (${f.year})` : f.title));
  li.append(el('span', ` in ${f.whose}’s library`, 'whose'));
  if (f.because) {
    const said = f.because.rating >= 4.5 ? 'loved' : 'liked';
    li.append(el('span', `Because you ${said} ${f.because.title}`, 'because'));
  }
  if (f.mine === null && f.href) {
    const a = el('a', `Watch from ${f.whose}’s library`, 'watch');
    a.href = f.href;
    a.addEventListener('click', () => keep(WATCHING, { title: f.title, year: f.year, ids: f.ids }));
    li.append(a);
  }
  const buttons = el('span', undefined, 'rate');
  for (const [words, stars] of RATINGS) {
    const on = f.mine !== null && Math.round(f.mine) === stars;
    const b = el('button', words);
    b.type = 'button';
    b.setAttribute('aria-pressed', String(on));
    b.addEventListener('click', () => rate(f, on ? null : stars));
    buttons.append(b);
  }
  li.append(buttons);
  return li;
}

function draw() {
  const parts = { likely: [], unguessed: [], unlikely: [], rated: [] };
  for (const f of scored()) {
    if (f.mine !== null) parts.rated.push(f);
    else if (f.score === null) parts.unguessed.push(f);
    else parts[f.score > 0 ? 'likely' : 'unlikely'].push(f);
  }
  // Until the first few are rated, every film not rated waits to be.
  const first = Math.min(FIRST, all.length - 1);
  const waiting = parts.rated.length < first;
  if (waiting) parts.unguessed.push(...parts.likely.splice(0), ...parts.unlikely.splice(0));
  $('unguessed').querySelector('h2').textContent = waiting
    ? `First, rate ${first} films you’ve seen (${parts.rated.length} of ${first} so far)`
    : 'Not guessed yet';
  for (const [id, list] of Object.entries(parts)) {
    $(id).hidden = list.length === 0;
    $(id)
      .querySelector('ol')
      .replaceChildren(...list.map(row));
  }
  const elsewhere = rows.filter((r) => !all.some((f) => same(f, r))).length;
  const also = elsewhere
    ? ` You also rated ${elsewhere} ${elsewhere === 1 ? 'film' : 'films'} no library here has.`
    : '';
  $('guessed').textContent = vectors
    ? `Guesses use ${model.lender}’s ${model.kind}. It gets the list of films in your friends’ libraries, never what you rated.${also}`
    : also.trim();
}

// After watching: the film its person left for, asked about once on the
// way back, then forgotten.
function after() {
  const f = kept(WATCHING);
  $('after').hidden = !f;
  if (!f) return;
  $('after').querySelector('h2').textContent = `How was ${f.title}?`;
  const done = () => {
    keep(WATCHING, null);
    $('after').hidden = true;
  };
  const buttons = RATINGS.map(([words, stars]) => {
    const b = el('button', words);
    b.type = 'button';
    b.addEventListener('click', () => {
      done();
      rate(f, stars);
    });
    return b;
  });
  const not = el('button', 'I didn’t watch it');
  not.type = 'button';
  not.addEventListener('click', done);
  $('after')
    .querySelector('p')
    .replaceChildren(...buttons, not);
}
// Back from Jellyfin, the browser may show this page as it was left.
window.addEventListener('pageshow', (e) => e.persisted && after());

// One CSV line's fields, quotes kept as Letterboxd writes them.
function fields(line) {
  const out = [];
  let cur = '';
  let quoted = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (quoted && c === '"' && line[i + 1] === '"') {
      cur += '"';
      i++;
    } else if (c === '"') quoted = !quoted;
    else if (c === ',' && !quoted) {
      out.push(cur);
      cur = '';
    } else cur += c;
  }
  out.push(cur);
  return out;
}

// The ratings in a Letterboxd export's `ratings.csv` or `diary.csv` (Date,
// Name, Year, Letterboxd URI, Rating, …), the last of each film. A file
// that is neither throws, so the page says so and keeps nothing.
function letterboxd(text) {
  const lines = text.replace(/^﻿/, '').split(/\r?\n/).filter(Boolean);
  const head = fields(lines.shift() ?? '').map((h) => h.trim());
  const [name, year, rating] = ['Name', 'Year', 'Rating'].map((h) => head.indexOf(h));
  if (name < 0 || rating < 0) throw new Error('not a Letterboxd ratings or diary file');
  const got = [];
  for (const line of lines) {
    const f = fields(line);
    const stars = Number(f[rating]);
    if (!(stars > 0 && stars <= 5)) continue;
    const film = { title: f[name], year: year < 0 ? null : Number(f[year]) || null, ids: {} };
    const at = f[0] || null;
    const i = got.findIndex((r) => same(film, r));
    if (i >= 0) got[i] = { ...film, rating: stars, at };
    else got.push({ ...film, rating: stars, at });
  }
  return got;
}

$('file').addEventListener('change', async (e) => {
  const file = e.target.files?.[0];
  if (!file) return;
  let got;
  try {
    got = letterboxd(await file.text());
  } catch {
    $('brought').textContent =
      'That isn’t a Letterboxd ratings file. Choose ratings.csv from the export.';
    return;
  }
  // A film also rated here keeps the newer of the two.
  for (const r of got) {
    const i = rows.findIndex((o) => same(r, o));
    if (i < 0) rows.push(r);
    else if (String(r.at ?? '') > String(rows[i].at ?? '')) rows[i] = r;
  }
  $('brought').textContent =
    `Kept ${got.length} ${got.length === 1 ? 'rating' : 'ratings'} from Letterboxd.`;
  saved();
  draw();
});

// What its person said yes to it reading, and sending to, each at its address.
let told = null;
try {
  const res = await fetch('svrngs-reads');
  if (res.ok) told = await res.json();
} catch {
  told = null;
}
if (!told) say('Open Taste from your svrngs home page, so it can see your friends’ libraries.');
else {
  const lends = told.lends ?? [];
  const libs = lends.filter((l) => l.kind === 'jellyfin');
  model = lends.find((l) => l.kind === 'llama-server') ?? null;
  const lines = libs.map((l) => el('li', `Asking ${l.lender}’s library…`));
  for (const p of told.unanswered ?? [])
    lines.push(el('li', `${p}’s computer didn’t answer, so no library of theirs is here.`));
  if (!libs.length) lines.push(el('li', 'No one in your groups shares a film library yet.'));
  $('libraries').replaceChildren(...lines);
  const got = await Promise.all(libs.map(films));
  got.forEach((g, i) => {
    lines[i].textContent = WORDS[g.state](libs[i], g.films?.length ?? 0);
    lines[i].dataset.state = g.state;
  });
  // Every film of every library, by library then the library's own id.
  all = got
    .flatMap((g) => g.films ?? [])
    .sort((a, b) => `${a.library} ${a.id}`.localeCompare(`${b.library} ${b.id}`));
  if (!model)
    say(
      'Taste needs a friend’s llama-server, and your yes to send it your friends’ films: open Taste from your home page and tick it.',
    );
  else {
    say(
      `${all.length} films in your friends’ libraries. Asking ${model.lender}’s ${model.kind} what each is like…`,
    );
    try {
      vectors = await numbers(all);
      say(
        `${model.lender}’s ${model.kind} described ${vectors.length} films. Rate a few you’ve seen.`,
      );
    } catch (e) {
      say(e.message);
    }
  }
  draw();
  after();
}
document.body.dataset.ready = 'yes';
