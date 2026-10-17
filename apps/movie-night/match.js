// mn: own match — Movie Night's understanding of a library's catalogue
// (demo/runs/mn/2026-10-08/README.md §4): which films a query names, and
// which copies are one film. Each line here is counted by that rule's marks.
// mn: plant switch
import { PLANT } from './plant.js';
// mn: own match

// A title as a person might type it: lower case, no accents, letters and
// digits alone, so "Blade Runner", "blade-runner" and "BLADE RUNNER" are one.
export const plain = (s) =>
  (s ?? '')
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '');

// The fewest letters changed, added or dropped to find `q` anywhere in `t`
// (Sellers' approximate substring match): `bladruner` is two from
// `bladerunner`, `brazl` one from `brazil`.
function within(q, t) {
  let row = new Array(t.length + 1).fill(0);
  for (let i = 1; i <= q.length; i++) {
    const next = [i];
    for (let j = 1; j <= t.length; j++) {
      next[j] = Math.min(row[j] + 1, next[j - 1] + 1, row[j - 1] + (q[i - 1] === t[j - 1] ? 0 : 1));
    }
    row = next;
  }
  return Math.min(...row);
}

// Whether a query names a title: one slip in four letters forgiven, at
// least one, so a short query stays strict.
export function names(query, title) {
  const q = plain(query);
  if (!q) return false;
  return within(q, plain(title)) <= Math.max(1, Math.floor(q.length / 4));
}

// A film as Jellyfin's `/Items` gives it, in Movie Night's words.
export const copyOf = (item, library) => ({
  library,
  id: item.Id,
  server: item.ServerId,
  title: item.Name,
  year: item.ProductionYear ?? null,
  ids: Object.fromEntries(Object.entries(item.ProviderIds ?? {}).map(([k, v]) => [k.toLowerCase(), String(v)])),
});

// Whether two copies are one film: by a provider id both name, where both
// have ids; else by title and year, and a title with no year is never
// merged, so two different films of one name stay apart.
function same(a, b) {
  const shared = Object.keys(a.ids).filter((k) => k in b.ids);
  if (shared.length) return shared.some((k) => a.ids[k] === b.ids[k]);
  if (Object.keys(a.ids).length && Object.keys(b.ids).length) return false;
  return a.year !== null && a.year === b.year && plain(a.title) === plain(b.title);
}

// Copies grouped into films, each film's copies in the order they came.
export function films(copies) {
  // mn: plant per-copy — bar 1's plant: each copy a card of its own
  if (PLANT === 'per-copy') return copies.map((c) => ({ title: c.title, year: c.year, copies: [c] }));
  // mn: own match
  const out = [];
  for (const c of copies) {
    const film = out.find((f) => f.copies.some((o) => same(o, c)));
    if (film) film.copies.push(c);
    else out.push({ title: c.title, year: c.year, copies: [c] });
  }
  return out;
}

// mn: own choose — the item's own page in its Jellyfin: jellyfin-web's
// route to an item, under the library's address.
export const itemAt = (base, copy) => `${base}web/#/details?id=${copy.id}&serverId=${copy.server}`;
