// SPDX-License-Identifier: AGPL-3.0-or-later
import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./pages/layout.mjs', import.meta.url);
const { address, exposes, GroupPlaces } =
  await import('../../crates/svrngs-node/src/home/page/grouped.js?v=__VERSION__');
const { PLACES } =
  await import('../../crates/svrngs-node/src/home/page/group-read.js?v=__VERSION__');
const join = await import('../../crates/svrngs-node/src/home/page/join.js?v=__VERSION__');
const make = await import('../../crates/svrngs-node/src/home/page/make.js?v=__VERSION__');
const { first } =
  await import('../../crates/svrngs-node/src/home/page/home-reading.js?v=__VERSION__');

test('B places every original group capability exactly once', () => {
  const placed = [];
  GroupPlaces({
    draw: (names) => {
      placed.push(...names);
      return null;
    },
  });
  assert.deepEqual(placed.toSorted(), [...PLACES].sort());
});

test('B destinations retain stable ids and refuse unknown views without exposing controls', () => {
  assert.equal(address('groups', 'a&b'), '/?group=a%26b');
  assert.equal(address(), '/');
  assert.equal(address('computer'), '/?view=computer');
  assert.equal(exposes({ grouped: true, q: { view: 'computer' } }, 'acts'), true);
  assert.equal(exposes({ grouped: true, q: {} }, 'connection'), false);
  assert.equal(exposes({ grouped: true, q: { view: 'computer' } }, 'connection'), true);
  assert.equal(exposes({ grouped: false, q: {} }, 'connection'), true, 'A has it all on one page');
  for (const view of ['constructor', '__proto__', 'missing']) {
    assert.equal(exposes({ grouped: true, q: { view } }, 'acts'), false);
  }
  assert.equal(exposes({ grouped: true, q: { group: 'a', view: 'computer' } }, 'acts'), false);
});

test('making and joining land where the new group is, leaving the form behind', () => {
  const state = { at: { said: 'none' }, grouped: true, q: { view: 'join' } };
  for (const flow of [join, make]) {
    const next = flow.take(state, { q: { joined: 'House' } }).state;
    assert.deepEqual(next.q, { joined: 'House' });
    assert.equal(next.grouped, true, 'the layout is held apart from the address');
    assert.deepEqual(
      flow.take({ ...state, grouped: false, q: { joined: 'Old' } }, { q: { made: 'New' } }).state.q,
      { made: 'New' },
      'A retains its original notice replacement',
    );
  }
});

test('the layout is decided once, by the address: only `ui=a` is A', () => {
  const at = (search) => first({ pathname: '/', search, origin: 'http://x', hash: '' }).grouped;
  assert.equal(at(''), true);
  assert.equal(at('?ui=b'), true, 'an old bookmark of B opens B');
  assert.equal(at('?view=computer&moved'), true);
  assert.equal(at('?ui=a'), false);
});
