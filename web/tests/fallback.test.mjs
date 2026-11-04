// The home page says when this computer meets a group at its fallback
// (relay fallbacks; svrn-docs rings/current/GROUP_RELAYS.md §7): a group
// section, called as a function with no DOM, says it under the group's
// name when /v1/groups says `fallback` (surface.rs `Held::fallback`), in
// a person's words, why — its own relays unanswered, or another group
// with none — and says nothing of it otherwise. Planted: the line drawn
// whatever `fallback` says, and the last assertion failed.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./pages/layout.mjs', import.meta.url);
const { Group } = await import('../../crates/svrngs-node/src/home/page/group.js?v=__VERSION__');

// The words a vnode tree draws, in order.
const words = (v) =>
  v == null || typeof v === 'boolean'
    ? ''
    : typeof v !== 'object'
      ? String(v)
      : Array.isArray(v)
        ? v.map(words).join('')
        : words(v.props?.children);

test('a group met at its fallback says why in a person’s words, and one met at its own says nothing', () => {
  const section = (fallback) =>
    words(Group({ g: { id: 'ab12', name: 'House', removed: null, fallback } }));
  const said = {
    unanswered:
      'This computer can’t reach the place House usually meets, so it’s reaching House a backup way for now.',
    unowned:
      'This computer is also in a group with no meeting place of its own, so it reaches every group, House too, the backup way.',
  };
  for (const [why, line] of Object.entries(said)) {
    assert.ok(section(why).includes(line), section(why));
    assert.ok(!/relay|node|key/i.test(section(why)), 'none of the words §1 keeps from people');
  }
  assert.ok(!section(null).includes('backup'), section(null));
});
