// A lend's audience, on the home page with no browser: a new lend
// (home/page/lend-found.js LEND) is made to an explicit set of stable
// group ids, never to everyone by default, and a change of a lend
// (home/page/lend-change.js CHANGE) edits only its audience, preserving
// the lend. Each effect is read as data.
//
// Plant, watched failing at HEAD (git show HEAD:home/page/lend-found.js has
// no `choose`): LEND had no `choose`, so a ticked group was refused ("a
// program found takes no choose") and `lend` carried no `groups`; the
// planted law fails all its cases, `not ok 1..N`.
//
// A lend's row's words for its audience (lent-row.js `lentTo`) read
// `groups` from the owner's `/lent`: `[]` paused, `null` legacy needing
// selection (SCOPE_PARALLEL.md, owner `GET /lent`). Planted: shown
// `[]` and `null` alike, the paused/legacy case failed.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { LEND: lent } =
  await import('../../../crates/svrngs-node/src/home/page/lend-found.js?v=__VERSION__');
const { CHANGE } =
  await import('../../../crates/svrngs-node/src/home/page/lend-change.js?v=__VERSION__');
const { lentTo } =
  await import('../../../crates/svrngs-node/src/home/page/lent-row.js?v=__VERSION__');
const { GroupChoice } =
  await import('../../../crates/svrngs-node/src/home/page/group-choice.js?v=__VERSION__');

// The words a vnode tree draws, in order (fallback.test.mjs `words`).
const words = (v) =>
  v == null || typeof v === 'boolean'
    ? ''
    : typeof v !== 'object'
      ? String(v)
      : Array.isArray(v)
        ? v.map(words).join('')
        : words(v.props?.children);

const LEND = lent(() => {}),
  NEW = { at: { lend: 'unlent' }, port: 8096, name: '', kind: '', said: '', groups: [] };
const run = (m, s, ...ms) =>
  ms.reduce(
    ([s, out], msg) => ((r) => [r.state, [...out, r.refused ?? r.effects]])(m.step(s, msg)),
    [s, []],
  );

test('a new lend chooses its groups and sends exactly those stable ids', () => {
  const [s, out] = run(
    LEND,
    NEW,
    { type: 'choose', id: 'house', on: true },
    { type: 'choose', id: 'trip', on: true },
    { type: 'choose', id: 'trip', on: false },
    { type: 'lend' },
  );
  assert.deepEqual(s.groups, ['house'], 'only the ticked group, by id');
  assert.deepEqual(out[3], [
    {
      run: 'lend',
      port: 8096,
      name: null,
      kind: null,
      groups: ['house'],
      terms: null,
      failed: 'refused',
    },
  ]);
});

// The terms a group's own programs file adds, drawn by the first press
// (home.rs `drawn`), are posted back by the next, so the lend keeps what its
// lender saw (CONTRACTS §6.8); a tick lets them go, to be read again.
// Planted, watched failing: the effect's `terms` always null.
test('terms drawn are posted back with the next press, and a tick lets them go', () => {
  const drawn = { type: 'drawn', terms: '- notes keeps: /admin\n', why: 'Your group’s terms…' };
  const [s, out] = run(
    LEND,
    NEW,
    { type: 'choose', id: 'house', on: true },
    { type: 'lend' },
    drawn,
    { type: 'lend' },
  );
  assert.deepEqual([s.at.lend, out[2]], ['lending', []]);
  assert.equal(out[3][0].terms, '- notes keeps: /admin\n', 'the text drawn, posted back');
  const [after, again] = run(
    LEND,
    s,
    { type: 'refused', why: 'no' },
    { type: 'choose', id: 'trip', on: true },
    { type: 'lend' },
  );
  assert.equal(after.terms, undefined);
  assert.equal(again[2][0].terms, null, 'read again for the groups ticked now');
});

test('a new lend with no group chosen is refused, never made to everyone', () => {
  const [s, out] = run(LEND, NEW, { type: 'lend' });
  assert.deepEqual(out[0], []);
  assert.deepEqual([s.at.lend, s.said], ['unlent', 'Choose which groups can open this first.']);
});

test('a tick never expands the audience: it adds or removes just its own id', () => {
  const [s] = run(
    LEND,
    NEW,
    { type: 'choose', id: 'house', on: true },
    { type: 'choose', id: 'trip', on: true },
  );
  assert.deepEqual(s.groups, ['house', 'trip'], 'exactly the ticked ids, in tick order');
});

const rows = CHANGE('jellyfin', () => {}),
  ROW = { at: { change: 'idle' }, said: '', groups: [] };

test('a change sends only the audience, preserving every other setting', () => {
  const [s, out] = run(
    rows,
    ROW,
    { type: 'choose', id: 'house', on: true },
    { type: 'choose', id: 'trip', on: true },
    { type: 'audience' },
  );
  assert.deepEqual(s.groups, ['house', 'trip']);
  assert.deepEqual(out[2], [
    { run: 'change', to: { groups: ['house', 'trip'] }, failed: 'refused' },
  ]);
});

test('a limit change keeps the row’s chosen audience', () => {
  const [s] = run(rows, { ...ROW, groups: ['house'] }, { type: 'many', value: '3' });
  assert.deepEqual(s.groups, ['house'], 'the audience is not touched by a limit change');
});

test('saving the audience twice while one is under way is refused, then retried', () => {
  const [s1, out] = run(rows, { ...ROW, groups: ['house'] }, { type: 'audience' });
  assert.deepEqual(out[0], [{ run: 'change', to: { groups: ['house'] }, failed: 'refused' }]);
  assert.equal(s1.at.change, 'changing');
  const [s2] = run(rows, s1, { type: 'audience' });
  assert.equal(s2.at.change, 'changing', 'the second press changed nothing');
  const [s3] = run(rows, s2, { type: 'refused', why: 'busy' });
  assert.equal(s3.at.change, 'refused');
  const [s4, out4] = run(rows, s3, { type: 'audience' });
  assert.deepEqual(out4[0], [{ run: 'change', to: { groups: ['house'] }, failed: 'refused' }]);
  assert.equal(s4.at.change, 'changing', 'a retry after a refusal is taken');
});

const CHOICES = [
  { id: 'house', name: 'House' },
  { id: 'trip', name: 'Trip' },
];

test('a row’s words read paused from [], legacy needing selection from null, names otherwise', () => {
  assert.equal(lentTo({ groups: [] }, CHOICES), 'Paused: no group can open it.');
  assert.equal(lentTo({ groups: null }, CHOICES), 'Choose which groups can open this.');
  assert.equal(
    lentTo({ groups: ['house', 'trip'] }, CHOICES),
    'Lent to House and Trip.',
    'the selected rings, shown by name',
  );
});

test('two groups of one name are told apart in the boxes by who made them', () => {
  const boxes = words(
    GroupChoice({
      groups: [
        { id: 'a', name: 'Trip', made: { name: 'Trip', maker: 'sam', since: 1 } },
        { id: 'b', name: 'Trip', made: null },
      ],
      ticked: [],
      send: () => {},
    }),
  );
  assert.ok(boxes.includes('Trip (sam’s'), boxes);
  assert.ok(boxes.includes('Trip'), 'the person’s own group keeps the bare name');
});

// A row names every id of its audience (audit F10): one this computer holds
// no group for is said by its id's start, never dropped, so no row reads
// "Lent to ."; two groups of one name are told apart as the boxes tell them
// (group-choice.js). Planted: the ids HOME does not hold filtered out, and
// a bare name for each.
test('a row names every audience id, unheld ones by their id, namesakes apart', () => {
  const gone = '0123456789abcdef';
  assert.equal(
    lentTo({ groups: ['house', gone] }, CHOICES),
    'Lent to House and a group this computer does not hold (01234567).',
  );
  const none = lentTo({ groups: [gone] }, CHOICES);
  assert.notEqual(none, 'Lent to .');
  assert.ok(none.includes('01234567'), none);
  const trips = [
    { id: 'a', name: 'Trip', made: { name: 'Trip', maker: 'sam', since: 1 } },
    { id: 'b', name: 'Trip', made: null },
  ];
  const both = lentTo({ groups: ['a', 'b'] }, trips);
  assert.ok(both.startsWith('Lent to Trip (sam’s'), both);
  assert.ok(both.endsWith(' and Trip.'), both);
});
