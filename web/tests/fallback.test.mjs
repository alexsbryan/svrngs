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
    fallback:
      'This computer is reachable through the public way for now, so a phone can open House too. It goes back to the place House usually meets once the public way stops answering.',
  };
  for (const [why, line] of Object.entries(said)) {
    assert.ok(section(why).includes(line), section(why));
    assert.ok(!/relay|node|key/i.test(section(why)), 'none of the words §1 keeps from people');
  }
  assert.ok(!section(null).includes('backup'), section(null));
});

// The button of "Check the connection now" (KEEP_GOING_AT_HOME 7): a plain
// form to home.rs `connection`, which sends the page back with what
// happened in its address; each said in a person's words, once, with no
// word §1 keeps from people. Planted: `SAID` without `moved`, which throws.
test('the page says in a person’s words what pressing the button did', async () => {
  const { first } =
    await import('../../crates/svrngs-node/src/home/page/home-reading.js?v=__VERSION__');
  const { saidOnce: said, Connection } =
    await import('../../crates/svrngs-node/src/home/page/home-page.js?v=__VERSION__');
  const at = (search) => said(first({ pathname: '/', search, origin: 'http://here', hash: '' }));
  assert.equal(at('?stayed'), 'The public way answers, so nothing changed.');
  assert.equal(
    at('?moved'),
    'The public way did not answer, so this computer moved to where your group usually meets.',
  );
  assert.equal(
    at('?nowhere'),
    'The public way did not answer, and neither did anywhere else your groups meet.',
  );
  for (const search of ['?stayed', '?moved', '?nowhere'])
    assert.ok(!/relay|node|key/i.test(at(search)), at(search));
  const form = Connection();
  assert.deepEqual([form.props.method, form.props.action], ['post', '/connection']);
  assert.equal(words(form), 'Check the connection now');
});
