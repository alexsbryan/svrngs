// The one scorer: how much Ana will probably like each film she has not
// rated, from the films she has. Each film is a list of numbers a model gave
// for its description (model.mjs); films alike are near. Her taste is the
// sum of her rated films' numbers, each weighted by her rating, so a film
// she loved pulls toward films like it and one not for her pushes away from
// films like that. An unrated film scores by how near it is to her taste
// (cosine similarity), from -1 to 1. Pure: nothing here reads a file or a
// network.
import { same } from './diary.mjs';

/// A rating's pull, from Letterboxd's stars: 5 is 1, 3 is nothing, 1 is -1.
export const pull = (stars) => (stars - 3) / 2;

const norm = (v) => Math.sqrt(v.reduce((s, x) => s + x * x, 0));
const unit = (v) => {
  const n = norm(v);
  return n ? v.map((x) => x / n) : v;
};
const dot = (a, b) => a.reduce((s, x, i) => s + x * b[i], 0);

/// Each of `films` (the catalog's, in its order) with `vectors[i]` its
/// numbers (none when no model answered), scored against `rows`, her
/// diary: her rating of it if she rated it (`mine`), else how near it is to
/// her taste (`score`) and, when that is above nothing, the film she rated
/// that pulls it nearest (`because`). In order: films she rated by her
/// rating's pull, others by score, as one scale from 1 to -1; a film no
/// number scores goes last; ties by title. A row no library holds counts for nothing: no model gave it
/// numbers, and none is asked to (model.mjs).
export function scored(films, vectors, rows, plant = null) {
  const units = films.map((_, i) => (vectors?.[i] ? unit(vectors[i]) : null));
  const rated = [];
  for (const row of rows) {
    const copies = films.flatMap((f, i) => (same(f, row) && units[i] ? [units[i]] : []));
    if (!copies.length) continue;
    const v = unit(copies[0].map((_, k) => copies.reduce((s, c) => s + c[k], 0)));
    rated.push({ row, v, pull: pull(row.rating) });
  }
  const dims = rated[0]?.v.length ?? 0;
  const taste = Array.from({ length: dims }, (_, k) =>
    rated.reduce((s, r) => s + r.pull * r.v[k], 0),
  );
  const tasteUnit = norm(taste) ? unit(taste) : null;
  const out = films.map((film, i) => {
    const mine = rows.find((r) => same(film, r));
    if (mine) return { film, mine: mine.rating, score: pull(mine.rating), because: null };
    if (!units[i] || !tasteUnit) return { film, mine: null, score: null, because: null };
    let score = dot(tasteUnit, units[i]);
    // A plant (taste.mjs `PLANT`): a scorer that knows nothing.
    if (plant === 'constant') score = 0;
    const near = rated
      .filter((r) => r.pull > 0)
      .map((r) => ({ title: r.row.title, by: r.pull * dot(r.v, units[i]) }))
      .sort((a, b) => b.by - a.by)[0];
    // A reason only for a film she'll probably like: the film she rated
    // that pulls it nearest.
    const because = score > 0 && near && near.by > 0 ? near.title : null;
    return { film, mine: null, score, because };
  });
  return out.sort(
    (a, b) =>
      (b.score ?? -Infinity) - (a.score ?? -Infinity) || a.film.title.localeCompare(b.film.title),
  );
}
