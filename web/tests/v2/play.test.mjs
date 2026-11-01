// Play on the home's v2 (page-home-rooms (b);
// crates/svrngs-node/src/home/v2/play.js), with no browser: PLAY's steps,
// each effect read as data, every turn PLAY's whichever list it was pressed
// in, and `plays`, the part acts.js draws under "What acts as you", written
// out as rooms.test.mjs writes a part, each Play on by its words.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { PLAY, plays } = await import('../../../crates/svrngs-node/src/home/v2/play.js?v=__VERSION__');

const out = (v) => {
  if (v == null || typeof v === 'boolean') return '';
  if (Array.isArray(v)) return v.map(out).join('');
  if (typeof v !== 'object') return String(v);
  if (v.type.name === 'Turn') return `[${v.props.words(v.props.l)}]`;
  return `<${v.type}>${out(v.props.children)}</${v.type}>`;
};
const { step } = PLAY;
const FIRST = { at: { play: 'reading' }, asked: { play: 1 }, round: 1, said: {}, copied: {} };
const played = { person: 'cy', key: 'k2', name: 'played', port: 41000, said: 'Connect to 127.0.0.1:41000.', open: { is: 'at' } };
const l = { key: 'k2', name: 'played' };
const read = step(FIRST, { type: 'answered', n: 1, plays: [] }).state;

test('read as it loads, then once a round of the groups’ reads, however many cells ask', () => {
  assert.deepEqual([read.at.play, read.plays], ['read', []]);
  assert.equal(step(read, { type: 'read', round: 1 }).refused, 'round 1 is read already');
  const again = step(read, { type: 'read', round: 2 });
  assert.deepEqual([again.state.at.play, again.effects], ['reading', [{ run: 'read', n: 2, failed: 'unread' }]]);
  assert.equal(step(again.state, { type: 'answered', n: 1, plays: [] }).refused, 'play 1 answered after play 2 was asked');
  assert.equal(step(read, { type: 'yes' }).refused, 'Play takes no yes');
});

test('a turn is PLAY’s: its answer is what has Play on, what it copied kept for its lend, a second turn and a read refused while it runs', () => {
  const turning = step(read, { type: 'turn', l, off: false });
  assert.deepEqual(turning.effects, [{ run: 'turn', n: 2, l, off: false, failed: 'refused' }]);
  assert.equal(turning.state.at.play, 'turning');
  for (const m of [{ type: 'turn', l, off: false }, { type: 'read', round: 2 }]) assert.equal(step(turning.state, m).refused, 'a turn is under way, and its answer is what has Play on');
  const on = step(turning.state, { type: 'answered', n: 2, plays: [played], copied: ' Copied.' }).state;
  assert.deepEqual([on.at.play, on.plays, on.copied], ['read', [played], { 'k2:played': ' Copied.' }]);
  const again = step(step(on, { type: 'read', round: 2 }).state, { type: 'answered', n: 3, plays: [played] }).state;
  assert.deepEqual(again.copied, { 'k2:played': ' Copied.' }, 'a read keeps what a turn copied');
  const off = step(step(again, { type: 'turn', l, off: true }).state, { type: 'answered', n: 4, plays: [], copied: '' }).state;
  assert.deepEqual([off.plays, off.copied], [[], { 'k2:played': '' }]);
});

test('a refused turn goes back to where it was, each refusal kept for that lend', () => {
  const turning = step(read, { type: 'turn', l, off: false }).state;
  const refused = step(turning, { type: 'refused', n: 2, why: 'played is not lent here now' }).state;
  assert.deepEqual([refused.at.play, refused.said], ['read', { 'k2:played': ' played is not lent here now' }]);
  const twice = step(step(refused, { type: 'turn', l, off: false }).state, { type: 'refused', n: 3, why: 'still not' }).state;
  assert.equal(twice.said['k2:played'], ' played is not lent here now still not');
});

test('"What acts as you" lists each Play on with its Stop, or why none can be read; none when nothing is on', () => {
  assert.deepEqual(plays(FIRST), []);
  assert.deepEqual(plays(read), []);
  const on = { ...read, plays: [played] };
  assert.equal(out(plays(on)), '[Open on this computer: played (cy’s), while Play is on. ]');
  const why = 'This computer cannot read what has Play on (…).';
  assert.equal(out(plays(step(FIRST, { type: 'unread', n: 1, why }).state)), `<p>${why}</p>`);
});
