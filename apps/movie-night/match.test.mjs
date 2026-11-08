// Movie Night's match (match.js): which films a search shows. A title typed
// whole finds that film alone, a slip still finds it, and copies of one
// film in two libraries stay one card.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { closest, copyOf, distance, films } from './match.js';

const lib = (library, titles) =>
  titles.map(([Name, ProductionYear, Tmdb], i) => {
    const c = copyOf(
      { Id: `${library}${i}`, Name, ProductionYear, ProviderIds: { Tmdb } },
      library,
    );
    return { ...c, d: distance(QUERY.q, c.title) };
  });
const QUERY = { q: '' };
const shown = (q, ...libs) => {
  QUERY.q = q;
  const copies = libs.flatMap(([l, t]) => lib(l, t)).filter((c) => c.d !== null);
  return closest(films(copies)).map((f) => `${f.title}: ${f.copies.map((c) => c.library)}`);
};

const SAM = [
  'Sam',
  [
    ['Test Pattern One', 2026, '1'],
    ['Test Pattern Two', 2026, '2'],
    ['Test Pattern Three', 2026, '3'],
    ['Blade Runner', 1982, '78'],
  ],
];
const DEE = [
  'Dee',
  [
    ['Blade Runner', 1982, '78'],
    ['Brazil', 1985, '68'],
  ],
];

test('a title typed whole finds that film alone', () => {
  assert.deepEqual(shown('Test Pattern One', SAM, DEE), ['Test Pattern One: Sam']);
});

test('what every title shares finds them all', () => {
  assert.equal(shown('test pattern', SAM).length, 3);
});

test('a slip still finds the film, one card for its two copies', () => {
  assert.deepEqual(shown('bladruner', SAM, DEE), ['Blade Runner: Sam,Dee']);
  assert.deepEqual(shown('brazl', SAM, DEE), ['Brazil: Dee']);
});

test('nothing near is nothing', () => {
  assert.deepEqual(shown('Arrival', SAM, DEE), []);
});
