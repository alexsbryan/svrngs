// The host's controls for joining by a public address, on the home page
// (crates/svrngs-node/src/home/page/group-links.js `PublicationControls`,
// group-link.js LINK's publication), drawn whole over twin.mjs with no
// browser: the page's half of browser::publication::onboarding_host_controls_survive_restart,
// each of its five phases from the receipt /publication answers (src
// publication.rs `status`: id, state, phrase, address, until; 503 and
// `{ error }` when the resolver failed). The node's half, each phase's
// one resolver call and the invitation kept or ended across a restart of
// the process, is crates/svrngs-node/tests/home/publication.rs's
// publication_recovers_pending_open_closing_and_closed_across_process_restarts.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { open, words } from './twin.mjs';

const AT = 'http://svrngs.localhost:4000';
const ADA = 'a1'.repeat(32);
const ADDRESS = 'https://join.example/acorn-basil-cabin-daisy-eagle';
const UNAVAILABLE = {
  status: 503,
  body: {
    error:
      'joining is temporarily unavailable; retry in a minute from this computer. Your invitation is kept for retry.',
  },
};
const receipt = (state) => ({
  id: 'p1',
  state,
  phrase: 'acorn-basil-cabin-daisy-eagle',
  address: ADDRESS,
  until: 1_791_000_000,
});
const group = {
  id: 'house',
  name: 'house',
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
};
const lends = {
  lent: [],
  unanswered: [],
  unheard: [],
  refused: [],
  older: [],
  shelf: [],
  changed: [],
};

// Ada's computer, its receipt `held` (none: `{ published: false }`), each
// change posted answered by `change` from the body posted, which may keep
// a new receipt.
const ada = (held, change) => {
  const asked = [];
  const answers = {
    'GET /v1/groups': { groups: [group], key: ADA },
    'GET /v1/rail/people?namespace=house': {
      people: [{ person: 'ada', person_key: ADA, key: ADA, device: 'computer', joined: 'started' }],
    },
    'GET /v1/lends?namespace=house': lends,
    'GET /publication?group=house': () => held.now ?? { published: false },
    'POST /publication': (body) => (asked.push(['publish', body]), change(body)),
    'POST /publication/close': (body) => (asked.push(['close', body]), change(body)),
    'GET /lent': { lent: [], signing: [], readers: {} },
    'GET /reaching': {},
    'GET /v1/room': [],
    'GET /v1/play': [],
    'GET /update': { state: 'current' },
  };
  return { answers, asked };
};
const controls = (page) => words(page.all('.publication')[0]);
const pressed = (page, button) => page.press('.publication button', button);

test('the host’s controls: each phase drawn from its receipt, one post a press, and drawn again from the receipt kept', async () => {
  // pending: nothing published; Invite people here fails at the resolver.
  let held = { now: null };
  let at = ada(held, () => ((held.now = receipt('pending')), UNAVAILABLE));
  let page = await open(`${AT}/?ui=a`, at.answers);
  assert.ok(controls(page).includes('Invite people here'), controls(page));
  await pressed(page, 'Invite people here');
  assert.deepEqual(at.asked, [['publish', { group: 'house' }]]);
  let said = controls(page);
  assert.ok(said.includes('joining is temporarily unavailable'), said);
  assert.ok(!/invitation ended|Joining is closed/.test(said), said);
  assert.ok(
    page.all('.publication button').some((b) => words(b) === 'Retry joining'),
    said,
  );
  page = await open(`${AT}/?ui=a`, at.answers);
  assert.ok(controls(page).includes('Retry joining'), 'drawn again from the receipt kept');

  // open: the pending receipt retried by its id, and the address shown.
  held = { now: receipt('pending') };
  at = ada(held, () => (held.now = receipt('open')));
  page = await open(`${AT}/?ui=a`, at.answers);
  await pressed(page, 'Retry joining');
  assert.deepEqual(at.asked, [['publish', { group: 'house', id: 'p1' }]]);
  assert.equal(words(page.all('.publication code')[0]), ADDRESS);
  page = await open(`${AT}/?ui=a`, at.answers);
  assert.equal(words(page.all('.publication code')[0]), ADDRESS, 'drawn again');

  // closing: the address large, copied by its button, how long it lasts;
  // Close joining fails at the resolver.
  held = { now: receipt('open') };
  at = ada(held, () => ((held.now = receipt('closing')), UNAVAILABLE));
  page = await open(`${AT}/?ui=a`, at.answers);
  const [code] = page.all('.publication code');
  assert.equal(words(code), ADDRESS);
  assert.match(code.style.cssText, /font-size:\s*1\.4em/);
  assert.match(controls(page), /Ends .+This computer must remain reachable/);
  await pressed(page, 'Copy the address');
  assert.deepEqual(page.copied, [ADDRESS]);
  await pressed(page, 'Close joining');
  assert.deepEqual(at.asked, [['close', { group: 'house', id: 'p1' }]]);
  said = controls(page);
  assert.ok(said.includes('joining is temporarily unavailable'), said);
  assert.ok(!/invitation ended|Joining is closed/.test(said), said);
  assert.ok(
    page.all('.publication button').some((b) => words(b) === 'Retry closing'),
    said,
  );
  page = await open(`${AT}/?ui=a`, at.answers);
  assert.ok(controls(page).includes('Retry closing'), 'drawn again');

  // closed: the close retried, and joining said closed.
  held = { now: receipt('closing') };
  at = ada(held, () => (held.now = receipt('closed')));
  page = await open(`${AT}/?ui=a`, at.answers);
  await pressed(page, 'Retry closing');
  assert.deepEqual(at.asked, [['close', { group: 'house', id: 'p1' }]]);
  assert.ok(controls(page).includes('Joining is closed'), controls(page));
  page = await open(`${AT}/?ui=a`, at.answers);

  // inspect: closed, drawn as closed, nothing posted.
  assert.ok(controls(page).includes('Joining is closed'), controls(page));
  assert.deepEqual(at.asked, [['close', { group: 'house', id: 'p1' }]], 'no post on a look');
  assert.ok(
    page.asked.every(([, path]) => path.startsWith('/')),
    'the page asks its own origin alone',
  );
});
