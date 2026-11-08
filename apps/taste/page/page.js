// Taste's own page, served by Ana's taste on her computer and lent by her,
// so it runs at her own origin: what it is told of her taste goes back to
// her taste and nowhere else. It reads each library her groups lend by the
// account its lender keeps for apps that read it (`svrngs share
// --readers`; DESIGN §3, "Lent apps, as they come" 4), hands taste the
// whole list, and shows what taste says. It keeps nothing itself.
const $ = (id) => document.getElementById(id);
const said = $('said');
const guessed = $('guessed');

// What a library is asked: every film, with what describes it.
const LIST =
  'Items?Recursive=true&IncludeItemTypes=Movie&Fields=Overview,Genres,People,ProviderIds,ProductionYear&EnableImages=false';

const el = (tag, text, cls) => {
  const e = document.createElement(tag);
  if (text !== undefined) e.textContent = text;
  if (cls) e.className = cls;
  return e;
};

// svrngs tells a page at its own origin each library it may read, whose
// it is and where to ask it (DESIGN §3, "Lent apps", call 6).
async function libraries() {
  try {
    const r = await fetch('/svrngs-reads');
    return r.ok ? r.json() : null;
  } catch {
    return null;
  }
}

// A library's films, or what kept them from here, told apart as Movie
// Night tells them (apps/movie-night/app.js `ask`): 502 and 504 are the
// lender's computer not answering; 401, no account kept for apps.
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
  const library = new URL(lib.address).hostname.split('.')[0];
  const items = (await res.json()).Items ?? [];
  return {
    state: 'answered',
    films: items.map((i) => ({
      library,
      id: i.Id,
      title: i.Name,
      year: i.ProductionYear ?? null,
      ids: i.ProviderIds ?? {},
      genres: i.Genres ?? [],
      overview: i.Overview ?? '',
      people: (i.People ?? []).map((p) => p.Name),
    })),
  };
}

const WORDS = {
  answered: (l, n) => `${l.lender}’s library: ${n} ${n === 1 ? 'film' : 'films'}`,
  away: (l) => `${l.lender}’s computer didn’t answer. It may be asleep, so its films aren’t here.`,
  closed: (l) =>
    `${l.lender}’s library doesn’t let taste read it yet. Only ${l.lender} can change that.`,
  refused: (l) => `${l.lender}’s library wouldn’t let you in.`,
  failed: (l) => `${l.lender}’s library couldn’t be read from here.`,
};

// POST to Ana's taste, at this page's own origin: its answer, or its words
// when it refused.
async function taste(path, body) {
  const res = await fetch(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  const json = await res.json().catch(() => null);
  if (!res.ok) throw new Error(json?.error ?? `Taste couldn’t answer (${res.status}).`);
  return json;
}

const RATINGS = [
  ['loved', 'Loved it', 5],
  ['liked', 'Liked it', 4],
  ['not-for-me', 'Not for me', 1],
];
const loved = (stars) => (stars >= 4.5 ? 'loved' : 'liked');

let all = [];

function row(f) {
  const li = el('li');
  li.dataset.title = f.title;
  li.append(el('strong', f.year ? `${f.title} (${f.year})` : f.title));
  li.append(el('span', ` in ${f.whose}’s library`, 'whose'));
  if (f.because) li.append(el('span', `Because you ${f.becauseSaid} ${f.because}`, 'because'));
  const buttons = el('span', undefined, 'rate');
  for (const [said, words, stars] of RATINGS) {
    const b = el('button', words);
    b.type = 'button';
    b.setAttribute('aria-pressed', String(f.mine !== null && Math.round(f.mine) === stars));
    b.addEventListener('click', () =>
      rate(f, f.mine !== null && Math.round(f.mine) === stars ? null : said),
    );
    buttons.append(b);
  }
  li.append(buttons);
  return li;
}

// Whose library each film is in, by its library and id: the page's own,
// so taste is told nothing of who lends what.
const whose = new Map();
const key = (f) => `${f.library}\u0000${f.id}`;

function show(ranked) {
  for (const f of ranked.films) f.whose = whose.get(key(f));
  const byTitle = new Map(ranked.films.map((f) => [f.title, f]));
  for (const f of ranked.films) {
    const from = byTitle.get(f.because);
    f.becauseSaid = from?.mine ? loved(from.mine) : 'liked';
  }
  const parts = { likely: [], unguessed: [], unlikely: [], rated: [] };
  for (const f of ranked.films) {
    if (f.mine !== null) parts.rated.push(f);
    else if (f.guess === null) parts.unguessed.push(f);
    else parts[f.guess > 0 ? 'likely' : 'unlikely'].push(f);
  }
  // Nothing is guessed until she rates something.
  $('unguessed').querySelector('h2').textContent = parts.rated.length
    ? 'Not guessed yet'
    : 'Rate a few films you’ve seen, and taste starts guessing';
  for (const [id, list] of Object.entries(parts)) {
    $(id).hidden = list.length === 0;
    $(id)
      .querySelector('ol')
      .replaceChildren(...list.map(row));
  }
  const elsewhere = ranked.elsewhere
    ? ` You also rated ${ranked.elsewhere} ${ranked.elsewhere === 1 ? 'film' : 'films'} no library here has.`
    : '';
  guessed.textContent = ranked.model
    ? `Guesses use ${ranked.model}. It gets the list of films in your friends’ libraries, never what you rated.${elsewhere}`
    : `Nothing is guessed yet: ${ranked.unguessed}${elsewhere}`;
}

async function rank() {
  try {
    show(await taste('/rank', { films: all }));
  } catch (e) {
    guessed.textContent = e.message;
    guessed.dataset.state = 'refused';
  }
}

async function rate(f, said) {
  const film = { title: f.title, year: f.year, ids: f.ids };
  try {
    await taste('/rate', { film, said });
  } catch (e) {
    guessed.textContent = e.message;
    guessed.dataset.state = 'refused';
    return;
  }
  await rank();
}

$('file').addEventListener('change', async (e) => {
  const file = e.target.files?.[0];
  if (!file) return;
  const brought = $('brought');
  try {
    const kept = await taste('/import', { text: await file.text() });
    brought.textContent = `Kept ${kept.kept} ${kept.kept === 1 ? 'rating' : 'ratings'} from Letterboxd.`;
  } catch (err) {
    brought.textContent = err.message;
    return;
  }
  await rank();
});

async function start() {
  const told = await libraries();
  if (!told) {
    said.replaceChildren(
      el('li', 'Open taste from your svrngs home page, so it can see your friends’ libraries.'),
    );
    return;
  }
  const libs = told.lends.filter((l) => l.kind === 'jellyfin');
  const lines = libs.map((l) => el('li', `Asking ${l.lender}’s library…`));
  for (const p of told.unanswered ?? [])
    lines.push(el('li', `${p}’s computer didn’t answer, so no library of theirs is here.`));
  if (!libs.length) lines.push(el('li', 'No one in your groups shares a film library yet.'));
  said.replaceChildren(...lines);
  const got = await Promise.all(libs.map(films));
  got.forEach((g, i) => {
    lines[i].textContent = WORDS[g.state](libs[i], g.films?.length ?? 0);
    lines[i].dataset.state = g.state;
  });
  got.forEach((g, i) => g.films?.forEach((f) => whose.set(key(f), libs[i].lender)));
  all = got.flatMap((g) => g.films ?? []);
  await rank();
  document.body.dataset.ready = 'yes';
}

start();
