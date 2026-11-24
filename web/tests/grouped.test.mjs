// SPDX-License-Identifier: AGPL-3.0-or-later
import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./pages/layout.mjs', import.meta.url);
const { address, exposes, GroupPlaces } =
  await import('../../crates/svrngs-node/src/home/page/grouped.js?v=__VERSION__');
const { PLACES } =
  await import('../../crates/svrngs-node/src/home/page/group-read.js?v=__VERSION__');
const { HOME, first } =
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

// HOME's `again` with what was done (join.js, make.js, leaving.js) lands
// where its address opens the page: All your groups, the form or the group
// left behind. One with none leaves the page where it is, as a removal on
// a group's page does.
test('making, joining and leaving land on All your groups, leaving the form or group behind', () => {
  const { step } = HOME({});
  const state = { at: { groups: 'read', said: 'none' }, grouped: true };
  const done = { joined: { joined: 'House' }, made: { made: 'New' }, left: { left: 'Trip' } };
  for (const [said, q] of Object.entries(done)) {
    for (const from of [{ view: 'join' }, { view: 'make' }, { group: 'trip' }]) {
      const next = step({ ...state, q: from }, { type: 'again', q }).state;
      assert.deepEqual([next.q, next.at.said], [q, said]);
      assert.equal(next.grouped, true, 'the layout is held apart from the address');
    }
    const a = step({ ...state, grouped: false, q: { joined: 'Old' } }, { type: 'again', q });
    assert.deepEqual(a.state.q, q, 'A retains its original notice replacement');
  }
  const stays = step({ ...state, q: { group: 'trip' } }, { type: 'again' }).state;
  assert.deepEqual([stays.q, stays.at.said], [{ group: 'trip' }, 'none']);
});

test('the layout is decided once, by the address: only `ui=a` is A', () => {
  const at = (search) => first({ pathname: '/', search, origin: 'http://x', hash: '' }).grouped;
  assert.equal(at(''), true);
  assert.equal(at('?ui=b'), true, 'an old bookmark of B opens B');
  assert.equal(at('?view=computer&moved'), true);
  assert.equal(at('?ui=a'), false);
});
