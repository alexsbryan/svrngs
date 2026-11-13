// What the browser step's opens, play and native-arrival tests held of a
// page, with no browser (docs/CHECKS.md harness 5): the home page run
// whole by twin.mjs, each route answered from a recording in the shape its
// writer gives it (cited beside each), and the phone page's join flow
// composed and stepped as enter.test.mjs steps it. Each law names the
// browser test whose page claim it holds; what the node did, a key kept,
// a port opened, a record left alone, is claimed by the Rust tests that
// commit names.
//
// Plants, each watched failing in the page's own module, then put back:
// named in the commit that brought these laws.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { register } from 'node:module';
import { test } from 'node:test';
import { open } from './twin.mjs';

register('./layout.mjs', import.meta.url);

const AT = 'http://svrngs.localhost:4000';
const [ADA, CY] = ['a1', 'c3'].map((k) => k.repeat(32));
const row = (person, key, joined = { at: 1_791_000_000 }) => ({
  person,
  person_key: key,
  key,
  device: 'computer',
  joined,
});
// A lend as /v1/lends and /lent list it (group.js `Lent`, home.rs `lent`).
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
  ...more,
});
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

// Ada's computer in `groups`, each answer replaced by `more`.
const ada = ({
  groups = [group()],
  people = [row('ada', ADA, 'started')],
  lent = [],
  ...more
}) => ({
  'GET /v1/groups': { groups, key: ADA },
  ...Object.fromEntries(
    groups.flatMap((g) => [
      [`GET /v1/rail/people?namespace=${g.id}`, { people }],
      [`GET /v1/lends?namespace=${g.id}`, lends({ lent })],
      [`GET /publication?group=${g.id}`, { published: false }],
    ]),
  ),
  'GET /lent': { lent: [], signing: [], readers: {} },
  'GET /reaching': {},
  'GET /v1/room': [],
  'GET /v1/play': [],
  'GET /update': { state: 'current' },
  ...more,
});

// browser::play::a_member_presses_play_and_its_stop_is_listed_under_what_acts_as_you:
// cy lends `played`, a UDP port; ada's index lists it with Play, and no
// address for a program, a raw port being no page. Pressed, its cell says
// where a program connects, as /v1/play answers it (play.rs `Playing`, its
// `said` play.rs:78 `Open::said`), and that address is copied (play.js
// `copy`). Opened again with Play on, "What acts as you" lists it with its
// Stop; that Stop is the row's own: off posted, the row offers Play again
// and the line is gone. That the port is `room::port_of` of the lend's
// label, a program is answered there, and nothing listens once it is off
// are the node's: crates/svrngs/tests/play.rs.
test('Play says where to connect and copies it, and its Stop under what acts as you is the row’s own', async () => {
  const played = lend('played', { key: CY, person: 'cy', kind: 'echo', raw: 'udp' });
  const port = 41000;
  const on = [
    {
      person: 'cy',
      key: CY,
      name: 'played',
      port,
      open: { is: 'at', words: null },
      said: `Connect to 127.0.0.1:${port}.`,
    },
  ];
  const turns = [];
  const answers = (playing) =>
    ada({
      people: [row('ada', ADA, 'started'), row('cy', CY)],
      lent: [played],
      'GET /v1/play': playing,
      'POST /v1/play': (body) => (turns.push(body), body.on ? on : []),
    });
  const cells = (page) => page.text('#home table.index td').join('\n');

  const page = await open(`${AT}/?ui=a`, answers([]));
  assert.ok(cells(page).includes('Play'), cells(page));
  assert.ok(!cells(page).includes('Copy its address for a program'), cells(page));
  assert.ok(!cells(page).includes('Connect to'), cells(page));
  await page.press('#home table.index td button', 'Play');
  assert.ok(cells(page).includes(`Connect to 127.0.0.1:${port}`), cells(page));
  assert.deepEqual(page.copied, [`127.0.0.1:${port}`]);

  const again = await open(`${AT}/?ui=a`, answers(on));
  assert.ok(cells(again).includes(`Connect to 127.0.0.1:${port}`), cells(again));
  const line = 'Open on this computer: played (cy’s), while Play is on. Stop';
  assert.ok(again.text('#acts p').includes(line), again.text('#acts p').join('\n'));
  await again.press('#acts p button', 'Stop');
  assert.ok(!cells(again).includes('Connect to') && cells(again).includes('Play'), cells(again));
  assert.ok(!again.text('#acts p').includes(line), again.text('#acts p').join('\n'));
  assert.deepEqual(turns, [
    { on: { key: CY, name: 'played' } },
    { off: { key: CY, name: 'played' } },
  ]);
  assert.deepEqual([page.went, again.went, again.unanswered], [[], [], []]);
});

// How a program's file says its lender makes the account or key apps read
// it with, as share.rs `accounts_of` (:316) reads it and home.rs `lent`
// (:440) answers it, `signs.accounts`: the shipped row's words.
const PROGRAMS = new URL('../../../crates/svrngs-node/src/programs/', import.meta.url);
const accounts = (kind) =>
  readFileSync(new URL(`${kind}.txt`, PROGRAMS), 'utf8').match(
    new RegExp(`^\\d+ ${kind} accounts: (.*)$`, 'm'),
  )[1];

// Ada lends `name` of `kind`; /lent answers whether apps' account or key
// is kept as home.rs `lent` says it (:439-444), kept once a POST /lent
// takes it (home.rs `change`, 204 as tests/home.rs's right answer).
const reading = (name, kind, key) => {
  const lent = lend(name, { kind, limit: null });
  const kept = { now: false };
  const readers = () => ({
    [name]: { kept: kept.now, signs: { accounts: accounts(kind) }, key, by: '' },
  });
  return ada({
    lent: [lent],
    'GET /lent': () => ({ lent: [lent], signing: [], readers: readers() }),
    'POST /lent': () => ((kept.now = true), { status: 204, body: undefined }),
  });
};
// Every request the page sent holding `secret`, by method and path.
const holding = (page, secret) =>
  page.asked
    .filter(([, path, body]) => `${path} ${JSON.stringify(body ?? '')}`.includes(secret))
    .map(([method, path]) => `${method} ${path}`);

// browser::opens::the_apps_account_is_made_from_the_home_page: the row
// says apps can't read jellyfin and how to make the account, from its
// program's file; the name and password typed and saved, it says what the
// account does; the password is sent once, to this computer's /lent alone.
// That it is signed in with once, kept as `MediaBrowser Token="tok-1"` and
// the password kept nowhere: tests/home.rs
// the_apps_account_is_given_on_the_home_page.
test('the apps’ account is made on the home page, and its password sent once, to /lent alone', async () => {
  const page = await open(`${AT}/?ui=a`, reading('jellyfin', 'jellyfin', false));
  const before = page.text('#lent li').join('\n');
  assert.ok(
    before.includes(
      'Apps can’t read your jellyfin yet. To let them, make one account for your group.',
    ),
    before,
  );
  assert.ok(before.includes('Dashboard, Users'), before);
  await page.type('#lent li input[name=account]', 'films-group');
  await page.type('#lent li input[name=password]', 'pw-1');
  await page.press('#lent li button', 'Save the account');
  const after = page.text('#lent li').join('\n');
  assert.ok(
    after.includes(
      'Apps your friends open can read your jellyfin with the account you made. They can’t change anything.',
    ),
    after,
  );
  assert.deepEqual(holding(page, 'pw-1'), ['POST /lent']);
  const [, , sent] = page.asked.find(([method]) => method === 'POST');
  assert.deepEqual(sent, {
    lend: 'jellyfin',
    to: { readers: { name: 'films-group', password: 'pw-1' } },
  });
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// browser::opens::a_readers_key_is_pasted_on_the_home_page: the row says
// apps can't read immich and how to make a key, "New API Key" from its
// program's file; pasted and saved, it says what the key does; the key is
// sent once, to /lent alone. That it is tried, kept as
// secrets/lends/immich/readers/x-api-key and never told:
// tests/home.rs a_readers_key_is_pasted_on_the_home_page_and_never_told.
test('a readers’ key is pasted on the home page, and sent once, to /lent alone', async () => {
  const page = await open(`${AT}/?ui=a`, reading('immich', 'immich', true));
  const before = page.text('#lent li').join('\n');
  assert.ok(
    before.includes(
      'Apps can’t read your immich yet. To let them, make a key in immich and paste it here.',
    ),
    before,
  );
  assert.ok(before.includes('New API Key'), before);
  await page.type('#lent li input[name=password]', 'pasted-key-1');
  await page.press('#lent li button', 'Save the key');
  const after = page.text('#lent li').join('\n');
  assert.ok(
    after.includes(
      'Apps your friends open can read your immich with the key you made. They can’t change anything.',
    ),
    after,
  );
  assert.deepEqual(holding(page, 'pasted-key-1'), ['POST /lent']);
  const [, , sent] = page.asked.find(([method]) => method === 'POST');
  assert.deepEqual(sent, { lend: 'immich', to: { key: 'pasted-key-1' } });
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// browser::native_arrival::native_arrival_names_once_and_shows_the_actual_folder,
// the phone page's half: a link's `&native` (the resolver's 303,
// native-arrival.test.mjs) is asked of the issuer as a native welcome
// (join.js `asking`, `/ask?native=true`), named once, and offered as a
// line to run; no step admits the browser (join.js `admit`, the redeem)
// or keeps a group in it (enter.js `keep`), whatever it is asked again.
// That the issuer's native ask writes nothing is redeem.rs `ask_route`
// (:146), which reads (`serve_ask`, :279) and commits nothing.
test('native arrival asks the issuer, names once, offers a line, and never admits or keeps the browser', async () => {
  const { compose } = await import('../../page/page.js?v=__VERSION__');
  const flows = Object.fromEntries(
    await Promise.all(
      ['enter', 'join', 'computer'].map(async (name) => [
        name,
        await import(`../../page/${name}.js?v=__VERSION__`),
      ]),
    ),
  );
  const parts = compose(flows);
  const token = 'ordinary_invitation';
  const runs = [];
  const step = (state, message) => {
    const next = parts.step(state, message);
    runs.push(...next.effects.map((e) => (e.run === 'send' ? `send ${e.m.type}` : e.run)));
    return next;
  };
  const looked = step(parts.first({ fine: true }), {
    type: 'looked',
    fragment: `${token}&native`,
    kept: [],
    linked: true,
  });
  const joining = step(looked.state, looked.effects[0].m);
  assert.deepEqual(joining.effects, [{ run: 'ask', token, native: true, failed: 'unjoined' }]);
  let { state } = step(joining.state, {
    type: 'asked',
    told: 'issuer-answer',
    group: { id: 'abcd', name: 'house', maker: 'ada', inviter: 'ada' },
  });
  assert.deepEqual(parts.heading(state), ['ada invited you to house']);
  // #join is shown while it names (name-form.js:42, join.js:93 `asks`),
  // #computer only once a line is offered (computer-line.js:31).
  assert.deepEqual([state.at.join, state.line], ['naming', undefined]);
  state = step(state, { type: 'typed', field: 'name', value: "Bo O'Brien" }).state;
  const named = step(state, { type: 'named' });
  const offer = step(named.state, named.effects[0].m);
  state = step(offer.state, {
    type: 'offered',
    ...offer.effects[0],
    page: 'https://groups.example/',
  }).state;
  assert.equal(state.at.join, 'installing');
  assert.ok(state.line[1].startsWith('curl -fsSL https://groups.example/install.sh'));
  assert.ok(state.line[1].includes("--at '/#group-abcd' --as 'Bo O'\\''Brien'"), state.line[1]);
  assert.equal(step(state, { type: 'named' }).effects.length, 0);
  assert.deepEqual(runs, ['send join', 'ask', 'send offer', 'where']);
  assert.deepEqual(state.joining.kept, []);
});

// The native arrival's home page, as native-arrival.mjs fed it: ada's
// /v1/groups (surface.rs `Held`) holding house, and beside it a group of
// house's name with a folder of its own and another group.
const FOLDER = '/home/ada/already-used-name-2';
const arrived = (target = {}, how = {}) => {
  const house = group({ id: 'h1', name: 'house', folders: [FOLDER], ...target });
  const groups = [
    house,
    group({ id: 'same-name', name: 'house', folders: ['/wrong-folder'] }),
    group({ id: 'other', name: 'Other group', folders: ['/other-folder'] }),
  ].filter((g) => !how.missing || g.id !== 'h1');
  const folders = [];
  return [
    ada({
      groups,
      'GET /v1/groups': how.groups ?? { groups, key: ADA },
      'POST /folder': (body) => (folders.push(body), {}),
    }),
    folders,
  ];
};
const sections = (page) => page.all('#home section').filter((s) => s.id.startsWith('group-'));

// browser::native_arrival::native_arrival_names_once_and_shows_the_actual_folder,
// the home page's half: opened at `/#group-<id>`, as the line's `--at`
// opens it, it welcomes the person to that group alone, with its own
// folder, never the one of a group of its name; its button opens that
// folder; the address keeps the group, so opened again it welcomes again;
// "All your groups" leads to every group, which a look back at the tab
// leaves so; and while the groups are read, or cannot be, or the group is
// gone, or its folder unknown or none, it says so and never shows the
// other house.
test('a native arrival welcomes to its group alone, with its folder, until All your groups', async () => {
  const [answers, folders] = arrived();
  const page = await open(`${AT}/#group-h1`, answers);
  assert.deepEqual(
    sections(page).map((s) => s.id),
    ['group-h1'],
  );
  assert.deepEqual(page.text('#home h2'), ['Welcome to house']);
  assert.ok(!page.text().join('\n').includes('Invite people here'));
  assert.ok(page.text('#group-h1 .folder')[0].includes(FOLDER), page.text('.folder').join());
  const opener = page.text('#group-h1 button').filter((b) => b.startsWith('Open shared files in '));
  assert.deepEqual(opener, ['Open shared files in Finder']);
  await page.press('#group-h1 button', 'Open shared files in');
  assert.deepEqual(folders, [{ group: 'h1', dir: FOLDER }]);
  assert.deepEqual(page.went, [], 'the address keeps #group-h1');

  const again = await open(`${AT}/#group-h1`, answers);
  assert.equal(sections(again).length, 1, 'opened again, it welcomes again');
  await again.press('#home a', 'All your groups');
  const [[how, to]] = again.went;
  assert.deepEqual([how, to], ['link', '/']);
  const all = await open(new URL(to, AT).href, answers);
  assert.equal(all.all('.grouped-nav').length, 1);
  assert.equal(sections(all).length, 3);
  await all.shown();
  assert.equal(sections(all).length, 3, 'a look back never welcomes again');
  assert.deepEqual(all.went, []);
});

test('a native arrival says why it cannot welcome, and never shows a group of its name instead', async () => {
  let release;
  const held = new Promise((resolve) => (release = resolve));
  const [loading] = arrived({}, { groups: () => held });
  const unread = arrived({}, { groups: { status: 503, body: undefined } })[0];
  for (const [answers, said] of [
    [loading, 'Reading your groups…'],
    [unread, 'cannot read your groups'],
    [arrived({}, { missing: true })[0], 'not available on this computer'],
    [arrived({ folders: null })[0], 'cannot say where'],
    [arrived({ folders: [] })[0], 'no shared folder'],
  ]) {
    const page = await open(`${AT}/#group-h1`, answers);
    assert.ok(page.text().join('\n').includes(said), `${said}: ${page.text().join('\n')}`);
    assert.equal(page.all('#group-same-name').length, 0, said);
    if (answers === loading) {
      release(arrived()[0]['GET /v1/groups']);
      await page.settle();
      assert.ok(page.text('#group-h1 .folder')[0].includes(FOLDER), page.text().join('\n'));
    }
  }
});

// browser::acts::the_home_page_says_what_acts_as_you_and_its_stop_holds:
// cy lends notes, reading films, and films; ada's computer carries Money;
// ada opened the notes, her yes, today, as /reaching answers it (home.rs
// `reached`, door/choices.rs `Choice`: said, at, the lends it named).
// "What acts as you" is folded under its heading, and says, part by part,
// what Money does, the notes opening the films she chose since today with
// a Stop and that a Stop holds from the app's next request, that what she
// opens knows her name, and that only this computer decides for her. Its
// Stop is the home page's form, posting the notes' label and `stop` to
// /reaching; opened again, the line says the notes stopped. That the Stop
// holds from the notes' next request, `unasked` in its words, is
// tests/surface.rs a_stop_on_the_home_page_holds_from_the_app_s_next_request.
test('what acts as you is said part by part, and its Stop is posted and said', async () => {
  const q = await import('../../../crates/svrngs-node/src/home/question.js?v=__VERSION__');
  const SHELF = new URL('../../../crates/svrngs-node/src/shelf.txt', import.meta.url);
  // The shelf's words for Money, as /v1/lends's `shelf` carries them.
  const [, , , said] = readFileSync(SHELF, 'utf8')
    .split('\n')
    .find((line) => line.startsWith('Money\t'))
    .split('\t');
  const notes = lend('notes', { key: CY, person: 'cy', kind: 'notes', reads: ['films'] });
  const films = lend('films', { key: CY, person: 'cy', kind: 'films' });
  const money = lend('Money0house', { kind: 'Money', group: 'house', limit: null });
  const at = Math.floor(Date.now() / 1000) - 60;
  const choice = (word) => ({ [q.label(notes)]: { said: word, at, lends: [q.label(films)] } });
  const answers = (word) => {
    const all = ada({
      people: [row('ada', ADA, 'started'), row('cy', CY)],
      lent: [money, notes, films],
      'GET /reaching': choice(word),
    });
    all['GET /v1/lends?namespace=house'] = lends({
      lent: [money, notes, films],
      shelf: [{ name: 'Money', words: said }],
    });
    return all;
  };
  const page = await open(`${AT}/?ui=a`, answers('yes'));
  const [details] = page.all('#acts details');
  assert.ok(details && !('open' in details.attributes), 'folded under its heading');
  const section = page.text('#acts')[0];
  const today = q.day(at);
  for (const part of [
    'What acts as you',
    'Reads and writes for you',
    'Money reads and writes who owes whom, and for what, when you open it.',
    'Opens friends’ things as you',
    'Each Stop holds from the app’s next request.',
    `cy’s notes opens the films you chose, as you. Since ${today}.`,
    'Knows who you are',
    'What you open knows your name.',
    'Can decide for you',
    'Your group: This computer.',
  ])
    assert.ok(section.includes(part), `${part}: ${section}`);
  const line = page.all('#acts p').find((p) => p.textContent.includes('as you. Since'));
  await page.press('#acts p form button', 'Stop');
  const [[how]] = page.went;
  assert.equal(how, 'submit');
  const form = line.childNodes.find((child) => child.localName === 'form');
  const posted = Object.fromEntries(
    form.childNodes
      .filter((input) => input.localName === 'input')
      .map((input) => [input.name ?? input.attributes.name, input.value]),
  );
  assert.deepEqual([form.attributes.action, form.attributes.method], ['/reaching', 'post']);
  assert.deepEqual([posted.app, posted.said], [q.label(notes), 'stop']);
  const again = await open(`${AT}/?ui=a`, answers('stop'));
  const stopped = again.text('#acts')[0];
  assert.ok(
    stopped.includes(`cy’s notes stopped opening your friends’ films as you on ${today}.`),
    stopped,
  );
  assert.deepEqual(again.unanswered, []);
});

// browser::guests::every_home_page_shows_sam_visiting_and_kims_stop_ends_it:
// Kim hosts Sam until a day 40 hours from now; each computer's /v1/groups
// lists the visit (rail.rs `visiting`: id, guest, host, host_key, until,
// yours). Bo's home page lists Sam visiting, Kim's guest, until that day;
// Kim's lists Sam under her own guests with a Stop that says what it does,
// and visiting; pressed, the Stop posts the group and the visit to
// /visits, and Kim's page, read again, lists nobody, as Bo's does once
// /v1/groups lists nobody. That each computer lists it, that the Stop is
// in Kim's record and Bo's computer then lists nobody: tests/surface.rs
// a_visit_is_listed_on_every_computer_and_its_host_s_stop_ends_it_on_each.
const KIM = 'f7'.repeat(32);
const sams = (yours) => {
  const until = Math.floor(Date.now() / 1000) + 40 * 60 * 60;
  const visit = { id: 'v1', guest: 'Sam', host: 'Kim', host_key: KIM, until, yours };
  return [visit, new Date(until * 1000).toLocaleDateString('en-US', { weekday: 'long' })];
};
const visited = (me, visiting, more = {}) => {
  const people = [row('Kim', KIM, me === KIM ? 'started' : undefined), row('bo', ADA)];
  const g = group({ name: 'house', person: me === KIM ? 'Kim' : 'bo', visiting });
  const answers = ada({ groups: [g], people, ...more });
  answers['GET /v1/groups'] = more['GET /v1/groups'] ?? { groups: [g], key: me };
  return answers;
};
// Who a home page lists as its own guests, then as visiting (visits.js).
const listed = (page) => [...page.text('#home .yours li'), ...page.text('#home .visiting li')];
test('every home page lists Sam visiting, and Kim’s Stop posts it and lists nobody', async () => {
  const [bos, day] = sams(false);
  const bo = await open(`${AT}/?ui=a`, visited(ADA, [bos]));
  const onBos = listed(bo);
  assert.equal(onBos.length, 1, onBos.join('\n'));
  assert.ok(onBos[0].startsWith(`Sam, Kim’s guest, until ${day}`), onBos[0]);

  const [kims] = sams(true);
  const stopped = [];
  const listing = { now: [kims] };
  const answers = visited(KIM, [kims], {
    'POST /visits': (body) => (stopped.push(body), (listing.now = []), {}),
  });
  const g = answers['GET /v1/groups'].groups[0];
  answers['GET /v1/groups'] = () => ({ groups: [{ ...g, visiting: listing.now }], key: KIM });
  const kim = await open(`${AT}/?ui=a`, answers);
  const onKims = listed(kim);
  assert.equal(onKims.length, 2, onKims.join('\n'));
  assert.ok(onKims[0].startsWith(`Sam, until ${day}`), onKims[0]);
  assert.ok(onKims[0].includes('Stop ends the visit now'), onKims[0]);
  assert.ok(onKims[1].startsWith(`Sam, Kim’s guest, until ${day}`), onKims[1]);
  await kim.press('#home .yours button', 'Stop');
  assert.deepEqual(stopped, [['house', 'v1']]);
  assert.deepEqual(listed(kim), [], 'read again, with no reload');
  assert.deepEqual(kim.went, []);
  const again = await open(`${AT}/?ui=a`, answers);
  assert.deepEqual(listed(again), []);
  const boAgain = await open(`${AT}/?ui=a`, visited(ADA, []));
  assert.deepEqual(listed(boAgain), []);
});

// browser::guests::the_phones_page_shows_sam_visiting_and_kims_stop_ends_it:
// Ana's phone reads house through Bo's computer, whose /people/lends
// answers who is visiting (node.rs:1029, rail.rs `visiting`); the page
// lists Sam visiting, Kim's guest, until that day, and once a read after
// Kim's Stop answers nobody, lists nobody. That Bo's computer answers so
// before and after the Stop it synced: tests/surface.rs
// a_visit_is_listed_on_every_computer_and_its_host_s_stop_ends_it_on_each.
test('the phone’s page lists Sam visiting as Bo’s computer answers, and nobody once it answers none', async () => {
  const { compose } = await import('../../page/page.js?v=__VERSION__');
  const { VisitingNow } = await import('../../page/visiting-now.js?v=__VERSION__');
  const flows = [
    'enter',
    'computer',
    'join',
    'ask',
    'index',
    'door',
    'shelf',
    'visit',
    'among',
    'acts',
    'uses',
  ];
  const parts = compose(
    Object.fromEntries(
      await Promise.all(
        flows.map(async (f) => [f, await import(`../../page/${f}.js?v=__VERSION__`)]),
      ),
    ),
  );
  const house = { id: 'g1', name: 'house', person: 'Ana', token: 'TOKEN', addresses: [] };
  const [sam, day] = sams(false);
  const people = [row('Kim', KIM), row('bo', ADA), row('Ana', 'aa'.repeat(32))];
  const body = (visiting) => ({
    version: 5,
    lent: [],
    unanswered: [],
    unheard: [],
    refused: [],
    older: [],
    removals: [],
    visiting,
    people,
    shelf: [],
    changed: [],
  });
  const bo = { key: ADA, person: 'bo', relay: null };
  const through = (s, ...ms) =>
    ms.reduce((at, m) => {
      const r = parts.step(at, m);
      assert.equal(r.refused, undefined, `${m.type}: ${r.refused}`);
      return r.state;
    }, s);
  // What a person reads of a drawing: its words, function parts drawn.
  const words = (v) => {
    if (v == null || typeof v === 'boolean') return '';
    if (Array.isArray(v)) return v.map(words).join('');
    if (typeof v !== 'object') return String(v);
    if (typeof v.type === 'function') return words(v.type(v.props));
    return words(v.props.children);
  };
  const read = through(
    parts.first({ fine: false }),
    { type: 'hold', group: house },
    { type: 'held' },
    { type: 'opened', group: house },
    { type: 'read', n: 1, ok: true, via: bo, body: body([sam]), lent: [] },
  );
  const drawn = words(VisitingNow({ state: read }));
  assert.ok(drawn.includes(`Sam, Kim’s guest, until ${day}`), drawn);
  const again = through(
    read,
    { type: 'reread' },
    { type: 'read', n: 2, ok: true, via: bo, body: body([]), lent: [] },
  );
  assert.equal(words(VisitingNow({ state: again })), '');
});
