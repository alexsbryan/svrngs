// The home page’s own svrngs (page/update.js UPDATE): its check as the page
// opens, the three answers it keeps apart, the update and its wait for the
// restarted svrngs, and each refusal said with its why. No browser: each
// effect is read as data, as the pages’ machines are.

import assert from 'node:assert/strict';
import { test } from 'node:test';

const { UPDATE } = await import('../../crates/svrngs-node/src/home/page/update.js?v=__VERSION__');

const FIRST = { at: { update: 'checking' } };
const { step } = UPDATE;

test('the page checks as it opens; current says nothing and runs nothing', () => {
  const checked = step(FIRST, { type: 'checked', state: 'current' });
  assert.deepEqual([checked.state.at.update, checked.effects], ['current', []]);
});

test('available offers it; update asks, updated waits, back reloads', () => {
  const available = step(FIRST, { type: 'checked', state: 'available' });
  assert.equal(available.state.at.update, 'available');
  const updating = step(available.state, { type: 'update' });
  assert.deepEqual(updating.effects, [{ run: 'update', failed: 'refused' }]);
  const updated = step(updating.state, { type: 'updated' });
  assert.deepEqual(updated.effects, [{ run: 'wait', failed: 'stuck' }]);
  const back = step(updated.state, { type: 'back' });
  assert.deepEqual(back.effects, [{ run: 'reload', failed: 'stuck' }]);
});

test('a check that cannot be judged is not current, and says nothing until asked', () => {
  const unread = step(FIRST, { type: 'unread', why: 'the page did not answer' });
  assert.deepEqual(
    [unread.state.at.update, unread.state.why, unread.state.tried],
    ['unread', 'the page did not answer', undefined],
  );
  const retry = step(unread.state, { type: 'retry' });
  assert.deepEqual(
    [retry.state.at.update, retry.state.tried, retry.effects],
    ['checking', true, [{ run: 'check', failed: 'unread' }]],
  );
});

test('an unknown answer is unread, never current', () => {
  const checked = step(FIRST, { type: 'checked', state: 'unknown', why: 'no page kept' });
  assert.deepEqual([checked.state.at.update, checked.state.why], ['unread', 'no page kept']);
});

test('a refusal keeps its why, and can be tried again or left', () => {
  const refused = step(FIRST, { type: 'refused', why: 'it did not match the checksum' });
  assert.deepEqual(
    [refused.state.at.update, refused.state.why],
    ['refused', 'it did not match the checksum'],
  );
  const again = step(refused.state, { type: 'update' });
  assert.deepEqual(again.effects, [{ run: 'update', failed: 'refused' }]);
  const later = step(refused.state, { type: 'later' });
  assert.equal(later.state.at.update, 'dismissed');
});

test('a wait that hears nothing says how to open svrngs again', () => {
  const stuck = step(FIRST, { type: 'stuck', why: 'it did not answer' });
  assert.deepEqual([stuck.state.at.update, stuck.state.why], ['stuck', 'it did not answer']);
});

test('later dismisses an offered update', () => {
  const offered = step(FIRST, { type: 'checked', state: 'available' });
  assert.equal(step(offered.state, { type: 'later' }).state.at.update, 'dismissed');
});

test('an update that was already current reloads once, and the reload does not ask again', () => {
  const current = step(FIRST, { type: 'current' });
  assert.deepEqual(current.effects, [{ run: 'reload', failed: 'stuck' }]);
  const back = step(current.state, { type: 'back' });
  assert.deepEqual([back.state.at.update, back.effects], ['current', []]);
});
