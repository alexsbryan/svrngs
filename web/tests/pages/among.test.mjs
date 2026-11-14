// A lend's address or alias on the group's page, with no browser
// (web/page/among.js): what is said when nothing in the group goes by it,
// and a lend's address opened cold sent to its group's page once a minute
// (`cold`, over a stood-in location and sessionStorage). The words of
// browser::a_lends_alias_opens_it_from_the_groups_page and
// browser::a_lends_address_opens_cold_in_a_browser_that_joined, whose real
// navigations, sessionStorage across them and built 404.html stay theirs.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { takes, io } = await import('../../page/among.js?v=__VERSION__');

const house = { group: { id: 'house', name: 'house' } };

test('an alias or address nothing in the group goes by is said so, and of the other groups held', () => {
  const said = (m) => takes.others(house, { type: 'others', ...m }).state.missed;
  assert.equal(said({ open: 'app@ada', others: [] }), 'Nothing in house goes by app@ada now.');
  assert.equal(
    said({ open: 'nothing@ada', others: [{ name: 'trip' }] }),
    'Nothing in house goes by nothing@ada now, nor in trip when you last opened it.',
  );
  assert.equal(
    said({ open: 'x', label: true, others: [] }),
    'What that address opens is not lent in house now.',
  );
});

test('a lend’s address opened cold goes to its group’s page once a minute, then says why not', () => {
  const kept = new Map();
  const went = [];
  Object.assign(globalThis, {
    __trace: () => {},
    sessionStorage: {
      getItem: (k) => kept.get(k) ?? null,
      setItem: (k, v) => kept.set(k, v),
    },
    location: {
      href: 'https://app0lbl.group.example/at/a/deep?n=1',
      origin: 'https://app0lbl.group.example',
      protocol: 'https:',
      host: 'app0lbl.group.example',
      hostname: 'app0lbl.group.example',
      replace: (to) => went.push(to),
    },
  });
  assert.equal(io.cold().type, 'sent', 'sent on');
  assert.deepEqual(went, ['https://group.example/#open=app0lbl&at=%2Fat%2Fa%2Fdeep%3Fn%3D1&cold']);
  const again = io.cold();
  assert.equal(again.type, 'failed');
  assert.equal(again.said, 'This could not be opened. Open it again from your group’s page.');
  assert.equal(went.length, 1, 'never sent on twice in a minute');
  kept.set('svrngs cold', String(Date.now() - 61_000));
  io.cold();
  assert.equal(went.length, 2, 'a minute on, sent on again');
});

// web/page/how-it-reaches.js, drawn as a function of the state: `fine`, a
// pointer that is not coarse (enter.js `first`), is a computer's.
const { HowItReaches } = await import('../../page/how-it-reaches.js?v=__VERSION__');
const words = (v) =>
  v == null || typeof v === 'boolean'
    ? ''
    : Array.isArray(v)
      ? v.map(words).join(' ')
      : typeof v === 'object'
        ? words(v.props?.children)
        : String(v);

test('how this reaches you is shown at a computer once the group is read, a phone’s first look without it, in no word of the system’s', () => {
  const drawn = (state) => HowItReaches({ s: state });
  assert.equal(drawn({ fine: true, lent: [] }).props.hidden, false, 'a computer, read');
  assert.equal(drawn({ fine: false, lent: [] }).props.hidden, true, 'a phone');
  assert.equal(drawn({ fine: true }).props.hidden, true, 'not read yet');
  const told = words(drawn({ fine: true, lent: [] })).toLowerCase();
  assert.ok(told.includes('how this reaches you'), told);
  const said = new Set(told.split(/[^\p{L}\p{N}]+/u));
  const jargon = ['key', 'member', 'ring', 'sync', 'journal', 'node', 'relay', 'vpn', 'tunnel'];
  assert.deepEqual(
    jargon.filter((w) => said.has(w) || said.has(`${w}s`)),
    [],
  );
});
