// The phone's enter (page-open (b); web/page/enter.js), with no browser: PAGE
// composed from the flows, each `looked` routed as web/app.js `opening`
// routes it (:667-706), the tab's lock held by one tab, and the read's
// refusals said by whose computer gave them, an older answer refused by
// name. The edge's io is not run here: each step's effects are read as data.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { compose } = await import('../../page/page.js?v=__VERSION__');
const FLOWS = ['enter', 'computer', 'join', 'ask', 'index', 'door', 'shelf', 'visit', 'among', 'acts', 'uses'];
const parts = compose(Object.fromEntries(await Promise.all(FLOWS.map(async (f) => [f, await import(`../../page/${f}.js?v=__VERSION__`)]))));

const house = { id: 'g1', name: 'house', person: 'bo', token: 'TOKEN', addresses: ['notes@ada'] };
const lake = { id: 'g2', name: 'lake', person: 'bo', token: 'LAKE', addresses: ['notes@ada'], maker: 'cy' };
const first = parts.first({ fine: false });
// What `looked` answers for `fragment`: each effect as [run, its message's type, its words].
const looked = (fragment, more = {}) => parts.step(first, { type: 'looked', fragment, kept: [house], linked: true, ...more }).effects.map((f) => [f.run, f.m?.type, f.m?.said ?? f.m?.token ?? f.id]);

test('each fragment is routed as web/app.js opening routes it', () => {
  assert.deepEqual(looked('TOKEN'), [['send', 'offer', 'TOKEN'], ['lock', undefined, 'g1']], 'a held link: its line offered, its group held');
  assert.deepEqual(looked(''), [['lock', undefined, 'g1']], 'no link: the group last opened');
  assert.deepEqual(looked('NEW'), [['send', 'offer', 'NEW'], ['send', 'join', 'NEW']], 'a new link joins');
  assert.deepEqual(looked('', { kept: [] }), [['send', 'offer', ''], ['send', 'failed', 'Open this page from your group’s link.']]);
  assert.deepEqual(looked('TOKEN&x=1&y=2'), [['send', 'failed', 'This address asks for something this page can’t read: x, y. Open your group’s link again.']]);
  assert.deepEqual(looked('TOKEN', { linked: false }), [['send', 'failed', 'This link isn’t one svrngs can read. Ask whoever sent it for a new one.']]);
  assert.deepEqual(looked('', { missing: true }), [['send', 'failed', 'There’s no page at this address. Open your group’s link to reach your group.']]);
  assert.deepEqual(looked('visit=V'), [['send', 'visit', 'V']]);
  assert.deepEqual(looked('', { cold: true }), [['send', 'cold', undefined]]);
  assert.deepEqual(looked('open=notes@ada&at=/', { kept: [house, lake] }), [['send', 'among', undefined]], 'lent in two groups: each offered');
  assert.deepEqual(looked('open=films@ada', { kept: [] }), [['send', 'failed', 'This browser hasn’t joined your group yet, so it can’t open this. Open your group’s link from the chat here first, then this again.']]);
});

// `s` stepped through each message in turn, each step taken.
const through = (s, ...ms) => ms.reduce((at, m) => {
  const r = parts.step(at, m);
  assert.equal(r.refused, undefined, `${m.type}: ${r.refused}`);
  return r.state;
}, s);

test('one tab holds the group: another says so, and takes it here when asked', () => {
  const held = parts.step(first, { type: 'hold', group: lake });
  assert.deepEqual(held.effects, [{ run: 'lock', id: 'g2', failed: 'failed' }]);
  const there = through(held.state, { type: 'elsewhere' });
  assert.deepEqual(parts.heading(there), ['You have lake (cy’s) open in another tab', 'good']);
  assert.equal(parts.step(there, { type: 'held' }).refused, 'no lock was asked for');
  const taking = parts.step(there, { type: 'take' });
  assert.deepEqual(taking.effects, [{ run: 'lock', id: 'g2', steal: true, failed: 'failed' }]);
  const holding = through(taking.state, { type: 'held' });
  assert.deepEqual(holding.at, { ...first.at, tab: 'holding' });
  assert.equal(parts.step(holding, { type: 'take' }).refused, 'the group is not open in another tab');
  assert.equal(parts.step(first, { type: 'reread' }).refused, 'the list is unasked, in a tab unheld');
});

const body = { version: 3, lent: [], unanswered: [{ person: 'cy', person_key: 'kc' }], unheard: [], refused: [], older: [], removals: [], visiting: [], people: [], shelf: [], changed: [] };
const ada = { key: 'ka', person: 'ada', relay: null };

test('the read: its refusals said, an older answer refused by name, and what it read', () => {
  const opened = through(first, { type: 'hold', group: house }, { type: 'held' });
  const reading = parts.step(opened, { type: 'opened', group: house });
  assert.deepEqual(reading.effects, [{ run: 'read', group: house, n: 1, failed: 'unread' }]);
  assert.deepEqual(parts.heading(reading.state), ['Reaching house…']);
  const said = (m) => parts.step(reading.state, { type: 'read', n: 1, body, lent: [], ...m }).effects.map((f) => f.m?.said ?? f.run);
  assert.deepEqual(said({ ok: false, refusal: 'removed' }), ['release', 'You are no longer in house.']);
  assert.deepEqual(said({ ok: false, via: ada }), ['release', 'ada’s computer said no when you asked what house lends.']);
  assert.deepEqual(said({ ok: true, older: true }), ['release', 'The computer that sent your link runs an older svrngs, so this page cannot show what house lends. Installing svrngs again there updates it.']);
  const again = parts.step(through(reading.state, { type: 'read', n: 1, ok: true, via: ada, body, lent: [] }), { type: 'reread' });
  assert.deepEqual(parts.heading(again.state), ['You’re in house, as bo, through ada’s computer', 'good'], 'the last read stands while it is read again');
  assert.equal(parts.response(again.state), 'cy’s computer did not answer, so what they lend is not listed.');
  assert.equal(parts.step(again.state, { type: 'read', n: 1, ok: true, body, lent: [] }).refused, 'list 1 answered after list 2 was asked');
});

test('back on screen the list is read again, and a read again that fails keeps the list, saying why', () => {
  const read = through(first, { type: 'hold', group: house }, { type: 'held' }, { type: 'opened', group: house }, { type: 'read', n: 1, ok: true, body, lent: [] });
  assert.equal(parts.step(first, { type: 'shown' }).refused, 'the list is unasked, in a tab unheld', 'nothing to read again before the first');
  const again = parts.step(read, { type: 'shown' });
  assert.deepEqual(again.effects, [{ run: 'read', group: read.group, n: 2, failed: 'unread' }]);
  const failed = parts.step(again.state, { type: 'unread', n: 2, said: 'ada’s computer did not answer. Try again later.' });
  assert.deepEqual(failed.effects, [], 'the tab keeps the group, and nothing is said over the heading');
  assert.deepEqual(parts.heading(failed.state), ['You’re in house, as bo', 'good']);
  assert.equal(parts.response(failed.state), 'What house lends cannot be read again just now: ada’s computer did not answer. Try again later.\ncy’s computer did not answer, so what they lend is not listed.');
  const back = through(failed.state, { type: 'shown' }, { type: 'read', n: 3, ok: true, body: { ...body, unanswered: [] }, lent: [{ key: 'ka', name: 'notes', person: 'ada', label: 'l' }] });
  assert.equal(parts.response(back), '', 'read again: the line goes, and what is lent now is in the list');
  assert.deepEqual(back.group.addresses, ['l', 'notes@ada']);
});

// Picked, the offer and its line go in the click's own step, as v1's click
// emptied #door and #said before `hold` (web/app.js@19b63b2b:586): the browser
// driver reads the next line once `hold`'s lock answers (cold.mjs `landed`).
test('a group picked from the offer leaves it in the same step', () => {
  const offered = through(first, { type: 'among', groups: [house, lake], wants: { open: 'notes@ada' } });
  assert.equal(parts.response(offered), 'Open it in the one you mean.');
  const r = parts.step(offered, { type: 'pick', group: house, wants: { open: 'notes@ada' } });
  assert.equal(r.refused, undefined, r.refused);
  assert.equal(parts.response(r.state), '', 'no line left to read as the answer');
  assert.deepEqual(r.effects.map((f) => [f.run, f.m?.type, f.m?.group?.id]), [['send', 'hold', 'g1']]);
  assert.match(parts.step(r.state, { type: 'pick', group: lake }).refused, /nothing is offered/);
});
