// SPDX-License-Identifier: AGPL-3.0-or-later
import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./pages/layout.mjs', import.meta.url);
const { awaited } = await import('../../crates/svrngs-node/src/home/page/acting.js?v=__VERSION__');

// A group an older svrngs made draws only what is said of it (group.js
// `below`), so its GroupTold never mounts and never tells. Awaited, it held
// "What acts as you" unread, and "What you lend" behind it, forever.
test('"What acts as you" waits only for groups whose section tells it', () => {
  const group = (id, more) => ({ id, name: id, left: false, removed: false, ...more });
  const groups = [
    group('in'),
    group('older', { older: 'Pier was made by an older svrngs…' }),
    group('left', { left: true }),
    group('removed', { removed: true }),
    group('unread', { removed: null }),
  ];
  assert.deepEqual(
    awaited(groups).map((g) => g.id),
    ['in'],
  );
});
