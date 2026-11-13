// What the browser step's sends tests held of the home page, with no
// browser (docs/CHECKS.md harness 5): the index row of an app that reads a
// lend it may send to, drawn whole by twin.mjs from /v1/lends answered in
// surface.rs `opening`'s shape (surface.rs:526-529), its fold's ticks
// pressed as a person presses them, and what its Open posts read off the
// form (home/page/reach.js `Reach`). That the post is the yes to the send,
// and the send then reaches the service once, body whole, is the node's:
// crates/svrngs-node/tests/carried.rs
// the_home_page_s_open_with_its_send_ticked_is_the_yes_to_the_send.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { open, words } from './twin.mjs';

const AT = 'http://svrngs.localhost:4000';
const [ADA, SAM] = ['a1', 'b2'].map((k) => k.repeat(32));
const LLAMA = 'llama-server0b2';
// Probe, carried in house on ada's computer, its sends.txt naming
// llama-server: /v1/lends lists it with what its open names (surface.rs
// `opening`), Sam's llama-server, whose `sends:` row is POST /v1/embeddings.
const probe = (sending) => ({
  key: ADA,
  name: 'Probe',
  kind: 'Probe',
  person: 'ada',
  group: 'house',
  origin: 'http://probe0lbl.localhost:4000/',
  limit: 3,
  using: 0,
  groups: ['house'],
  reads: ['llama-server'],
  version: 9,
  reading: [
    {
      label: LLAMA,
      person: 'Sam',
      name: 'llama-server',
      kind: 'llama-server',
      ticked: true,
      new: false,
      sends: ['POST /v1/embeddings'],
      sending,
    },
  ],
});
const answers = (sending) => ({
  'GET /v1/groups': {
    key: ADA,
    groups: [
      {
        id: 'house',
        name: null,
        person: 'ada',
        removed: false,
        removals: [],
        left: false,
        keeps: true,
        made: null,
        folders: [],
        waiting: [],
        visiting: [],
        fallback: null,
      },
    ],
  },
  'GET /v1/rail/people?namespace=house': {
    people: [
      { person: 'ada', person_key: ADA, key: ADA, device: 'computer', joined: 'started' },
      { person: 'Sam', person_key: SAM, key: SAM, device: 'computer', joined: { at: 1 } },
    ],
  },
  'GET /v1/lends?namespace=house': {
    lent: [probe(sending)],
    unanswered: [],
    unheard: [],
    refused: [],
    older: [],
    shelf: [],
    changed: [],
  },
  'GET /publication?group=house': { published: false },
  'GET /lent': { lent: [], signing: [], readers: {} },
  'GET /reaching': {},
  'GET /v1/room': [],
  'GET /v1/play': [],
  'GET /update': { state: 'current' },
});

// The row's ticks, the read's then the send's, and what its Open posts, as
// web/tests/sends.mjs `ticks` read them in a browser.
const ticks = (page) => {
  const boxes = page.all('#home table.index details input');
  const send = page.all('#home table.index details label').find((l) => /Sends to/.test(words(l)));
  const posted = (name) => page.all(`#home table.index input[name=${name}]`)[0].value;
  return {
    line: send ? words(send) : null,
    checked: boxes.map((b) => b.checked),
    disabled: boxes.map((b) => 'disabled' in b.attributes),
    reads: posted('reads'),
    sends: posted('sends'),
  };
};

// browser::sends::the_send_tick_is_the_yes_to_a_send, both engines, and the
// home page's half of browser::sends::a_carried_taste_sends_and_draws_what_comes_back:
// the open draws the send's tick under its read's, unticked, posting no
// send; ticked, the Open posts it; with its read unticked the send is
// neither tickable nor posted; the Open posts the form to /reaching in a
// tab of its own; and the last yes, `sending`, ticks it when drawn again.
test('the send’s tick is drawn under its read’s, posted only while its read is, and the last yes ticks it', async () => {
  const page = await open(`${AT}/?ui=a`, answers(false));
  assert.deepEqual(page.unanswered, []);
  assert.deepEqual(ticks(page), {
    line: 'Sends to Sam’s llama-server at /v1/embeddings. Sam can see what it sends.',
    checked: [true, false],
    disabled: [false, false],
    reads: LLAMA,
    sends: '',
  });
  await page.tick('#home table.index details input', 1);
  assert.equal(ticks(page).sends, LLAMA, 'ticked, the Open posts the send');
  await page.tick('#home table.index details input', 0);
  const unread = ticks(page);
  assert.deepEqual([unread.disabled, unread.sends], [[false, true], ''], 'its read unticked');
  await page.tick('#home table.index details input', 0);
  assert.equal(ticks(page).sends, LLAMA, 'its read ticked again, the send posted again');
  await page.press('#home table.index form button', 'Open Probe');
  const form = page.all('#home table.index form')[0];
  assert.deepEqual(page.went, [['submit', '/reaching']]);
  assert.equal(form.attributes.target, '_blank', 'the Open goes on to the app in a tab of its own');
  const again = await open(`${AT}/?ui=a`, answers(true));
  const kept = ticks(again);
  assert.deepEqual([kept.checked, kept.sends], [[true, true], LLAMA], 'the last yes ticks it');
});
