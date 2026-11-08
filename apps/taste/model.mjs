// The one way a film leaves Ana's computer for a model: the whole list of
// films her group's libraries hold, as taste was handed it, in one order no
// rating of hers can change (sorted by library, then by the library's own
// id). So a friend's model learns what the libraries hold, which every
// member can read, and never which of those films she rated, nor when: a
// list asked once is kept by what it holds, so a rating asks nothing again.
// This file never imports diary.mjs; her rated films' numbers are looked up
// in the list's (score.mjs), and a film she rated that no library holds is
// sent to no model at all.
import { createHash } from 'node:crypto';

const text = (v, most) => (typeof v === 'string' ? v.slice(0, most) : '');
const words = (v, most) =>
  Array.isArray(v) ? v.filter((w) => typeof w === 'string' && w).slice(0, most) : [];

/// A film as a library listed it, kept to what describes it: never more
/// than the page was handed by the library, and nothing of hers.
function film(f) {
  if (!f || typeof f !== 'object') return null;
  const [library, id, title] = [text(f.library, 200), text(f.id, 100), text(f.title, 300)];
  if (!library || !id || !title) return null;
  const ids = {};
  for (const [k, v] of Object.entries(f.ids ?? {}))
    if (typeof v === 'string' || typeof v === 'number') ids[text(k, 40)] = String(v).slice(0, 60);
  const year = Number.isInteger(f.year) ? f.year : null;
  return {
    library,
    id,
    title,
    year,
    ids,
    genres: words(f.genres, 8),
    overview: text(f.overview, 1500),
    people: words(f.people, 6),
  };
}

/// The list handed, in its one order, or why it is not one: what a model is
/// ever sent is a Catalog's, made here and nowhere else.
const cmp = (x, y) => (x < y ? -1 : x > y ? 1 : 0);

export class Catalog {
  constructor(films) {
    // By code unit, not locale: the same on every computer.
    this.films = Object.freeze(films.sort((a, b) => cmp(a.library, b.library) || cmp(a.id, b.id)));
    this.texts = this.films.map(describe);
  }
}

/// The candidate list `handed` as a Catalog, or the reason it is not one.
export function catalog(handed) {
  if (!Array.isArray(handed)) return { why: 'films: a list of films' };
  if (handed.length > 5000) return { why: 'films: at most 5000' };
  const films = handed.map(film);
  if (films.some((f) => !f)) return { why: 'films: each with its library, id and title' };
  return { catalog: new Catalog(films) };
}

/// What a model reads of a film: its title and year, genres, what it is
/// about and who is in it, as the library describes it.
export function describe(f) {
  const parts = [f.year ? `${f.title} (${f.year}).` : `${f.title}.`];
  if (f.genres.length) parts.push(`${f.genres.join(', ')}.`);
  if (f.overview) parts.push(f.overview);
  if (f.people.length) parts.push(`With ${f.people.join(', ')}.`);
  return parts.join(' ');
}

/// How many films go in one ask: llama-server takes a list, and a library
/// of thousands is still sent whole, in order, a run of 64 at a time.
const RUN = 64;

/// The last list asked and what the model answered: one kept, by what the
/// list holds, so opening the page again with the same libraries asks none.
let kept = null;

/// Each film's numbers from `model` (a lend of a friend's, reached as Ana
/// through svrngs, or her own `--embed`), in the catalog's order: the whole
/// list, never a part of it.
export async function numbers(list, model, log) {
  if (!(list instanceof Catalog)) throw new Error('only a Catalog is sent to a model');
  // Kept by the very texts a model would be sent, in their order.
  const key = createHash('sha256').update(JSON.stringify(list.texts)).digest('hex');
  if (kept?.key === key && kept.model === model.at) {
    log('model: kept numbers for', list.texts.length, 'films; asked nothing');
    return kept.vectors;
  }
  const vectors = [];
  for (let i = 0; i < list.texts.length; i += RUN) {
    const input = list.texts.slice(i, i + RUN);
    const res = await fetch(`${model.at}/v1/embeddings`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        ...(model.token && { authorization: `Bearer ${model.token}` }),
      },
      body: JSON.stringify({ input, encoding_format: 'float' }),
      redirect: 'error',
      signal: AbortSignal.timeout(120_000),
    });
    if (!res.ok) throw new Error(`${model.who} answered ${res.status}: ${await res.text()}`);
    const data = (await res.json()).data ?? [];
    const got = [...data].sort((a, b) => a.index - b.index).map((d) => d.embedding);
    if (got.length !== input.length || got.some((v) => !Array.isArray(v)))
      throw new Error(`${model.who} answered ${got.length} of ${input.length}`);
    vectors.push(...got);
  }
  log('model: asked', model.who, 'for', vectors.length, 'films, the whole list in order');
  kept = { key, model: model.at, vectors };
  return vectors;
}
