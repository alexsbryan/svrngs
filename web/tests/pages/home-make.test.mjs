// The home's MAKE (page-join (d); crates/svrngs-node/src/home/page/make.js),
// with no browser: each name typed is asked for its namesakes, counted
// (`ask` on `name`), and an older answer refused by name, where
// home.js@19b63b2b:420 compared the typed value; a second press refused while it
// makes; a refusal said once; made, the form empty again and HOME saying
// so (`take`). Each step's effects are read as data. Plant,
// watched failing: the counter taken out (`ask(s, 'name')` read as
// `[s.asked, 1]`), so every name is ask 1 and the older answer is drawn.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { MAKE: made, take } =
  await import('../../../crates/svrngs-node/src/home/page/make.js?v=__VERSION__');

const MAKE = made(() => {}),
  FIRST = { at: { make: 'typing' }, name: '', person: '', before: '', said: '' };
const run = (s, ...ms) =>
  ms.reduce(
    ([s, out], m) => ((r) => [r.state, [...out, r.refused ?? r.effects]])(MAKE.step(s, m)),
    [s, []],
  );

test('an older answer of namesakes is refused by name', () => {
  const [s, out] = run(
    FIRST,
    { type: 'typed', field: 'name', value: 'tri' },
    { type: 'typed', field: 'name', value: 'trip' },
    {
      type: 'checked',
      n: 2,
      words: 'You’re already in Trip (yours, since 1 Nov). This makes a second group called trip.',
    },
    { type: 'checked', n: 1, words: '' },
  );
  assert.deepEqual(out[0], [{ run: 'check', name: 'tri', n: 1, failed: 'checked' }]);
  assert.deepEqual(out[1], [{ run: 'check', name: 'trip', n: 2, failed: 'checked' }]);
  assert.equal(out[3], 'name 1 answered after name 2 was asked');
  assert.deepEqual(
    [s.at.make, s.before],
    [
      'checked',
      'You’re already in Trip (yours, since 1 Nov). This makes a second group called trip.',
    ],
  );
});

test('a second press is refused while it makes, and a refusal is said once', () => {
  const [s, out] = run(
    FIRST,
    { type: 'typed', field: 'name', value: 'Trip' },
    { type: 'typed', field: 'person', value: 'Dee' },
    { type: 'make' },
    { type: 'make' },
    { type: 'refused', why: 'a name cannot be empty' },
    { type: 'make' },
    { type: 'refused', why: 'a name cannot be empty' },
  );
  assert.deepEqual(out[2], [{ run: 'make', name: 'Trip', person: 'Dee', failed: 'refused' }]);
  assert.equal(out[3], 'a group is being made, and is answered first');
  assert.deepEqual([s.at.make, s.said], ['refused', ' a name cannot be empty']);
});

test('made, the form is empty again, and HOME says so from what was made', () => {
  const [s] = run(
    FIRST,
    { type: 'typed', field: 'name', value: 'Lab' },
    { type: 'make' },
    { type: 'made' },
  );
  assert.deepEqual([s.at.make, s.name, s.asked], ['typing', '', { name: 1 }]);
  const q = { made: 'Lab', also: '', unplaced: null };
  assert.deepEqual(
    take(
      { at: { groups: 'read', said: 'joined' }, q: { joined: 'Trip' } },
      { type: 'made', flow: 'make', q },
    ).state,
    { at: { groups: 'read', said: 'made' }, q },
  );
});
