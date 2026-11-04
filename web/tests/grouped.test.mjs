// SPDX-License-Identifier: AGPL-3.0-or-later
import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./pages/layout.mjs', import.meta.url);
const { address, exposes, GroupPlaces } = await import('../../crates/svrngs-node/src/home/page/grouped.js?v=__VERSION__');
const { PLACES } = await import('../../crates/svrngs-node/src/home/page/group-read.js?v=__VERSION__');
const join = await import('../../crates/svrngs-node/src/home/page/join.js?v=__VERSION__');
const make = await import('../../crates/svrngs-node/src/home/page/make.js?v=__VERSION__');

test('B places every original group capability exactly once', () => {
  const placed = [];
  GroupPlaces({ draw: (names) => { placed.push(...names); return null; } });
  assert.deepEqual(placed.toSorted(), [...PLACES].sort());
});

test('B destinations retain stable ids and refuse unknown views without exposing controls', () => {
  assert.equal(address('groups', 'a&b'), '/?ui=b&group=a%26b');
  assert.equal(exposes({ q: { view: 'computer' } }, 'acts'), true);
  for (const view of ['constructor', '__proto__', 'missing']) {
    assert.equal(exposes({ q: { view } }, 'acts'), false);
  }
  assert.equal(exposes({ q: { group: 'a', view: 'computer' } }, 'acts'), false);
});

test('making and joining preserve B and its destination through the reread', () => {
  const state = { at: { said: 'none' }, q: { ui: 'b', view: 'join' } };
  for (const flow of [join, make]) {
    const next = flow.take(state, { q: { joined: 'House' } }).state;
    assert.deepEqual(next.q, { ui: 'b', view: 'join', joined: 'House' });
  }
});
