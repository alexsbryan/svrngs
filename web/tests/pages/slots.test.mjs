// How many each computer can have visiting at once, on the home page
// (home/page/visits.js `Slots` and its machine SLOTS), drawn whole by the
// twin (twin.mjs) from recorded answers in the shape the node writes
// (surface.rs `Held` for /v1/groups, home.rs `slots` for /slots).
//
// Plants, each watched failing in visits.js, then put back: the buttons
// drawn for a member who does not keep the group; the change posted with
// the number unchanged; the refusal's words left unsaid.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { open, words } from './twin.mjs';

const AT = 'http://svrngs.localhost:4000';
const ADA = 'a1'.repeat(32);
const ROW = { person: 'ada', person_key: ADA, key: ADA, device: 'computer', joined: 'started' };

// House as /v1/groups answers it, with `slots` each, and whether ada keeps it.
const group = (slots, keeps) => ({
  id: 'house',
  name: 'house',
  person: 'ada',
  removed: false,
  removals: [],
  left: false,
  keeps,
  made: null,
  folders: [],
  waiting: [],
  visiting: [],
  slots,
  fallback: null,
});

// Ada's computer, its groups read from `groups()` each time they are asked.
const ada = (groups, more = {}) => ({
  'GET /v1/groups': () => ({ groups: groups(), key: ADA }),
  'GET /v1/rail/people?namespace=house': { people: [ROW] },
  'GET /v1/lends?namespace=house': {
    lent: [],
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
  ...more,
});

const home = (answers) => open(`${AT}/?ui=a`, answers);
const row = (page) => page.text('#home p.slots');

test('a member who does not keep the group reads how many, and changes nothing', async () => {
  const page = await home(ada(() => [group(2, false)]));
  assert.deepEqual(row(page), ['Each computer can have 2 visiting at once']);
  assert.deepEqual(page.all('#home p.slots button'), []);
});

test('the keeper’s One more posts the next number, and the groups read again say it', async () => {
  let each = 2;
  const posted = [];
  const answers = ada(() => [group(each, true)], {
    'POST /slots': (body) => (posted.push(body), (each = body[1])),
  });
  const page = await home(answers);
  assert.deepEqual(row(page), ['Each computer can have 2 visiting at once One fewer One more']);
  await page.press('#home p.slots button', 'One more');
  assert.deepEqual(posted, [['house', 3]]);
  assert.deepEqual(row(page), ['Each computer can have 3 visiting at once One fewer One more']);
  assert.ok(page.lines().some((line) => line.startsWith('SLOTS: changed')));
});

test('a change refused says why, and the number stays', async () => {
  const most = 'each computer can have at most 8 visiting at once';
  const answers = ada(() => [group(8, true)], {
    'POST /slots': { status: 403, body: { error: most } },
  });
  const page = await home(answers);
  await page.press('#home p.slots button', 'One more');
  const [said] = row(page);
  assert.ok(said.startsWith('Each computer can have 8 visiting at once'), said);
  assert.ok(said.endsWith(most), said);
  assert.equal(words(page.all('#home p.slots')[0]).split(most).length - 1, 1, said);
});
