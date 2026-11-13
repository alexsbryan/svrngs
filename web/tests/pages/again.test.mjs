// What the pages step (crates/svrngs/tests/pages/) held of the home page
// read again, with no browser: the page over its twin (twin.mjs, harness
// 5), each route answered from a recording in the shape its writer makes,
// named beside it. Each law names the pages test whose claim it holds.
// "No reload" is the twin's `went`: a reload, a form or a link followed is
// kept there, and each law asserts it empty. Layout A (`/?ui=a`), as those
// tests' drivers opened it (web/tests/page.mjs `longPage`).
//
// A long timer, a visit's end or a screen's next read, waits on `clock()`
// below, not on the real one: held until a law lets it go, so a law says
// what was drawn before and after, in order (docs/CHECKS.md seam C).
//
// Plants, each watched failing in a product module, then put back: named
// in the commit that brought these laws.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { open, words } from './twin.mjs';

const HOME = new URL('../../../crates/svrngs-node/src/home/', import.meta.url);
const q = await import(new URL('question.js?v=__VERSION__', HOME).href);

const AT = 'http://svrngs.localhost:4000';
const [ADA, BO, CY] = ['a1', 'b2', 'c3'].map((k) => k.repeat(32));
// A person as /v1/rail/people lists them (rail.rs `standing_json`).
const row = (person, key, joined = { at: 1_791_000_000 }) => ({
  person,
  person_key: key,
  key,
  device: 'computer',
  joined,
});
// A lend as /v1/lends lists it (group.js `Lent`) and /lent too (home.rs
// `lent`), as home.test.mjs `lend` records it.
const lend = (name, more = {}) => ({
  key: ADA,
  name,
  kind: 'k',
  person: 'ada',
  group: null,
  origin: `http://${name}0lbl.localhost:4000/`,
  limit: 3,
  using: 0,
  groups: ['house'],
  reads: [],
  version: 9,
  ...more,
});
// A group as /v1/groups answers it (surface.rs `Held`).
const group = (more = {}) => ({
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
  ...more,
});
// What /v1/lends answers (group.js `Lends`, surface.rs `lends`).
const lends = (more = {}) => ({
  lent: [],
  unanswered: [],
  unheard: [],
  refused: [],
  older: [],
  shelf: [],
  changed: [],
  ...more,
});

// Ada's computer in house, lending `lent`, each answer replaced by `more`.
const ada = ({
  groups = [group()],
  people = [row('ada', ADA, 'started')],
  lent = [],
  ...more
} = {}) => ({
  'GET /v1/groups': { groups, key: ADA },
  ...Object.fromEntries(
    groups.flatMap((g) => [
      [`GET /v1/rail/people?namespace=${g.id}`, { people }],
      [`GET /v1/lends?namespace=${g.id}`, lends({ lent })],
      [`GET /publication?group=${g.id}`, { published: false }],
    ]),
  ),
  'GET /lent': { lent, signing: [], readers: {} },
  'GET /reaching': {},
  'GET /v1/room': [],
  'GET /v1/play': [],
  'GET /update': { state: 'current' },
  ...more,
});

const home = (answers, path = '/?ui=a') => open(`${AT}${path}`, answers);
// The trace lines that start with `start`, and where the first does.
const traced = (page, start) => page.lines().filter((line) => line.startsWith(start)).length;
const first = (page, start) => page.lines().findIndex((line) => line.startsWith(start));
// A refusal of the node's, as the door says one (door.rs `LendRefusal::say`).
const refused = (status, error, kind) => ({ status, body: { error, kind } });

// A clock for the page's long timers, a visit's Stop waiting for its time
// (home/page/visit.js `wait`) or a screen's next read (on-screen.js
// `tick`): each of `ms` or more held until `run` lets it go, the shorter,
// a frame or Preact's effects (vendor/hooks.js, 35 ms), on the real one.
// `restore` puts the real clock back.
const clock = (ms = 500) => {
  const real = { setTimeout: globalThis.setTimeout, clearTimeout: globalThis.clearTimeout };
  const held = new Map();
  globalThis.setTimeout = (run, wait = 0, ...args) => {
    if (wait < ms) return real.setTimeout(run, wait, ...args);
    const id = { wait };
    held.set(id, () => run(...args));
    return id;
  };
  globalThis.clearTimeout = (id) => (held.has(id) ? held.delete(id) : real.clearTimeout(id));
  return {
    held: () => [...held.keys()].map((id) => id.wait),
    run: () => {
      const due = [...held.values()];
      held.clear();
      for (const run of due) run();
    },
    restore: () => Object.assign(globalThis, real),
  };
};
// Turns of the loop, for a law to read the page while an answer is held.
const turns = async (n = 50) => {
  for (let i = 0; i < n; i++) await new Promise((resolve) => setImmediate(resolve));
};

// ---- What acts as you (pages/acts.rs) ----

// Ada's page, which reads kind `k`, and her shelf, which asks (acts.rs
// `ada`); her yes to her page using the shelf, as /choices?lend= answers
// it for her own lend (door/choices.rs `kept`: `{choices}`, each `Choice`
// with its `lends` always written, :61-82).
const page = lend('page', { reads: ['k'] });
const shelf = lend('shelf', { asks: 'the page reads the shelf' });
const AT_YES = 1_791_000_000;
const yes = { said: 'yes', at: AT_YES, lends: [] };
const using = (said) => ({ choices: { [q.label(page)]: { ...yes, said } } });
const uses = (at) =>
  at.text('#acts p').filter((p) => p.includes(' uses ') || p.includes(' was stopped using '));

// pages::acts::a_stop_on_what_apps_use_is_read_again_in_place: "What acts
// as you" says ada's page uses her shelf, with Stop; pressed, its stop is
// posted (home.rs `choose`), the groups and the section read again, and
// the line says the page was stopped using the shelf, with no reload
// (home.js@19b63b2b:221 reloaded).
test('a Stop on what apps use is read again in place, with no reload', async () => {
  const kept = { said: 'yes' };
  const stops = [];
  const answers = ada({
    lent: [page, shelf],
    [`GET /choices?lend=${q.label(shelf)}`]: () => using(kept.said),
    'POST /choices': (body) => {
      stops.push(body);
      kept.said = 'stop';
      return { app: body.app, choice: { ...yes, said: 'stop' } };
    },
  });
  const at = await home(answers);
  await at.press('#acts summary', 'What acts as you');
  assert.deepEqual(uses(at), ['ada’s page (k) uses ada’s shelf (k). Stop']);
  const groups = at.asked.filter(([, path]) => path === '/v1/groups').length;
  await at.press('#acts p button', 'Stop');
  assert.deepEqual(stops, [{ lend: q.label(shelf), app: q.label(page), said: 'stop' }]);
  assert.deepEqual(uses(at), ['ada’s page (k) was stopped using ada’s shelf (k).']);
  assert.equal(at.asked.filter(([, path]) => path === '/v1/groups').length, groups + 1);
  assert.ok(first(at, 'ACTS: stopped') < first(at, 'HOME: again: '), at.lines().join('\n'));
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// pages::acts::a_refused_stop_on_what_apps_use_is_said_beside_it: the
// Stop refused as the lender's door refuses a choice it cannot keep
// (door/choices.rs `keep`, :240-242, a 500 `fault`); ACTS traces it
// unstopped once, and the line with that Stop says why after it, with no
// reload (4b7160c0 kept it under "undefined undefined").
test('a refused Stop on what apps use is said beside it', async () => {
  const why = 'this could not be kept on the computer that lends it';
  const at = await home(
    ada({
      lent: [page, shelf],
      [`GET /choices?lend=${q.label(shelf)}`]: using('yes'),
      'POST /choices': refused(500, why, 'fault'),
    }),
  );
  await at.press('#acts summary', 'What acts as you');
  await at.press('#acts p button', 'Stop');
  assert.equal(traced(at, 'ACTS: unstopped'), 1, at.lines().join('\n'));
  const [line] = uses(at);
  assert.ok(line.includes('Stop') && line.includes(why), line);
  assert.ok(line.indexOf('Stop') < line.indexOf(why), line);
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// pages::acts::a_yes_in_another_tab_is_shown_on_visible: the page open,
// ada's page said yes to nowhere; she says yes to it in another tab, and
// /reaching, read again as the page is shown again, answers it as the
// door keeps it (door/choices.rs `reachings`, a yes whose file named no
// lend read as `lends: []`, :61-75); "What acts as you" says her page
// opens nothing yet, with no reload (v1 read /reaching once a load).
test('a yes in another tab is shown when the page is shown again', async () => {
  const reaching = { said: {} };
  const at = await home(
    ada({
      lent: [page, shelf],
      'GET /reaching': () => reaching.said,
      [`GET /choices?lend=${q.label(shelf)}`]: { choices: {} },
    }),
  );
  await at.press('#acts summary', 'What acts as you');
  const said = 'ada’s page (k) opens nothing of your friends’ as you.';
  assert.ok(!at.text('#acts').join().includes(said), at.text('#acts').join());
  reaching.said = { [q.label(page)]: yes };
  await at.shown();
  assert.ok(
    at.text('#acts p').some((p) => p.startsWith(said)),
    at.text('#acts p').join('\n'),
  );
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// ---- The index and "What you lend" (pages/index.rs) ----

// A node's answers that change as its writes land: `held` the state the
// routes read, each a function of it.
const changing = (held, routes) =>
  Object.fromEntries(
    Object.entries(routes).map(([key, answer]) => [key, (body) => answer(held, body)]),
  );

// pages::index::a_lend_from_what_runs_here_is_in_the_index_at_once:
// "Already running here" looked at, something unknown on a port (home.rs
// `running`, each row `shown`, :632-638), named notes and said notes, its
// group ticked, and lent (home.rs `lend_one`, answered `shown` with its
// name); the row says so, and HOME's `again`, after LEND's `lent`, reads
// the groups and the list, so the index lists ada's notes and "What you
// lend" lists it too, with no reload (home.js@19b63b2b:247 redrew only
// "What you lend").
test('a lend from what runs here is in the index and in What you lend at once', async () => {
  const port = 21000;
  const shown = (kind, name) => ({ port, kind, name, signs: null, keeps: [], unknown: null });
  const notes = lend('notes', {
    kind: 'notes',
    limit: null,
    origin: 'http://notes0lbl.localhost:4000/',
  });
  const posted = [];
  const at = await home({
    ...ada(),
    ...changing(
      { lent: [] },
      {
        'GET /v1/lends?namespace=house': (held) => lends({ lent: held.lent }),
        'GET /lent': (held) => ({ lent: held.lent, signing: [], readers: {} }),
        'GET /running': (held) => ({
          running: held.lent.length ? [] : [shown(null, null)],
          named: [],
        }),
        'POST /running': (held, body) => {
          posted.push(body);
          held.lent = [notes];
          return shown('notes', 'notes');
        },
      },
    ),
  });
  await at.press('#running button', 'Look for programs');
  const row = `#running li`;
  assert.ok(at.text(row)[0].startsWith(`Something at http://localhost:${port}.`), at.text(row)[0]);
  await at.type(`${row} input[name=name]`, 'notes');
  await at.type(`${row} input[name=kind]`, 'notes');
  await at.tick(`${row} input[type=checkbox]`);
  await at.press(`${row} button`, 'Lend it to house');
  assert.deepEqual(posted, [
    { port, name: 'notes', kind: 'notes', groups: ['house'], terms: null },
  ]);
  assert.ok(at.text(row)[0].startsWith('You lent notes'), at.text(row)[0]);
  assert.ok(
    at.text('#home table.index').join().includes('ada’s notes'),
    at.text('#home table.index').join(),
  );
  assert.ok(
    at.text('#lent li')[0].startsWith('notes: everyone in the groups you chose at once'),
    at.text('#lent li').join('\n'),
  );
  const lent = first(at, 'LEND: lent');
  assert.ok(lent >= 0 && lent < first(at, 'HOME: again: '), at.lines().join('\n'));
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// What /lent answers once a change lands (home.rs `change`: 204, no body,
// :497-500), the declaration as `lend::mutate` or `withdraw` leaves it.
const NONE = { status: 204, body: undefined };

// pages::index::three_changes_to_a_lend_are_read_again_in_place: ada lends
// shelf to one at once; she sets four, then everyone, then stops lending
// it; each change posted, CHANGE answered, and "What you lend" read again
// by LENDING through HOME's `again`, never a load of the page
// (home.js@19b63b2b:283 loaded it again after each).
test('three changes to a lend are each read again in place', async () => {
  const posted = [];
  const answers = changing(
    { lent: [lend('shelf', { kind: 'books', limit: 1 })] },
    {
      'GET /lent': (held) => ({ lent: held.lent, signing: [], readers: {} }),
      'POST /lent': (held, { lend: name, to }) => {
        posted.push(to);
        held.lent =
          to === 'stop'
            ? []
            : held.lent.map((one) => (one.name === name ? { ...one, ...to } : one));
        return NONE;
      },
    },
  );
  const at = await home({ ...ada(), ...answers });
  // What "What you lend" lists: its rows, or that it lends nothing (the
  // twin's selectors take no comma).
  const listed = () => [...at.text('#lent > p'), ...at.text('#lent li')];
  const said = [];
  const press = async (words) => {
    const read = traced(at, 'LENDING: answered');
    await at.press('#lent button', words);
    assert.equal(traced(at, 'LENDING: answered'), read + 1, `${words}: ${at.lines().join('\n')}`);
    said.push(listed()[0]);
  };
  await at.type('#lent input', '4');
  await press('Let in this many');
  await press('Let everyone in');
  await press('Stop lending');
  assert.ok(said[0].startsWith('shelf: up to 4 people at once'), said[0]);
  assert.ok(said[1].startsWith('shelf: everyone in the groups you chose at once'), said[1]);
  assert.deepEqual(listed(), ['You lend nothing yet.']);
  assert.deepEqual(posted, [{ limit: 4 }, { limit: null }, 'stop']);
  assert.equal(traced(at, 'HOME: again: '), 3, at.lines().join('\n'));
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// pages::index::the_secret_stays_shown_when_the_page_is_read_again: signing
// people in with svrngs turned on for shelf (home.rs `sign_in`, its fill
// :586-593), the secret is shown in the row; the page hidden and shown,
// "What you lend" read again (/lent's `signing` names shelf now, home.rs
// `lent` :424-426): the row says credentials are made and the app is still
// to set up, its button gone, and still shows the secret, which no
// request, link or trace line carries (SIGN_IN_WITH_SVRNGS §5).
test('the secret stays shown when the page is read again, and nothing carries it', async () => {
  const secret = 'S3CR3T-of-the-shelf-client';
  const answers = changing(
    { signing: [] },
    {
      'GET /lent': (held) => ({
        lent: [lend('shelf', { kind: 'books' })],
        signing: held.signing,
        readers: {},
      }),
      'POST /lent': (held, { to }) => {
        assert.equal(to, 'sign-in');
        held.signing = ['shelf'];
        return {
          issuer: 'http://svrngs.localhost:4000/oidc/shelf',
          client_id: 'shelf-client',
          client_secret: secret,
          deviations: 'It is plain http on this computer, not https.',
        };
      },
    },
  );
  const at = await home({ ...ada(), ...answers });
  await at.press('#lent button', 'Sign people in with svrngs');
  assert.equal(traced(at, 'CHANGE: signed'), 1, at.lines().join('\n'));
  assert.ok(at.text('#lent li')[0].includes(secret), at.text('#lent li')[0]);
  const read = traced(at, 'LENDING: answered');
  await at.shown();
  assert.equal(traced(at, 'LENDING: answered'), read + 1, at.lines().join('\n'));
  const [later] = at.text('#lent li');
  assert.ok(later.includes('Sign-in credentials created; app setup still required.'), later);
  assert.ok(!later.includes('Sign people in with svrngs'), later);
  assert.ok(later.includes(secret), later);
  const links = at.all('a').map((a) => a.attributes.href);
  for (const carried of [at.asked, links, at.lines()])
    assert.ok(!JSON.stringify(carried).includes(secret), JSON.stringify(carried));
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// The shelf as /v1/lends lists it (surface.rs `lends`, `shelf`: what
// shelf.txt offers, each `Shelved`), and Money, brought to house, as the
// group's own lend (group.js `Lent`, `group` its id).
const SHELVED = [
  { name: 'Money', words: 'who owes whom, and for what.' },
  { name: 'Events', words: 'the get-togethers people post.' },
];
const money = lend('house Money', {
  kind: 'Money',
  group: 'house',
  limit: null,
  origin: 'http://money0lbl.localhost:4000/',
});
// House's lends, Money in them once `held.brought`; /shelf answering each
// app brought as home.rs's /shelf does (folder.rs `bring`, `Brought`),
// `hold` holding its answer when given.
const shelved = (held, hold) =>
  changing(held, {
    'GET /v1/lends?namespace=house': (h) =>
      lends({ lent: h.brought ? [money] : [], shelf: SHELVED }),
    'POST /shelf': (h, body) => {
      h.posts.push(body);
      const answer = () => ((h.brought = true), body[1].map((app) => ({ app })));
      return hold ? hold.then(answer) : answer();
    },
  });

// pages::index::a_bring_is_in_the_home_s_index_with_no_reload: ada ticks
// Money and brings it; house's index lists Money, read again by HOME's
// `again`, with no reload (question.js@19b63b2b:207 redrew only the
// shelf).
test('a bring is in the home’s index, read again, with no reload', async () => {
  const held = { brought: false, posts: [] };
  const at = await home({ ...ada(), ...shelved(held) });
  assert.ok(!at.text('#home table.index tr').join().includes('Money'));
  await at.tick('#home label input');
  await at.press('#home button', 'Bring the ticked ones');
  assert.ok(
    at.text('#home table.index tr').some((tr) => tr.includes('Money Who owes')),
    at.text('#home table.index tr').join('\n'),
  );
  assert.ok(first(at, 'SHELF: brought (') < first(at, 'HOME: again: '), at.lines().join('\n'));
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// ---- The shelf (pages/shelf.rs) ----

// pages::shelf::the_home_page_s_bring_is_answered_beside_its_button: Money
// ticked, the button pressed twice at once, as a quick thumb does: held
// while it brings, given back once answered, the second press refused by
// SHELF, so /shelf is posted once; the answer the button's next sibling;
// then the group read again, HOME's and GROUP's `again` after SHELF's
// `brought`, which says Money is on the table and offers it no more, with
// no reload (question.js@19b63b2b:207 replaced the words above the boxes).
test('the home page’s bring is answered beside its button, the button held while it brings', async () => {
  let give;
  const hold = new Promise((resolve) => (give = resolve));
  const held = { brought: false, posts: [] };
  const at = await home({ ...ada(), ...shelved(held, hold) });
  await at.tick('#home label input');
  const button = () =>
    at.all('#home button').find((b) => words(b).startsWith('Bring the ticked ones'));
  await Promise.all([1, 2].map(() => at.press('#home button', 'Bring the ticked ones')));
  const busy = ['disabled' in button().attributes];
  give();
  await at.settle();
  busy.push('disabled' in button().attributes);
  assert.deepEqual(busy, [true, false]);
  assert.equal(traced(at, 'SHELF: bring: idle -> bringing'), 1, at.lines().join('\n'));
  assert.equal(traced(at, 'SHELF: bring refused in bringing'), 1, at.lines().join('\n'));
  assert.deepEqual(held.posts, [['house', ['Money']]]);
  assert.equal(
    words(button().nextSibling ?? { textContent: '' }),
    'You brought Money to house: everyone in it can open it in a few seconds.',
  );
  const brought = first(at, 'SHELF: brought (');
  assert.ok(brought >= 0, at.lines().join('\n'));
  for (const again of ['HOME: again: ', 'GROUP: again: '])
    assert.ok(first(at, again) > brought, `${again}: ${at.lines().join('\n')}`);
  const shelf = at.text('#home section section').find((s) => s.startsWith('Bring things')) ?? '';
  assert.ok(shelf.includes('Money is on the table.'), shelf);
  assert.ok(!shelf.includes('Money: who owes'), shelf);
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// ---- A group joined while the page is open (pages/join.rs) ----

// pages::join::a_group_joined_by_the_running_svrngs_is_on_the_page_when_shown_again:
// ada's page open on house; her running svrngs joins Sam's Trip as Dee
// (as `svrngs join` asks it, /v1/joins), so /v1/groups answers Trip too
// (surface.rs `Held`); the page, hidden and shown again, reads its groups
// (HOME's `visible`) and draws Trip's section beside house's, with no
// reload. Planted at the time: a page that read its groups once drew
// house alone.
test('a group joined by the running svrngs is on the page when it is shown again', async () => {
  const trip = group({
    id: 'trip',
    name: 'Trip',
    person: 'Dee',
    keeps: false,
    made: { maker: 'Sam', since: 1_791_000_000 },
  });
  const held = { groups: [group()] };
  const answers = ada({ groups: [group(), trip] });
  answers['GET /v1/groups'] = () => ({ groups: held.groups, key: ADA });
  const at = await home(answers);
  const heads = () => at.text('#home section h2');
  assert.ok(
    heads().some((h) => h.includes('house')),
    heads().join('\n'),
  );
  assert.ok(!heads().some((h) => h.includes('Trip')), heads().join('\n'));
  held.groups = [group(), trip];
  await at.shown();
  assert.equal(traced(at, 'HOME: visible:'), 1, at.lines().join('\n'));
  assert.ok(
    heads().some((h) => h.includes('house')),
    heads().join('\n'),
  );
  assert.ok(
    heads().some((h) => h.includes('Trip')),
    heads().join('\n'),
  );
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// ---- The door, visits and the screen (pages/members.rs) ----

// pages::members::a_guest_s_stop_ends_the_visit_with_no_reload: Sam is
// ada's guest for an hour, listed under "Who you have over" with Stop
// (/v1/groups' `visiting`, rail.rs `visiting`, :185-193); Stop posts the
// visit's id (home.rs `stop`, answered as rail.rs `append_act` answers,
// its `ts_unix` a second on, :250-257); VISITS waits for that time, and
// only then has HOME read the groups again, which list Sam nowhere, with
// no reload (home.js@19b63b2b:163 reloaded).
test('a guest’s Stop waits for its time, then the groups are read again and the guest is gone', async () => {
  const time = clock();
  try {
    // Its Stop's time a second past the next whole second, so its wait is
    // one at least, whenever in a second the law runs.
    const now = Math.ceil(Date.now() / 1000);
    const visit = {
      id: 'v1',
      guest: 'Sam',
      host: 'ada',
      host_key: ADA,
      until: now + 3600,
      yours: true,
    };
    const held = { visiting: [visit] };
    const stops = [];
    const answers = ada();
    answers['GET /v1/groups'] = () => ({ groups: [group({ visiting: held.visiting })], key: ADA });
    answers['POST /visits'] = (body) => {
      stops.push(body);
      held.visiting = [];
      return { id: 'stop1', seq: 2, actor: ADA, ts_unix: now + 1, namespace: 'house', op: {} };
    };
    const at = await home(answers);
    const [listed] = at.text('#home .yours li');
    assert.ok(listed.startsWith('Sam, until ') && listed.endsWith('Stop Sam’s visit'), listed);
    assert.equal(at.text('#home .yours li').length, 1);
    const reads = at.asked.filter(([, path]) => path === '/v1/groups').length;
    await at.press('#home .yours button', 'Stop Sam’s visit');
    assert.deepEqual(stops, [['house', 'v1']]);
    assert.equal(time.held().length, 1, 'the Stop waits for its time');
    assert.equal(traced(at, 'VISITS: waited'), 0);
    assert.equal(at.asked.filter(([, path]) => path === '/v1/groups').length, reads);
    time.run();
    await at.settle();
    assert.equal(traced(at, 'VISITS: waited'), 1, at.lines().join('\n'));
    assert.equal(at.asked.filter(([, path]) => path === '/v1/groups').length, reads + 1);
    assert.deepEqual([...at.text('#home .yours li'), ...at.text('#home .visiting li')], []);
    assert.deepEqual([at.went, at.unanswered], [[], []]);
  } finally {
    time.restore();
  }
});

// pages::members::the_door_is_judged_again_when_shown_again: cy in ada's
// house, lit "on now", bo drawn off beside, bo's computer never
// answering (/v1/lends' `unanswered`, node.rs `Unlisted`); cy's computer
// goes off while the page is open; hidden and shown again, the page reads
// the group again and its door draws cy "did not answer", with no reload
// (v1 judged its door once).
test('the door is judged again when the page is shown again', async () => {
  const people = [row('ada', ADA, 'started'), row('bo', BO), row('cy', CY)];
  const off = (person, key) => ({ person, person_key: key, joined: { at: 1_791_000_000 } });
  const held = { unanswered: [off('bo', BO)] };
  const answers = ada({ people });
  answers['GET /v1/lends?namespace=house'] = () => lends({ unanswered: held.unanswered });
  const at = await home(answers);
  const door = () => at.text('#home .door li');
  assert.deepEqual(at.text('#home .door li.off'), ['bo did not answer'], door().join('\n'));
  assert.ok(
    at.text('#home .door li.on').some((li) => li.startsWith('cy')),
    door().join('\n'),
  );
  held.unanswered = [off('bo', BO), off('cy', CY)];
  await at.shown();
  assert.equal(traced(at, 'GROUP: answered (lends 2)'), 1, at.lines().join('\n'));
  assert.ok(!at.text('#home .door li.on').some((li) => li.startsWith('cy')), door().join('\n'));
  assert.ok(door().at(-1).endsWith('cy did not answer'), door().join('\n'));
  assert.deepEqual([at.went, at.unanswered], [[], []]);
});

// pages::members::ten_screen_ticks_never_read_asking_again: house on a
// screen, `/?screen=house`; once its door is drawn, ada lit and bo off,
// the screen reads the group ten times more by itself (SCREEN's `tick`, a
// second after each read began, held here on `clock`), and while each
// tick's read is out, and once it lands, no window reads "asking…": each
// door is judged before it takes the last one's place
// (home.js@19b63b2b:299-301).
test('ten screen ticks never read asking again, each read out or landed', async () => {
  const time = clock();
  try {
    const people = [row('ada', ADA, 'started'), row('bo', BO)];
    const off = { person: 'bo', person_key: BO, joined: { at: 1_791_000_000 } };
    const gives = [];
    const answers = ada({ people });
    // Answered at once until the screen is first drawn, then held, each read.
    const held = { on: false };
    answers['GET /v1/lends?namespace=house'] = () =>
      held.on
        ? new Promise((give) => gives.push(() => give(lends({ unanswered: [off] }))))
        : lends({ unanswered: [off] });
    const at = await home(answers, '/?screen=house');
    assert.ok(at.text('#home .door li.on')[0].startsWith('ada'), at.text('#home').join());
    assert.deepEqual(at.text('#home .door li.off'), ['bo did not answer']);
    const asking = [];
    held.on = true;
    for (let tick = 0; tick < 10; tick++) {
      assert.equal(time.held().length, 1, `tick ${tick}: the next read waits on the clock`);
      time.run();
      await turns();
      assert.equal(gives.length, 1, `tick ${tick}: its read is out`);
      asking.push(at.text('#home').join().includes('asking…'));
      gives.shift()();
      await at.settle();
      asking.push(at.text('#home').join().includes('asking…'));
    }
    assert.equal(traced(at, 'SCREEN: tick'), 10, at.lines().join('\n'));
    assert.deepEqual(asking, Array(20).fill(false));
    assert.deepEqual([at.went, at.unanswered], [[], []]);
  } finally {
    time.restore();
  }
});
