// The home's JOIN (page-join (c); crates/svrngs-node/src/home/v2/join.js),
// with no browser: each link typed is asked of the computer that sent it,
// counted, and an older answer refused by name (`late`), where home.js:411
// compared the typed value; a second press refused while it joins. Each
// step's effects are read as data. Plant, watched failing: `late` taken
// out of JOIN's step (`old = false`), so the older answer is drawn.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { JOIN } = await import('../../../crates/svrngs-node/src/home/v2/join.js?v=__VERSION__');

const FIRST = { at: { join: 'typing' }, link: '', person: '', why: '' };
// Each message in turn from `s`: the state, and every effect and refusal.
const run = (s, ...ms) => ms.reduce(([s, out], m) => ((r) => [r.state, [...out, r.refused ?? r.effects]])(JOIN.step(s, m)), [s, []]);

test('an older answer to what a link is for is refused by name', () => {
  const [s, out] = run(FIRST, { type: 'typed', field: 'link', value: 'svrngs://join/a' }, { type: 'typed', field: 'link', value: 'svrngs://join/ab' }, { type: 'checked', n: 2, for: ['Trip', null] }, { type: 'checked', n: 1, for: ['Lab', null] }, { type: 'refused', n: 1, why: 'not a whole link' });
  assert.deepEqual(out[0], [{ run: 'check', link: 'svrngs://join/a', n: 1, failed: 'refused' }]);
  assert.deepEqual(out[1], [{ run: 'check', link: 'svrngs://join/ab', n: 2, failed: 'refused' }]);
  assert.equal(out[3], 'link 1 answered after link 2 was asked');
  assert.equal(out[4], 'link 1 answered after link 2 was asked');
  assert.deepEqual([s.at.join, s.for, s.why], ['checked', ['Trip', null], '']);
});

test('the name the link joins as is the one asked, and a second press is refused', () => {
  const [s, out] = run(FIRST, { type: 'typed', field: 'link', value: 'l' }, { type: 'typed', field: 'person', value: ' Dee ' }, { type: 'join' }, { type: 'checked', n: 1, for: ['Trip', null] }, { type: 'join' }, { type: 'join' });
  assert.equal(out[2], 'no link is asked of yet');
  assert.deepEqual(out[4], [{ run: 'join', link: 'l', person: 'Dee', failed: 'unjoined' }]);
  assert.equal(out[5], 'a join is under way, and is answered first');
  assert.equal(s.at.join, 'joining');
  const [device] = run(FIRST, { type: 'typed', field: 'link', value: 'l' }, { type: 'checked', n: 1, for: ['Trip', 'Sam'] });
  assert.deepEqual(JOIN.step(device, { type: 'join' }).effects, [{ run: 'join', link: 'l', person: 'Sam', failed: 'unjoined' }]);
});

test('a refusal is said in place of the last, the link checked kept to press again', () => {
  const [s] = run(FIRST, { type: 'typed', field: 'link', value: 'l' }, { type: 'checked', n: 1, for: ['Trip', null] }, { type: 'join' }, { type: 'unjoined', why: 'Sam’s computer did not answer' });
  assert.deepEqual([s.at.join, s.why, s.for], ['refused', 'Sam’s computer did not answer', ['Trip', null]]);
  assert.equal(JOIN.step(s, { type: 'join' }).state.at.join, 'joining');
  assert.equal(JOIN.step(s, { type: 'opened' }).refused, 'joining takes no opened');
});
