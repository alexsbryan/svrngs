// What is open to this WiFi on the home's v2 (page-home-rooms (a);
// crates/svrngs-node/src/home/v2/wifi.js), with no browser: ROOM's steps,
// each effect read as data, and `rooms`, the part acts.js draws under
// "What acts as you", written out as parts.test.mjs writes a part, each
// lend's line by its words.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { ROOM, rooms } = await import('../../../crates/svrngs-node/src/home/v2/wifi.js?v=__VERSION__');

const out = (v) => {
  if (v == null || typeof v === 'boolean') return '';
  if (Array.isArray(v)) return v.map(out).join('');
  if (typeof v !== 'object') return String(v);
  if (v.type.name === 'Line') return `[${v.props.words}]`;
  const id = v.props.id ? ` id="${v.props.id}"` : '';
  return `<${v.type}${id}>${out(v.props.children)}</${v.type}>`;
};
const { step } = ROOM;
const FIRST = { at: { room: 'reading' }, asked: { room: 1 }, round: 1, closed: [], said: {} };
const shelf = { person: 'ada', key: 'k1', name: 'shelf', said: 'Open to this WiFi: type http://192.168.1.2:40000/ …' };
const read = step(FIRST, { type: 'answered', n: 1, room: [shelf] }).state;
const LINE = 'Anyone on this WiFi can open these as you while svrngs runs: your TV, a phone, another computer. Each stays on the WiFi where you turned it on.';

test('read as it loads, then once a round of the groups’ reads, however many groups ask', () => {
  assert.deepEqual([read.at.room, read.room], ['read', [shelf]]);
  assert.equal(step(read, { type: 'read', round: 1 }).refused, 'round 1 is read already');
  const again = step(read, { type: 'read', round: 2 });
  assert.deepEqual([again.state.at.room, again.effects], ['reading', [{ run: 'read', n: 2, failed: 'unread' }]]);
  assert.equal(step(again.state, { type: 'read', round: 2 }).refused, 'round 2 is read already');
  assert.equal(step(again.state, { type: 'answered', n: 1, room: [] }).refused, 'room 1 answered after room 2 was asked');
  assert.equal(step(read, { type: 'yes' }).refused, 'the room takes no yes');
});

test('a turn is ROOM’s: its answer is what is open, a second turn and a read refused while it runs', () => {
  const turning = step(read, { type: 'turn', l: shelf, off: true });
  assert.deepEqual(turning.effects, [{ run: 'turn', n: 2, l: shelf, off: true, failed: 'refused' }]);
  assert.equal(turning.state.at.room, 'turning');
  for (const m of [{ type: 'turn', l: shelf, off: true }, { type: 'read', round: 2 }]) assert.equal(step(turning.state, m).refused, 'a turn is under way, and its answer is what is open');
  const closed = step(turning.state, { type: 'answered', n: 2, room: [], turned: { l: shelf, off: true } }).state;
  assert.deepEqual([closed.at.room, closed.room, closed.closed], ['read', [], [shelf]]);
  const opened = step(step(closed, { type: 'turn', l: shelf, off: false }).state, { type: 'answered', n: 3, room: [shelf], turned: { l: shelf, off: false } }).state;
  assert.deepEqual([opened.room, opened.closed], [[shelf], []]);
});

test('a refused turn goes back to where it was, why kept for that lend', () => {
  const turning = step(read, { type: 'turn', l: shelf, off: true }).state;
  const refused = step(turning, { type: 'refused', n: 2, why: 'shelf is not lent here now' }).state;
  assert.deepEqual([refused.at.room, refused.said], ['read', { 'k1:shelf': ' shelf is not lent here now' }]);
});

test('"Open to this WiFi" lists what is open and what a press here closed; none when nothing is', () => {
  assert.deepEqual(rooms(FIRST), []);
  assert.deepEqual(rooms(step(FIRST, { type: 'answered', n: 1, room: [] }).state), []);
  assert.equal(out(rooms(read)), `<p>${LINE}</p><ul id="room">[ada’s shelf:]</ul>`);
  const closed = { ...read, room: [], closed: [shelf] };
  assert.equal(out(rooms(closed)), `<p>${LINE}</p><ul id="room">[ada’s shelf:]</ul>`);
  const why = 'This computer cannot read what you opened to this WiFi (…). Open a lend to this WiFi again, and it starts over from that one.';
  const unread = step(FIRST, { type: 'unread', n: 1, why }).state;
  assert.equal(out(rooms(unread)), `<p>${why}</p><ul id="room"></ul>`);
});
