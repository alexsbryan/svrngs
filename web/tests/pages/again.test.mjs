// What the pages step (crates/svrngs/tests/pages/) held of each page read
// again, with no browser. The home page over its twin (twin.mjs, harness
// 5), each route answered from a recording in the shape its writer makes,
// named beside it; layout A (`/?ui=a`), as those tests' drivers opened it
// (web/tests/page.mjs `longPage`). The phone's page composed from its flows
// and drawn by them (`phone` below), each effect that leaves the page
// answered by the law. Each law names the pages test whose claim it holds.
// "No reload" is the twin's `went`, a reload, a form or a link followed
// kept there, and each home law asserts it empty; the phone's page loads
// again only at its edge, as the tab's lock is taken (enter.js:533-535) or
// its address changes (web/app.js:80-83), so each phone law asserts no
// effect opened it again (`again(at)`).
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

// ---- The phone's page (pages/*.rs, WebKit) ----

// PAGE composed from the flows as web/app.js composes it (:45-58), its
// machine ui/machine.js's, so each step is traced as the browser traces it;
// its io the flows' own `send` and, for each effect that leaves the page
// (the wasm, the store, the clock), the law's answer (`answers`), or none,
// held. Its places drawn as Page draws them (page.js:369-371), each
// component called, but those that hold hooks: Ends and AskWhenRead, which
// draw nothing and send from an effect, rendered by Preact for real into a
// parent of their own; the shelf, ui/shelf.js `drawn` over a SHELF kept
// here as `Shelf` keeps its own (ui/shelf.js:201-210), its `bring` the
// law's, as dial.js `through` would answer it; and "What acts as you" and
// a Copy, which no law here reads.
const PHONE = ['enter', 'computer', 'join', 'ask', 'index', 'door', 'shelf'];
const parts = (await import('../../page/page.js?v=__VERSION__')).compose(
  Object.fromEntries(
    await Promise.all(
      [...PHONE, 'visit', 'among', 'acts', 'uses'].map(async (f) => [
        f,
        await import(`../../page/${f}.js?v=__VERSION__`),
      ]),
    ),
  ),
);
const UI = '../../../crates/svrngs-node/src/home/ui/';
const { machine } = await import(`${UI}machine.js?v=__VERSION__`);
const { html, render } = await import(`${UI}view.js?v=__VERSION__`);
const { SHELF, Shelf, drawn } = await import(`${UI}shelf.js?v=__VERSION__`);
const { Copy } = await import(`${UI}copy.js?v=__VERSION__`);
const { Ends } = await import('../../page/ends.js?v=__VERSION__');
const { AskWhenRead } = await import('../../page/ask-when-read.js?v=__VERSION__');
const { WhatActsAsYou } = await import('../../page/what-acts-as-you.js?v=__VERSION__');
// ui/shelf.js's first state (:181).
const SHELF_FIRST = { at: { bring: 'idle' }, ticked: [], brought: [], open: false, said: '' };

// A drawing as plain nodes, `{ tag, props, kids }`, words as strings.
const tree = (v, hooked) => {
  if (v == null || typeof v === 'boolean') return [];
  if (Array.isArray(v)) return v.flatMap((one) => tree(one, hooked));
  if (typeof v !== 'object') return [String(v)];
  if (typeof v.type === 'function')
    return hooked.has(v.type) ? hooked.get(v.type)(v.props) : tree(v.type(v.props), hooked);
  return [{ tag: v.type, props: v.props, kids: tree(v.props.children, hooked) }];
};
const said = (node) => (typeof node === 'string' ? node : node.kids.map(said).join(''));
const spaced = (node) => said(node).replace(/\s+/g, ' ').trim();
// Each node under `nodes` that `holds`, in the order a reader meets them.
const among = (nodes, holds) =>
  nodes.flatMap((node) =>
    typeof node === 'string' ? [] : [...(holds(node) ? [node] : []), ...among(node.kids, holds)],
  );
const byId = (nodes, id) => among(nodes, (node) => node.props.id === id)[0];
const NOT_REAL = { preventDefault() {}, currentTarget: {} };

/**
 * The phone's page from `state`, each effect run answered by `answers`
 * (by its `run`, a function of the effect answering a message or a
 * promise of one), and the shelf's bring by `answers.bring`.
 */
function phone(state, answers = {}) {
  const lines = [];
  globalThis.__trace = (line) => lines.push(line);
  const ran = [];
  const io = Object.fromEntries(
    Object.keys(parts.io).map((run) => [
      run,
      (effect) => {
        if (run === 'send') return effect.m;
        ran.push(effect);
        return answers[run] ? answers[run](effect) : new Promise(() => {});
      },
    ]),
  );
  const page = machine('PAGE', { states: parts.states, step: parts.step, io }, state);
  const parents = new Map();
  const real = (Component) => (props) => {
    if (!parents.has(Component)) parents.set(Component, { nodeType: 1, childNodes: [] });
    render(html`<${Component} ...${props} />`, parents.get(Component));
    return [];
  };
  let shelf;
  const hooked = new Map([
    [Ends, real(Ends)],
    [AskWhenRead, real(AskWhenRead)],
    [WhatActsAsYou, () => []],
    [Copy, () => []],
    [
      Shelf,
      (props) => {
        shelf ??= machine(
          'SHELF',
          SHELF(answers.bring ?? (() => new Promise(() => {})), props.after),
          SHELF_FIRST,
        );
        return tree(drawn(shelf.get(), shelf.dispatch, props), hooked);
      },
    ],
  ]);
  const draw = () =>
    tree(
      parts.places.map(
        (Place) =>
          html`<${Place} s=${page.get()} send=${page.dispatch} acts=${parts.acts(page.get())} />`,
      ),
      hooked,
    );
  const at = {
    page,
    ran,
    lines: () => [...lines],
    draw,
    heading: () => parts.heading(page.get()),
    // Drawn again on each turn, as Page draws on each step (page.js
    // `useStore`), so a hooked part's effect runs, until nothing is traced
    // or changed for ten turns.
    settle: async () => {
      let [last, still] = ['', 0];
      for (let turn = 0; turn < 2000 && still < 10; turn++) {
        await new Promise((resolve) => setTimeout(resolve, 0));
        const now = `${lines.length} ${JSON.stringify(page.get())} ${JSON.stringify(shelf?.get())}`;
        draw();
        [last, still] = [now, now === last ? still + 1 : 0];
      }
      return at;
    },
    send: (message) => (page.dispatch(message), at.settle()),
    // A button pressed by the words it holds, as a browser runs its onClick.
    press: (words) => {
      const [button] = among(draw(), (n) => n.tag === 'button' && spaced(n).includes(words));
      if (!button?.props.onClick) throw new Error(`nothing to press: ${words}`);
      button.props.onClick(NOT_REAL);
      return at.settle();
    },
    // A form sent, as its button sends it.
    submit: (id) => (byId(draw(), id).props.onSubmit(NOT_REAL), at.settle()),
    type: (id, value) => (
      byId(draw(), id).props.onInput({ currentTarget: { value } }),
      at.settle()
    ),
  };
  return at.settle();
}

// The read's body (enter.test.mjs `body`) as a group's computer answers
// it (surface.rs `lends`, rail.rs `standing_json` and `visiting`).
const body = (more = {}) => ({
  version: 3,
  lent: [],
  unanswered: [],
  unheard: [],
  refused: [],
  older: [],
  removals: [],
  visiting: [],
  people: [row('ada', ADA, 'started'), row('bo', BO)],
  shelf: [],
  changed: [],
  ...more,
});
// Bo's phone in house, held, opened and read through ada's computer, as
// enter.test.mjs steps it, `wants` what its address asks (enter.js `hold`):
// the state its first read leaves.
const PHONE_HOUSE = { id: 'house', name: 'house', person: 'bo', token: 'TOKEN', addresses: [] };
const via = { key: ADA, person: 'ada', relay: null };
const readOnce = (b, lent = [], wants = null) => {
  const steps = [
    { type: 'hold', group: PHONE_HOUSE, wants },
    { type: 'held' },
    { type: 'opened', group: PHONE_HOUSE },
    { type: 'read', n: 1, ok: true, via, body: b, lent },
  ];
  return steps.reduce(
    (s, m) => {
      const r = parts.step(s, m);
      assert.equal(r.refused, undefined, `${m.type}: ${r.refused}`);
      return r.state;
    },
    parts.first({ fine: false }),
  );
};
// A lend as the phone's read gives it (enter.js `Lent`: the edge's
// readLends, its `label` the wasm's).
const lentHere = (name, more = {}) => ({
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
// A read as enter.js `readLends` answers it (:577-590).
const answered = (n, b, lent) => ({
  type: 'read',
  n,
  ok: true,
  via,
  body: b,
  lent,
  older: false,
  why: `200, at version ${b.version}, ${lent.length} lent`,
});
// The effects that would open the page again, a look at its address or a
// hold of its tab (enter.js `look`, `lock`): none, drawn again in place.
const again = (at) => at.ran.filter((effect) => ['look', 'lock'].includes(effect.run));
const rows = (nodes) => among(byId(nodes, 'lent')?.kids ?? [], (n) => n.tag === 'tr').map(spaced);

// ---- A guest's visit, on the phone (pages/members.rs) ----

// The visit ada's computer answers a guest's redeem, as visit.js `stay`
// makes it of the node's /visit answer (:148-165, the lends by name and
// kind, each its label the wasm's), until `until`.
const visited = (until) => ({
  type: 'redeemed',
  ended: false,
  why: `a guest of ada until ${until}, opening notes`,
  stay: {
    host: 'ada',
    until,
    lent: [
      {
        key: ADA,
        relay: null,
        name: 'notes',
        kind: 'k',
        person: 'ada',
        version: 9,
        label: 'notes0lbl',
      },
    ],
  },
});
// Sam at ada's guest's link, in a browser holding nothing (no visit kept,
// visit.js `recalling`), their name given once asked.
const guest = async (until, more = {}) => {
  const redeems = [];
  const at = await phone(parts.first({ fine: false }), {
    recall: () => ({ type: 'unkept', why: 'no visit is kept for this link' }),
    redeem: (effect) => (redeems.push(effect), visited(until)),
    ...more,
  });
  await at.send({ type: 'looked', fragment: 'visit=VISIT', kept: [], linked: true });
  const asking = at.heading();
  const asked = byId(at.draw(), 'join');
  await at.type('name', 'Sam');
  await at.submit('join');
  return { at, asking, asked, redeems };
};

// pages::members::a_guest_at_their_link_is_asked_their_name_once_and_is_in:
// "You’re invited over as a guest", the name asked in #join; given, the
// link is redeemed once, as Sam; "You’re ada’s guest until …", ada's notes
// listed for Sam to open, and the name asked no more.
test('a guest at their link is asked their name once, and is in, ada’s notes listed', async () => {
  const until = Math.floor(Date.now() / 1000) + 3600;
  const { at, asking, asked, redeems } = await guest(until);
  assert.deepEqual(asking, ['You’re invited over as a guest']);
  assert.equal(asked.props.hidden, false);
  assert.deepEqual(
    redeems.map(({ token, guest: name }) => [token, name]),
    [['VISIT', 'Sam']],
  );
  const [heading, tone] = at.heading();
  assert.ok(heading.startsWith('You’re ada’s guest until '), heading);
  assert.equal(tone, 'good');
  assert.deepEqual(rows(at.draw()), ['ada’s notes (k)']);
  assert.equal(byId(at.draw(), 'join').props.hidden, true, 'the name is asked no more');
  assert.deepEqual(again(at), []);
});

// pages::members::a_guest_s_visit_of_5_s_ends_while_shown: Sam in,
// ada's notes listed; the visit's end waited for (visit.js `over`, dial.js
// `when`), held until the law lets it come; come, "PAGE: ended" once, the
// heading says the visit has ended and nothing is listed, the page drawn
// again in place (web/app.js@19b63b2b:221-224 listed them until the link
// was opened again).
test('a guest’s visit ends while shown: said ended, nothing listed', async () => {
  const until = Math.floor(Date.now() / 1000) + 5;
  let end;
  const over = [];
  const { at } = await guest(until, {
    over: (effect) => (over.push(effect.until), new Promise((come) => (end = come))),
  });
  assert.deepEqual(rows(at.draw()), ['ada’s notes (k)']);
  assert.deepEqual(over, [until], 'the end waited for is the visit’s');
  assert.equal(traced(at, 'PAGE: ended'), 0);
  end({ type: 'ended', why: `the visit ended at ${until}` });
  await at.settle();
  assert.equal(traced(at, 'PAGE: ended'), 1, at.lines().join('\n'));
  const [heading, tone] = at.heading();
  assert.ok(heading.startsWith('Your visit as ada’s guest ended '), heading);
  assert.equal(tone, 'bad');
  assert.deepEqual(rows(at.draw()), []);
  assert.deepEqual(again(at), []);
});

// pages::members::a_visit_ended_while_open_drops_on_the_phone: Sam is
// ada's guest; bo's phone reads house and lists Sam visiting, and waits
// for the visit's end (ends.js `Ends`, door.js `end`, dial.js `when`),
// held until the law lets it come; come, the page reads the group again
// by itself, "PAGE: reread (a visit ended" once, and lists nobody (v1
// read the group once).
test('a visit that ends while the phone’s page is open drops from it', async () => {
  const until = Math.floor(Date.now() / 1000) + 20;
  const sam = { id: 'v1', guest: 'Sam', host: 'ada', host_key: ADA, until, yours: false };
  let end;
  const ends = [];
  const at = await phone(readOnce(body({ visiting: [sam] })), {
    end: (effect) => (ends.push(effect.until), new Promise((come) => (end = come))),
    read: ({ n }) => answered(n, body(), []),
  });
  const visiting = () =>
    among(at.draw(), (n) => n.tag === 'li' && n.kids.length && /guest, until/.test(said(n))).map(
      spaced,
    );
  const [line] = visiting();
  assert.ok(line.startsWith('Sam, ada’s guest, until '), line);
  assert.deepEqual(ends, [until], 'the end waited for is Sam’s');
  end({ type: 'reread', why: `a visit ended at ${until}` });
  await at.settle();
  assert.equal(traced(at, 'PAGE: reread (a visit ended'), 1, at.lines().join('\n'));
  assert.deepEqual(visiting(), []);
  assert.deepEqual(again(at), []);
});

// ---- The phone's index and shelf (pages/index.rs, shelf.rs) ----

// Money, brought to house, as the phone's read lists it: the group's own.
const moneyHere = lentHere('house Money', {
  kind: 'Money',
  group: 'house',
  label: 'money0lbl',
  limit: null,
});
// Bo's phone reading house with the shelf offered; /shelf, through ada's
// computer, answering each app brought (folder.rs `bring`), and the group
// read again listing Money, each held until `holds` gives it when given.
const bringing = async (holds = {}) => {
  const brings = [];
  const at = await phone(readOnce(body({ shelf: SHELVED })), {
    bring: (group, apps) => {
      brings.push([group.id, apps]);
      return (holds.bring ?? Promise.resolve()).then(() => apps.map((app) => ({ app })));
    },
    read: ({ n }) =>
      (holds.read ?? Promise.resolve()).then(() =>
        answered(n, body({ shelf: SHELVED }), [moneyHere]),
      ),
  });
  const shelf = () => byId(at.draw(), 'shelf');
  const [money] = among([shelf()], (n) => n.tag === 'input' && n.props.type === 'checkbox');
  money.props.onChange({ currentTarget: { checked: true } });
  await at.settle();
  return { at, brings, shelf };
};

// pages::index::a_bring_is_in_the_phone_s_index_with_no_reload: bo ticks
// Money and brings it; the shelf's `after` has the page read the group
// again (`reread`, shelf.js), and #lent lists Money, drawn again in place.
test('a bring is in the phone’s index, read again by the page', async () => {
  const { at } = await bringing();
  assert.deepEqual(rows(at.draw()), []);
  await at.press('Bring the ticked ones');
  assert.ok(rows(at.draw())[0]?.startsWith('Money'), rows(at.draw()).join('\n'));
  const brought = first(at, 'SHELF: brought (');
  assert.ok(brought >= 0 && first(at, 'PAGE: reread: ') > brought, at.lines().join('\n'));
  assert.deepEqual(again(at), []);
});

// pages::shelf::the_phone_s_bring_is_answered_beside_its_button: Money
// ticked, the button pressed twice at once while the bring is out: held,
// then given back; the second press refused by SHELF; one bring; the
// answer the button's next sibling; then the page reads the group again,
// which says Money is on the table and offers it no more
// (question.js@19b63b2b:207 replaced the words above the boxes).
test('the phone’s bring is answered beside its button, the button held while it brings', async () => {
  const [hold, give] = [{}, {}];
  for (const what of ['bring', 'read']) hold[what] = new Promise((go) => (give[what] = go));
  const { at, brings, shelf } = await bringing(hold);
  const button = (drawing = shelf()) =>
    among([drawing], (n) => n.tag === 'button' && spaced(n).startsWith('Bring the ticked ones'))[0];
  // Both on the button as drawn, before the page draws again, as a quick
  // thumb's two presses land.
  const pressed = button();
  pressed.props.onClick(NOT_REAL);
  pressed.props.onClick(NOT_REAL);
  await at.settle();
  const busy = [button().props.disabled];
  give.bring();
  await at.settle();
  // Answered, while the group's read again is out.
  const now = shelf();
  const answered = button(now);
  busy.push(answered.props.disabled);
  assert.deepEqual(busy, [true, false]);
  assert.equal(traced(at, 'SHELF: bring: idle -> bringing'), 1, at.lines().join('\n'));
  assert.equal(traced(at, 'SHELF: bring refused in bringing'), 1, at.lines().join('\n'));
  assert.deepEqual(brings, [['house', ['Money']]]);
  const [line] = among([now], (n) => n.kids.includes(answered));
  assert.equal(
    said(line.kids[line.kids.indexOf(answered) + 1] ?? '').trim(),
    'You brought Money to house: everyone in it can open it in a few seconds.',
  );
  give.read();
  await at.settle();
  const brought = first(at, 'SHELF: brought (');
  assert.ok(brought >= 0 && first(at, 'PAGE: reread: ') > brought, at.lines().join('\n'));
  assert.ok(spaced(shelf()).includes('Money is on the table.'), spaced(shelf()));
  assert.ok(!spaced(shelf()).includes('Money: who owes'), spaced(shelf()));
  assert.deepEqual(again(at), []);
});

// pages::index::a_lend_filled_while_shown_reads_full_now_on_coming_back:
// ada lends shelf to one at once; bo's phone lists it, open to him; cy
// opens it meanwhile, so ada's computer counts one using it (door/limit.rs
// `ask`; /lends' `using`, door.rs `offered`); the page hidden and shown
// again (web/app.js:77-79 `shown`) reads the group again, "PAGE: read"
// once more, and its line says "Full now.", drawn again in place.
test('a lend filled while shown reads Full now on coming back', async () => {
  const one = lentHere('shelf', { kind: 'books', limit: 1 });
  const at = await phone(readOnce(body(), [one]), {
    read: ({ n }) => answered(n, body(), [{ ...one, using: 1 }]),
  });
  const [shown] = rows(at.draw());
  assert.ok(shown.includes('shelf') && !shown.includes('Full now.'), shown);
  const reads = traced(at, 'PAGE: read');
  await at.send({ type: 'shown' });
  assert.equal(traced(at, 'PAGE: read'), reads + 1, at.lines().join('\n'));
  assert.ok(rows(at.draw())[0].includes('Full now.'), rows(at.draw()).join('\n'));
  assert.deepEqual(again(at), []);
});

// ---- The ask, on the phone (pages/acts.rs) ----

// pages::acts::a_refused_yes_keeps_the_ask_shown: bo's phone at
// `#ask=<shelf>&app=<page>&at=/` asks, once the group is read
// (AskWhenRead), whether ada's page may use her shelf; ada stops lending
// the shelf, and bo's Yes is refused at her computer's door, as uses.js
// `door` says a refused answer (:133-135); the heading says so with the
// ask still shown, so bo can say it again (web/app.js@19b63b2b:316 hid
// the form and left it so).
test('a refused yes is said in the heading, and the ask stays shown', async () => {
  const appLent = lentHere('page', { reads: ['k'] });
  const lendLent = lentHere('shelf', { asks: 'the page reads the shelf' });
  const answers = [];
  const wants = { ask: lendLent.label, app: appLent.label, at: '/' };
  const at = await phone(readOnce(body(), [appLent, lendLent], wants), {
    answer: (effect) => (
      answers.push([effect.l.label, effect.app]),
      { type: 'unsaid', said: 'ada’s computer did not keep your answer.', why: 'answered 404' }
    ),
  });
  const [asked, tone] = at.heading();
  assert.ok(asked.startsWith('Let '), asked);
  assert.equal(tone, 'good');
  assert.equal(byId(at.draw(), 'ask').props.hidden, false);
  await at.submit('ask');
  assert.deepEqual(answers, [[lendLent.label, appLent.label]]);
  assert.deepEqual(at.heading(), ['ada’s computer did not keep your answer.', 'bad']);
  assert.equal(byId(at.draw(), 'ask').props.hidden, false, 'the ask stays shown');
  assert.deepEqual(again(at), []);
});
