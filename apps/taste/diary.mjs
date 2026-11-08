// Ana's diary: every film she rated, kept in her taste dir on her computer
// and nowhere else. Read where it is stored on each ask; written whole, so a
// rating cut short leaves the one before it. Nothing here asks a network:
// what leaves this computer is model.mjs's, which never imports this file.
import { readFileSync, renameSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

/// What each word on the page keeps, in Letterboxd's stars (0.5 to 5), so
/// a rating typed on the page and one imported read alike.
export const SAID = { loved: 5, liked: 4, 'not-for-me': 1 };

const plain = (s) =>
  String(s ?? '')
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '');

/// Whether `film` and `row` are one film: by a provider id both name, else
/// by title and year (the film the history names, taste's rule since su-0).
export function same(film, row) {
  const ids = Object.keys(film.ids ?? {}).filter((k) => k in (row.ids ?? {}));
  if (ids.length) return ids.some((k) => String(film.ids[k]) === String(row.ids[k]));
  return plain(film.title) === plain(row.title) && Number(film.year) === Number(row.year);
}

export function diary(dir) {
  try {
    return JSON.parse(readFileSync(join(dir, 'diary.json'), 'utf8')).rows ?? [];
  } catch (e) {
    if (e.code === 'ENOENT') return [];
    throw e;
  }
}

function keep(dir, rows) {
  const part = join(dir, '.diary.json.part');
  writeFileSync(part, JSON.stringify({ rows }, null, 1), { mode: 0o600 });
  renameSync(part, join(dir, 'diary.json'));
}

/// `film` rated `stars`, replacing any rating of that film; `null` takes it
/// back. The row keeps the film's title, year and provider ids, never which
/// library it was in: a film is hers, wherever she found it.
export function rate(dir, film, stars, at = new Date().toISOString()) {
  const rows = diary(dir).filter((r) => !same(film, r));
  if (stars !== null) {
    const ids = Object.fromEntries(Object.entries(film.ids ?? {}).filter(([, v]) => v));
    rows.push({ title: film.title, year: film.year ?? null, ids, rating: stars, at });
  }
  keep(dir, rows);
  return rows;
}

/// One CSV line's fields, quotes kept as Letterboxd writes them.
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

/// The rows of a Letterboxd export's `ratings.csv` or `diary.csv` (Date,
/// Name, Year, Letterboxd URI, Rating, …), the last rating of each film:
/// what it holds, and how many lines had no rating. A file that is neither
/// throws, so the page says so and keeps nothing.
export function letterboxd(text) {
  const lines = text.replace(/^﻿/, '').split(/\r?\n/).filter(Boolean);
  const head = fields(lines.shift() ?? '').map((h) => h.trim());
  const [name, year, rating, uri] = ['Name', 'Year', 'Rating', 'Letterboxd URI'].map((h) =>
    head.indexOf(h),
  );
  if (name < 0 || rating < 0) throw new Error('not a Letterboxd ratings or diary file');
  const rows = [];
  let unrated = 0;
  for (const line of lines) {
    const f = fields(line);
    const stars = Number(f[rating]);
    if (!f[rating] || !(stars > 0 && stars <= 5)) {
      unrated++;
      continue;
    }
    const film = { title: f[name], year: year < 0 ? null : Number(f[year]) || null, ids: {} };
    const kept = { ...film, rating: stars, at: f[0] || null, from: 'letterboxd' };
    if (uri >= 0 && f[uri]) kept.letterboxd = f[uri];
    const i = rows.findIndex((r) => same(film, r));
    if (i >= 0) rows[i] = kept;
    else rows.push(kept);
  }
  return { rows, unrated };
}

/// The Letterboxd rows kept beside hers: a film she also rated on the page
/// keeps the newer of the two.
export function imported(dir, rows) {
  const kept = diary(dir);
  for (const row of rows) {
    const i = kept.findIndex((r) => same(row, r));
    if (i < 0) kept.push(row);
    else if (String(row.at ?? '') > String(kept[i].at ?? '')) kept[i] = row;
  }
  keep(dir, kept);
  return kept;
}
