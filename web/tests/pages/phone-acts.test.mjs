// "What acts as you" on the group's page, on a phone (web/page/
// what-acts-as-you.js, opens-as-you.js), drawn by Preact from the state
// the phone's PAGE leaves once its group is read, with no browser: the
// claims of browser::acts::the_group_page_says_what_acts_as_you_and_its_
// stop_holds_at_the_next_pass but its pass, which web/tests/hand.test.mjs
// holds ('an open is its yes', 'a stop names none').

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { open, words } from './twin.mjs';

const HOME = new URL('../../../crates/svrngs-node/src/home/', import.meta.url);
const q = await import(new URL('question.js?v=__VERSION__', HOME).href);
const { html, render } = await import(new URL('ui/view.js?v=__VERSION__', HOME).href);
const { WhatActsAsYou } = await import('../../page/what-acts-as-you.js?v=__VERSION__');
const FLOWS = ['enter', 'computer', 'join', 'ask', 'index', 'door', 'shelf', 'visit', 'among'];
const parts = (await import('../../page/page.js?v=__VERSION__')).compose(
  Object.fromEntries(
    await Promise.all(
      [...FLOWS, 'acts', 'uses'].map(async (f) => [
        f,
        await import(`../../page/${f}.js?v=__VERSION__`),
      ]),
    ),
  ),
);

const [ADA, BO, CY] = ['a1', 'b2', 'c3'].map((k) => k.repeat(32));
const row = (person, key, device) => ({
  key,
  person,
  person_key: key,
  device,
  joined: { at: 1_791_000_000 },
});
// A lend as the phone's read gives it (enter.js `Lent`), its label the wasm's.
const lent = (name, more) => ({
  key: ADA,
  name,
  kind: 'k',
  person: 'ada',
  group: null,
  relay: null,
  label: `${name}0lbl`,
  limit: 3,
  using: 0,
  reads: [],
  version: 9,
  ...more,
});
const page = lent('page', { reads: ['k'] });
const shelf = lent('shelf', { key: CY, person: 'cy' });
// Bo's yes, as the open kept it with the group (hand.js `withYes`), today.
const AT = Math.floor(Date.now() / 1000) - 60;
const HOUSE = {
  id: 'house',
  name: 'house',
  person: 'bo',
  token: 'TOKEN',
  addresses: [],
  reads: { [page.label]: { said: 'yes', at: AT, lends: [shelf.label] } },
};
const body = {
  version: 3,
  lent: [],
  unanswered: [],
  unheard: [],
  refused: [],
  older: [],
  removals: [],
  visiting: [],
  people: [row('ada', ADA, 'computer'), row('cy', CY, 'computer'), row('bo', BO, 'browser')],
  shelf: [],
  changed: [],
};
const via = { key: ADA, person: 'ada', relay: null };

// Bo's phone holding house, read once through ada's computer, as
// enter.test.mjs steps it.
const read = () =>
  [
    { type: 'hold', group: HOUSE, wants: null },
    { type: 'held' },
    { type: 'opened', group: HOUSE },
    { type: 'read', n: 1, ok: true, via, body, lent: [page, shelf], me: BO },
  ].reduce(
    (s, m) => {
      const r = parts.step(s, m);
      assert.equal(r.refused, undefined, `${m.type}: ${r.refused}`);
      return r.state;
    },
    parts.first({ fine: false }),
  );

test('the group’s page on a phone says what acts as you, folded, and its Stop says when it holds', async () => {
  // The twin's document, for Preact to draw into.
  const home = await open('http://svrngs.localhost:4000/', {});
  const at = home.document.createElement('div');
  home.document.body.appendChild(at);
  const sent = [];
  const draw = (state) => {
    render(html`<${WhatActsAsYou} s=${state} send=${(m) => sent.push(m)} />`, at);
    return home.settle();
  };
  const state = read();
  await draw(state);
  const [details] = home.all('#acts details');
  assert.ok(details && !('open' in details.attributes), 'folded under its heading');
  const before = words(at);
  for (const part of [
    'What acts as you',
    'Opens friends’ things as you',
    'A Stop here holds from the next pass this phone signs for the app, within 12 hours.',
    `ada’s page (k) opens the k you chose, as you. Since ${q.day(AT)}.`,
    'Knows who you are',
    'What you open knows your name.',
    'Can decide for you',
    'This phone.',
  ])
    assert.ok(before.includes(part), `${part}: ${before}`);
  assert.ok(!before.includes('your other'), `only this phone decides for bo: ${before}`);

  await home.press('#acts button', 'Stop');
  assert.deepEqual(sent, [{ type: 'stop reads', label: page.label }]);
  const stopping = parts.step(state, sent[0]);
  assert.deepEqual(
    stopping.effects.map((e) => [e.run, e.label]),
    [['stopping', page.label]],
  );
  // What acts.js `stopping` answers once the group keeps the stop.
  const kept = { ...HOUSE, reads: { [page.label]: { said: 'stop', at: AT } } };
  const done = parts.step(stopping.state, { type: 'acted', group: kept, why: 'stopped' });
  await draw(done.state);
  const stopped = words(at);
  assert.ok(
    stopped.includes(
      'ada’s page (k) stops opening your friends’ k as you within 12 hours, when the pass it holds ends.',
    ),
    stopped,
  );
});
