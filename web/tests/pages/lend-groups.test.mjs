// A lend's audience, on the home page with no browser: a new lend
// (home/page/lend-found.js LEND) is made to an explicit set of stable
// group ids, never to everyone by default, and a change of a lend
// (home/page/lend-change.js CHANGE) edits only its audience, preserving
// the lend. Each effect is read as data. Plant, watched failing: LEND
// had no `choose`, so a ticked group was refused ("a program found takes
// no choose") and `lend` carried no `groups`.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { LEND: lent } =
  await import('../../../crates/svrngs-node/src/home/page/lend-found.js?v=__VERSION__');
const { CHANGE } =
  await import('../../../crates/svrngs-node/src/home/page/lend-change.js?v=__VERSION__');

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
    { run: 'lend', port: 8096, name: null, kind: null, groups: ['house'], failed: 'refused' },
  ]);
});

test('a new lend with no group chosen is refused, never made to everyone', () => {
  const [s, out] = run(LEND, NEW, { type: 'lend' });
  assert.deepEqual(out[0], []);
  assert.deepEqual([s.at.lend, s.said], ['unlent', 'Choose which groups can open this first.']);
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
