// What Events' acts come to (EVENTS.md §1, §3, §4), one rule a test. The laws
// are laws.test.mjs's; these are the answers.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { actsOf, answers, fold, hosted, init, ownActs, reduce, validate } from './events.js';

const act = (id, ts, by, op, corrects = null) => ({ id, ts, by, op, corrects });
const dinner = (id, by, more = {}) =>
  act(id, 1000, by, { kind: 'event', event: 'e1', what: 'dinner', when: 'Friday 7', where: 'the house', ...more });
const coming = (id, by, guests) => act(id, 1100, by, { kind: 'coming', event: 'e1', ...(guests && { guests }) });
const came = (id, by, guest) => act(id, 1200, by, { kind: 'came', event: 'e1', ...(guest && { guest }) });
const one = (acts) => fold(acts).events.e1;
const codes = (acts) => fold(acts).gaps.map((g) => g.code);

test('an event is its host\'s post, and who is coming is who said so, guests by name', () => {
  const e = one([dinner('a', 'sam'), coming('b', 'bo', ['Maya', ' Maya ', 'Jo']), coming('c', 'cy')]);
  assert.deepEqual(e, {
    host: 'sam', what: 'dinner', when: 'Friday 7', where: 'the house', at: 1000,
    coming: { bo: ['Jo', 'Maya'], cy: [] }, came: [], guests_came: {},
    via: { host: null, coming: {}, came: {}, guests_came: {} },
  });
});

test('who came is the came acts: a member\'s own, and the guests they brought, under them', () => {
  const e = one([dinner('a', 'sam'), coming('b', 'bo', ['Maya']), came('c', 'bo'), came('d', 'bo', 'Maya'), came('e', 'cy', 'Maya')]);
  assert.deepEqual(e.came, ['bo']);
  // Cy's "Maya" is Cy's guest, not Bo's: no one can say Bo's guest came but Bo.
  assert.deepEqual(e.guests_came, { bo: ['Maya'], cy: ['Maya'] });
});

test('changing your mind is a correction of your own act', () => {
  const e = one([dinner('a', 'sam'), coming('b', 'bo', ['Maya']), act('c', 1300, 'bo', { kind: 'coming', event: 'e1' }, 'b')]);
  assert.deepEqual(e.coming, { bo: [] });
  assert.deepEqual(one([dinner('a', 'sam'), coming('b', 'bo'), act('c', 1300, 'bo', null, 'b')]).coming, {});
});

test('a correction of someone else\'s answer counts for nothing, and says so', () => {
  const acts = [dinner('a', 'sam'), coming('b', 'bo'), act('c', 1300, 'cy', null, 'b'), act('d', 1300, 'cy', { kind: 'coming', event: 'e1', guests: ['X'] }, 'b')];
  assert.deepEqual(one(acts).coming, { bo: [] });
  assert.deepEqual(codes(acts), ['not_theirs', 'not_theirs']);
});

test('the record\'s voided flag is not this fold\'s: a voided act by another\'s correction still counts', () => {
  const log = {
    ops: [
      { id: 'a', ts_unix: 1000, person: 'sam', voided: false, payload: dinner('a', 'sam').op },
      { id: 'b', ts_unix: 1100, person: 'sam', on_behalf_of: 'bo', voided: true, payload: coming('b', 'bo').op },
      { id: 'c', ts_unix: 1200, person: 'cy', voided: false, corrects: 'b' },
      { id: 'd', ts_unix: 1200, person: 'cy', voided: false },
    ],
  };
  assert.deepEqual(actsOf(log).map((a) => [a.id, a.by]), [['a', 'sam'], ['b', 'bo'], ['c', 'cy']]);
  assert.deepEqual(fold(actsOf(log)).events.e1.coming, { bo: [] });
  assert.throws(() => actsOf({ error: 'no' }), /no list of acts/);
});

test('only the host changes the event, and no one else can post it again', () => {
  const moved = act('c', 1300, 'sam', { kind: 'event', event: 'e1', what: 'dinner', when: 'Friday 8', where: 'the yard' }, 'a');
  assert.equal(one([dinner('a', 'sam'), moved]).where, 'the yard');
  const taken = act('c', 1300, 'cy', { kind: 'event', event: 'e1', what: 'mine', when: '', where: '' }, 'a');
  assert.equal(one([dinner('a', 'sam'), taken]).host, 'sam');
  assert.deepEqual(codes([dinner('a', 'sam'), dinner('b', 'cy')]), ['not_the_host']);
  // The host withdrawing it leaves no event, and the answers to it said as answers to none.
  const gone = fold([dinner('a', 'sam'), coming('b', 'bo'), act('c', 1300, 'sam', null, 'a')]);
  assert.deepEqual([gone.events, gone.gaps.map((g) => g.code)], [{}, ['no_event']]);
});

test('nothing records "no": no act for it, and an answer that tries is not counted at all', () => {
  for (const op of [{ kind: 'not_coming', event: 'e1' }, { kind: 'declined', event: 'e1' }, { kind: 'coming', event: 'e1', coming: false }, { kind: 'came', event: 'e1', came: false }]) {
    assert.deepEqual(one([dinner('a', 'sam'), act('b', 1100, 'bo', op)]), one([dinner('a', 'sam')]), JSON.stringify(op));
  }
  assert.deepEqual(validate(act('b', 1, 'bo', { kind: 'coming', event: 'e1', coming: false })).map((g) => g.code), ['unread_field']);
});

test('a correction of an act this copy does not hold yet is said, and changes nothing', () => {
  assert.deepEqual(codes([dinner('a', 'sam'), act('c', 1300, 'bo', null, 'zz')]), ['changes_unheld']);
});

test('a correction ordered before its act, as one second\'s acts from two computers may be, still counts', () => {
  assert.deepEqual(one([dinner('a', 'sam'), act('c', 1100, 'bo', null, 'b'), coming('b', 'bo')]).coming, {});
});

test('a person\'s own live answers, newest first, are what changing their mind corrects', () => {
  const state = [dinner('a', 'sam'), coming('b', 'bo'), coming('c', 'bo', ['Maya']), coming('d', 'cy'), came('e', 'bo', 'Maya'), act('f', 1300, 'bo', null, 'b')].reduce(reduce, init());
  assert.deepEqual(ownActs(state, 'bo', 'e1', 'coming'), ['c']);
  assert.deepEqual(ownActs(state, 'bo', 'e1', 'came', 'Maya'), ['e']);
  assert.deepEqual(ownActs(state, 'bo', 'e1', 'came'), []);
  assert.deepEqual(ownActs(state, 'cy', 'e1', 'coming', null), ['d']);
});

test('the host\'s view joins each group where they host the event; an unread group is said as unread', () => {
  const house = fold([dinner('a', 'sam'), coming('b', 'bo', ['Maya'])]);
  const dweb = fold([dinner('x', 'sam'), coming('y', 'cy')]);
  const other = fold([dinner('z', 'di')]);
  const view = hosted('e1', [
    { group: 'house', answers: house, me: 'sam' },
    { group: 'dweb', answers: dweb, me: 'sam' },
    { group: 'club', answers: other, me: 'sam' },
    { group: 'far', unread: 'its record did not answer' },
  ]);
  assert.deepEqual(view.map((g) => [g.group, g.event ? Object.keys(g.event.coming) : g.unread]), [
    ['house', ['bo']], ['dweb', ['cy']], ['far', 'its record did not answer'],
  ]);
});

test('an answer to another group\'s event is an answer to none here: each record holds its own', () => {
  assert.deepEqual(fold([coming('b', 'bo')]).gaps.map((g) => g.code), ['no_event']);
});

test('which computer wrote it is kept: `via` is its person when it wrote for another, else null; the answers unchanged', () => {
  // ralph/FINDING_ON_BEHALF_OF.md's scene: Cy's computer writes Bo's answers in Bo's name.
  const op = (id, extra) => ({ id, ts_unix: 1000, voided: false, ...extra });
  const log = {
    ops: [
      op('a', { person: 'cy', on_behalf_of: 'sam', payload: dinner('a', 'sam').op }),
      op('b', { person: 'cy', on_behalf_of: 'bo', payload: coming('b', 'bo', ['Maya']).op }),
      op('c', { person: 'bo', on_behalf_of: 'bo', payload: came('c', 'bo').op }),
      op('d', { person: 'cy', on_behalf_of: 'bo', payload: came('d', 'bo', 'Maya').op }),
      op('e', { person: 'ama', payload: coming('e', 'ama').op }),
      // Ama's change of mind through Cy's computer: counted, as hers, and kept with its own `via`.
      op('f', { person: 'cy', on_behalf_of: 'ama', corrects: 'e' }),
    ],
  };
  const acts = actsOf(log);
  assert.deepEqual(acts.map((a) => [a.id, a.by, a.via]), [['a', 'sam', 'cy'], ['b', 'bo', 'cy'], ['c', 'bo', null], ['d', 'bo', 'cy'], ['e', 'ama', null], ['f', 'ama', 'cy']]);
  const state = acts.reduce(reduce, init());
  assert.equal(state.acts.f.via, 'cy');
  const e = answers(state).events.e1;
  const untold = fold(acts.map(({ via, ...a }) => a)).events.e1;
  assert.deepEqual([e.host, e.coming, e.came, e.guests_came], [untold.host, untold.coming, untold.came, untold.guests_came], 'via changes no answer');
  assert.deepEqual(e.via, { host: 'cy', coming: { bo: ['cy'] }, came: {}, guests_came: { bo: { Maya: ['cy'] } } });
});
