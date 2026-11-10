// Taste, carried in a group's folder (svrn-docs rings/current/CONTRACTS.md
// §4, §7): no server of its own. It reads each Jellyfin its person said yes
// to (reads.txt), sends the whole list of films, described, to the first
// llama-server it may send to (sends.txt) at /v1/embeddings, in runs of 64
// as ../model.mjs does, and guesses from the ratings kept in this browser
// (../score.mjs's rule: the sum of rated films' numbers, weighted by
// rating, against each film by cosine). Nothing it rates leaves this
// browser: the list sent is every film, in one order, whatever was rated.
const $ = (id) => document.getElementById(id);
const LIST =
  'Items?Recursive=true&IncludeItemTypes=Movie&Fields=Overview,Genres,People,ProductionYear&EnableImages=false';
const RUN = 64;
const KEPT = 'taste-ratings';
const PULL = { 'Loved it': 1, 'Liked it': 0.5, 'Not for me': -1 };

const ratings = () => JSON.parse(localStorage.getItem(KEPT) ?? '{}');
const say = (words) => {
  $('said').textContent = words;
};

// What a model reads of a film, as ../model.mjs `describe` says it.
const describe = (f) =>
  [
    f.ProductionYear ? `${f.Name} (${f.ProductionYear}).` : `${f.Name}.`,
    f.Genres?.length ? `${f.Genres.join(', ')}.` : '',
    f.Overview ?? '',
    f.People?.length
      ? `With ${f.People.slice(0, 6)
          .map((p) => p.Name)
          .join(', ')}.`
      : '',
  ]
    .filter(Boolean)
    .join(' ');

// Every film of every library, by library then the library's own id.
async function films(libraries) {
  const all = [];
  for (const lib of libraries) {
    const res = await fetch(`${lib.address}${LIST}`, { headers: { accept: 'application/json' } });
    if (!res.ok) continue;
    for (const item of (await res.json()).Items ?? []) all.push({ library: lib.address, ...item });
  }
  return all.sort((a, b) => `${a.library} ${a.Id}`.localeCompare(`${b.library} ${b.Id}`));
}

// Each film's numbers from `model`, in the list's order, the whole list.
async function numbers(list, model) {
  const vectors = [];
  for (let i = 0; i < list.length; i += RUN) {
    const input = list.slice(i, i + RUN).map(describe);
    const res = await fetch(`${model.address}v1/embeddings`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input, encoding_format: 'float' }),
    });
    if (!res.ok) throw new Error(await refusal(res, model));
    const data = [...((await res.json()).data ?? [])].sort((a, b) => a.index - b.index);
    vectors.push(...data.map((d) => d.embedding));
  }
  return vectors;
}

const unit = (v) => {
  const n = Math.sqrt(v.reduce((s, x) => s + x * x, 0));
  return n ? v.map((x) => x / n) : v;
};
const dot = (a, b) => a.reduce((s, x, i) => s + x * b[i], 0);

function draw(list, vectors) {
  const rated = ratings();
  const taste = list.reduce((sum, f, i) => {
    const pull = PULL[rated[f.Id]] ?? 0;
    return unit(vectors[i]).map((x, k) => (sum[k] ?? 0) + pull * x);
  }, []);
  const scored = list
    .map((f, i) => ({ f, score: rated[f.Id] ? null : dot(unit(taste), unit(vectors[i])) }))
    .filter((s) => s.score !== null && Object.keys(rated).length)
    .sort((a, b) => b.score - a.score);
  $('likely').hidden = scored.length === 0;
  $('likely')
    .querySelector('ol')
    .replaceChildren(
      ...scored
        .filter((s) => s.score > 0)
        .map((s) => Object.assign(document.createElement('li'), { textContent: s.f.Name })),
    );
  $('films')
    .querySelector('ol')
    .replaceChildren(
      ...list.map((f) => {
        const li = Object.assign(document.createElement('li'), { textContent: `${f.Name} ` });
        li.dataset.title = f.Name;
        for (const words of Object.keys(PULL)) {
          const b = Object.assign(document.createElement('button'), { textContent: words });
          b.disabled = rated[f.Id] === words;
          b.onclick = () => {
            localStorage.setItem(KEPT, JSON.stringify({ ...ratings(), [f.Id]: words }));
            draw(list, vectors);
          };
          li.append(b);
        }
        return li;
      }),
    );
}

// What a refused send says to its person: svrngs's own words when it held
// the line (its JSON `error`, which says what to do), else the service's
// status and answer as given.
async function refusal(res, model) {
  const text = await res.text();
  let said;
  try {
    said = JSON.parse(text).error;
  } catch {
    said = undefined;
  }
  return said ?? `${model.lender}’s ${model.kind} answered ${res.status}: ${text}`;
}

// What its person said yes to it reading, and sending to, each at its address.
const lends = (await (await fetch('svrngs-reads')).json()).lends ?? [];
const libraries = lends.filter((l) => l.kind === 'jellyfin');
const model = lends.find((l) => l.kind === 'llama-server');
if (!model)
  say(
    'Taste needs a friend’s llama-server, and your yes to send it your friends’ films: open Taste from your home page and tick it.',
  );
else {
  const list = await films(libraries);
  say(
    `${list.length} films in your friends’ libraries. Asking ${model.lender}’s ${model.kind} what each is like…`,
  );
  try {
    const vectors = await numbers(list, model);
    say(
      `${model.lender}’s ${model.kind} described ${vectors.length} films. Rate a few you’ve seen.`,
    );
    draw(list, vectors);
  } catch (e) {
    say(e.message);
  }
}
