// The home page drawn whole, with no browser (docs/CHECKS.md harness 5):
// twin.mjs runs its edge, home.js, its features and Preact in Node, each
// route answered from a recording in the shape the node writes (twin.mjs
// names each writer), and each law reads what the page drew and did. Each
// holds a claim the browser step's home-page tests held in Chromium
// (crates/svrngs-node/tests/browser/main.rs), named beside it; what is
// the browser's own, a first look's layout, stays there. Layout A
// (`/?ui=a`), as those tests' driver opened it (web/tests/page.mjs
// `longPage`).
//
// Plants, each watched failing in the page's own module, then put back:
// named in the commit that brought these laws.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { open, words } from './twin.mjs';

const HOME = new URL('../../../crates/svrngs-node/src/home/', import.meta.url);
const q = await import(new URL('question.js?v=__VERSION__', HOME).href);

const AT = 'http://svrngs.localhost:4000';
const [ADA, BO, CY, CY2, DEE, EVE] = ['a1', 'b2', 'c3', 'c4', 'd5', 'e6'].map((k) => k.repeat(32));
const row = (person, key, joined = { at: 1_791_000_000 }) => ({
  person,
  person_key: key,
  key,
  device: 'computer',
  joined,
});
// A lend as /v1/lends lists it (group.js `Lent`) and /lent too (home.rs
// `lent`): ada's `shelf`, kind `k`, as main.rs `lend` declares it.
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
// A group as /v1/groups answers it (surface.rs `Held`): a node with no
// seed reads no name, as main.rs `node` keeps none.
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

// browser::the_home_page_asks_in_the_one_wording: the ask heads its
// question as question.js words it, the lend named with its kind; an ask of
// an app and a lend not lent says `unlent`.
test('the ask is headed in question.js’s one wording, and one of nothing lent says so', async () => {
  const page = lend('page', { reads: ['books'] });
  const shelf = lend('shelf', { kind: 'books', asks: 'the page reads the shelf' });
  const answers = ada({ lent: [page, shelf] });
  const asked = await home(answers, `/ask?lend=${q.label(shelf)}&app=${q.label(page)}`);
  assert.equal(asked.text('#home h2')[0], q.question(page, shelf));
  assert.equal(asked.text('#home h2')[0], 'Let ada’s page (k) use ada’s shelf (books)?');
  const unlent = await home(answers, '/ask?lend=x&app=y');
  assert.equal(unlent.all('#home h2').length, 0);
  assert.equal(
    unlent.text('#home p').find((p) => p !== 'Reading your groups…'),
    q.unlent,
  );
});

// browser::the_home_page_says_its_address_as_the_command_does: the page's
// own address last, as home.rs `address(port)` writes it
// (session_tests.rs pins that to this origin and `/`).
test('the page says its own address at its foot, the origin it was opened at', async () => {
  const page = await home(ada());
  assert.deepEqual(page.text('#home > p.address'), [
    `Your home page, the same address each time svrngs starts: ${AT}/`,
  ]);
  assert.deepEqual(page.unanswered, []);
});

// browser::the_home_page_says_an_unreadable_register, and the removed
// person's words of browser::the_home_page_tells_a_removed_person: a
// register this computer cannot read, `removed: null`, is said as itself,
// never as what the group lends; removed, the person is told, and nothing
// of the group is read.
test('a register that cannot be read, and a removal, are each said, and nothing of the group read', async () => {
  for (const [removed, said] of [
    [null, 'This computer could not read who is in this group.'],
    [
      true,
      'You are no longer in this group. What you already have stays on this computer; nothing new reaches it.',
    ],
  ]) {
    const page = await home(ada({ groups: [group({ removed, removals: null, left: null })] }));
    assert.equal(page.text('#home section p')[0], said);
    assert.ok(!page.text().join().includes('lends'), page.text().join());
    assert.ok(!page.asked.some(([, path]) => path.startsWith('/v1/lends')), page.asked.join());
  }
});

// browser::the_home_page_tells_two_people_of_one_name_apart: the door
// names each person once, no two alike, each cy by when they joined, in
// the form `svrngs remove --joined` takes (home.rs WHEN, `%Y-%m-%d
// %H:%M:%S %z`), read back to the instant, and no word of the record.
test('the door tells two people of one name apart by when each joined, to the second', async () => {
  const joined = [1_791_000_000, 1_791_000_001];
  const people = [
    row('ada', ADA, 'started'),
    row('bo', BO),
    row('cy', CY, { at: joined[0] }),
    row('cy', CY2, { at: joined[1] }),
  ];
  const page = await home(ada({ people }));
  const names = page.text('#home .door strong');
  assert.equal(names.length, 4, names.join());
  assert.equal(new Set(names).size, 4, names.join());
  const WHEN = /^cy \(joined (\d{4})-(\d\d)-(\d\d) (\d\d):(\d\d):(\d\d) ([+-])(\d\d)(\d\d)\)$/;
  const read = names
    .filter((name) => name.startsWith('cy'))
    .map((name) => {
      const [, y, mo, d, h, mi, s, sign, oh, om] = name.match(WHEN) ?? assert.fail(name);
      const offset = (sign === '-' ? -1 : 1) * (Number(oh) * 60 + Number(om)) * 60;
      return Date.UTC(y, mo - 1, d, h, mi, s) / 1000 - offset;
    });
  assert.deepEqual(read.sort(), joined);
  for (const kept of ['key', 'member', 'keeper', 'ring', ' act', 'journal'])
    assert.ok(!names.join(', ').toLowerCase().includes(kept), names.join());
});

// browser::the_home_page_shows_every_removal_and_ask: each removal and
// ask from the record, by name, with when.
test('every removal and ask to remove is listed, by name, with when', async () => {
  const people = [row('ada', ADA, 'started'), row('cy', CY)];
  const at = 1_791_000_100;
  const removals = [
    {
      said: 'asked',
      by: 'cy',
      by_key: CY,
      by_joined: { at: 1 },
      of: 'dee',
      of_key: DEE,
      of_joined: { at: 2 },
      at,
    },
    {
      said: 'removed',
      by: 'ada',
      by_key: ADA,
      by_joined: 'started',
      of: 'dee',
      of_key: DEE,
      of_joined: { at: 2 },
      at: at + 5,
    },
  ];
  const page = await home(ada({ people, groups: [group({ removals })] }));
  const said = page.text('#home .removals li');
  assert.deepEqual(
    said.map((line) => line.split(' on ')[0]),
    ['cy asked to remove dee', 'ada removed dee'],
  );
  assert.deepEqual(
    said,
    removals.map((gone) => q.removal(gone, people)),
  );
});

// browser::the_home_page_says_where_the_groups_folder_is: from
// /v1/groups' `folders`, a group whose name is not read named as such.
test('where the group’s folder is on this computer is said', async () => {
  const page = await home(ada({ groups: [group({ folders: ['/home/ada/house'] })] }));
  assert.deepEqual(page.text('.folder'), [
    'The group’s folder is on this computer at /home/ada/house',
  ]);
});

// browser::the_home_page_says_a_refusal_apart_from_no_answer: each
// computer whose lends are not listed is said by how, from /v1/lends'
// `unanswered`, `unheard` and `refused` (node.rs `Unlisted`).
test('a computer that refused is said apart from one that did not answer', async () => {
  const named = (person, key) => ({ person, person_key: key, joined: { at: 1_791_000_000 } });
  const people = [
    row('ada', ADA, 'started'),
    ...[
      ['bo', BO],
      ['cy', CY],
      ['dee', DEE],
      ['eve', EVE],
    ].map(([p, k]) => row(p, k)),
  ];
  const answers = ada({ people });
  answers['GET /v1/lends?namespace=house'] = lends({
    unanswered: [named('bo', BO), named('dee', DEE)],
    unheard: [named('eve', EVE)],
    refused: [named('cy', CY)],
  });
  const page = await home(answers);
  assert.deepEqual(page.text('#home table.index td[colspan]').sort(), [
    "bo's computer did not answer, so what they lend is not listed.",
    "cy's computer answered without saying what they lend, so it is not listed.",
    "dee's computer did not answer, so what they lend is not listed.",
    "eve's computer has not heard of yours yet, so what they lend is not listed yet.",
  ]);
});

// The parts of home/index.html drawn before any script, read as served.
const INDEX = readFileSync(new URL('index.html', HOME), 'utf8');
const served = (id) =>
  [
    ...INDEX.split(`id="${id}"`)[1]
      .split(/<\/(?:header|section|footer)>/)[0]
      .matchAll(/<(p|li)[^>]*>(.*?)<\/\1>/g),
  ].map(([, , inner]) =>
    inner
      .replace(/<[^>]+>/g, '')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>'),
  );

// browser::the_home_page_says_what_it_is_for, all but June's first look:
// at its top what the page is, at its foot how it opens again; on a
// computer in no group, joining first, then making one, and nothing else.
test('the page says what it is for, and in no group offers joining, then making, alone', async () => {
  assert.equal(
    served('intro')[0],
    'This page is yours, and only this computer opens it: nobody in your groups sees it.',
  );
  assert.ok(INDEX.indexOf('id="terminal"') < INDEX.indexOf('id="again"'));
  assert.deepEqual(served('again'), [
    'To open it again, open svrngs from your Applications, or on Linux your applications menu. A bookmark of it opens it in this browser too.',
  ]);
  const page = await home(ada({ groups: [] }));
  assert.deepEqual(page.text('#home > p.none'), [
    'You’re in no group yet. Join one below from the link someone sent you, or make your own.',
  ]);
  assert.deepEqual(page.text('#home h2'), ['Join a group from a link', 'Make a group']);
  const drawn = page.text().join('\n');
  for (const gone of [
    'Open a link from your group',
    'What acts as you',
    'What you lend',
    'Already running here',
  ])
    assert.ok(!drawn.includes(gone), `${gone}: ${drawn}`);
});

// browser::the_home_page_names_what_the_terminal_does: the one line the
// page cannot do, and no line for what it can.
test('what is done in a terminal is named with its one line, and nothing the page does', () => {
  const said = served('terminal').join('\n');
  assert.ok(said.includes('svrngs remove <their name> --group <the group’s name>'), said);
  for (const done of ['leave', 'svrngs new', 'svrngs share']) assert.ok(!said.includes(done), said);
});

// browser::the_home_page_s_links_say_what_they_open and
// ::the_home_page_shows_less_to_puzzle_over: the index's link names the
// lend and its kind, what opening it does beside it, in a tab of its own;
// no WiFi or address in the row; the box's words beside it; no "What apps
// use" while nothing uses anything.
test('a lend’s link says what it opens and whose, in a tab of its own, and less to puzzle over', async () => {
  const shelf = lend('shelf');
  const page = await home(ada({ lent: [shelf] }));
  assert.deepEqual(page.text('#home table.index td a'), ['Open ada’s shelf (k)']);
  const [cell] = page.text('#home table.index td');
  assert.equal(
    cell,
    'Open ada’s shelf (k) Opening it lets it write its own part of your group’s record, as you.',
  );
  assert.ok(!/WiFi|Copy its address/.test(page.text('#home table.index td').join()));
  const [link] = page.all('#home table.index td a');
  assert.equal(link.attributes.target, '_blank');
  assert.equal(page.all('#lent label')[0].textContent, 'How many people at once: ');
  assert.ok(!page.text('#home h2').includes('What apps use'), page.text('#home h2').join());
});

// browser::the_home_page_copies_a_lend_s_address_for_a_program: below the
// group's things, the lend's address to keep, and its Copy, which writes
// that address and says where a program takes it; the group on a screen
// last. That the clipboard then holds what the page wrote is Chromium's
// (browser::the_page_offers_a_computer_one_line_and_sends_the_token_nowhere).
test('a lend’s address for a program is kept below the group, and its Copy writes it and says so', async () => {
  const shelf = lend('shelf');
  const page = await home(ada({ lent: [shelf] }));
  const foot = page.text('#home section ul.wifi li');
  assert.ok(foot[0].includes(`${shelf.origin} Bookmark this.`), foot[0]);
  const after = page.all('#home section ul.wifi')[0].nextSibling;
  assert.ok(after.textContent.endsWith(' on a screen'), after.textContent);
  await page.press('#home section ul.wifi li button', 'Copy its address for a program');
  assert.deepEqual(page.copied, [shelf.origin]);
  assert.ok(
    page
      .text('#home section ul.wifi li')[0]
      .includes(
        `Copied: ${shelf.origin}. Paste it where a program on this computer, such as a chat app, asks for an address.`,
      ),
    page.text('#home section ul.wifi li')[0],
  );
});

// browser::the_home_page_shows_a_group_before_its_slowest_computer, as an
// order, not a time: the group's people are drawn while its lends are
// still asked, bo's computer drawn off only once they answer that it did
// not.
test('a group’s people are drawn before its slowest computer answers, and it is drawn off after', async () => {
  let answer;
  const held = new Promise((resolve) => (answer = resolve));
  const people = [row('ada', ADA, 'started'), row('bo', BO)];
  const answers = ada({ people });
  answers['GET /v1/lends?namespace=house'] = () => held;
  const page = await home(answers);
  assert.deepEqual(page.text('#home .door li'), ['ada asking…', 'bo asking…']);
  assert.equal(page.all('#home .door li.off').length, 0);
  answer(lends({ unanswered: [{ person: 'bo', person_key: BO, joined: { at: 1_791_000_000 } }] }));
  await page.settle();
  assert.deepEqual(page.text('#home .door li.off'), ['bo did not answer']);
});

// browser::the_home_page_opens_a_link_only_within_its_lend (#153): a
// pasted link to a lend at a path opens that path through `/to`, in a tab
// of its own; one whose path is another host opens nothing and says so.
test('a pasted link opens only within its lend', async () => {
  const page = await home(ada());
  await page.type('#open input', 'http://page.example/#open=abc123&at=/at/a/path');
  await page.press('#open button', 'Open it');
  assert.deepEqual(page.opened, [['/to?lend=abc123&at=%2Fat%2Fa%2Fpath', '_blank']]);
  await page.type('#open input', 'http://page.example/#open=abc123&at=/\\evil.example/x');
  await page.press('#open button', 'Open it');
  assert.equal(page.opened.length, 1, JSON.stringify(page.opened));
  assert.ok(
    page.text('#open p').at(-1).endsWith('That is not a link to something lent in your groups.'),
    page.text('#open p').join('\n'),
  );
});

// browser::the_home_page_tells_two_groups_of_one_name_apart: two groups
// called Trip, headed by who made each and since when it is held here
// (group.rs `Namesake`; question.js `day`).
test('two groups of one name are headed by who made each and since when', async () => {
  const since = 1_791_000_000;
  const trip = (id, maker) => group({ id, name: 'Trip', made: { name: 'Trip', maker, since } });
  const page = await home(ada({ groups: [trip('t1', null), trip('t2', 'sam')] }));
  const heads = page.text('#home section[data-colour] > h2').sort();
  assert.deepEqual(heads, [
    `Trip (sam’s, since ${q.day(since)})`,
    `Trip (yours, since ${q.day(since)})`,
  ]);
});

// The pages step's home-page claims (crates/svrngs/tests/pages/), each
// named beside its law: what is done is said once, in place, with no
// reload. A press here never loads the page again: the twin keeps every
// `location.reload` and form the browser would have followed (`went`).

// A refusal of the node's, as home.rs says one (lend.rs `LendRefusal::say`).
const refused = (status, error) => ({ status, body: { error } });
// The words each press left at `selector`, and how often `words` is among them.
const pressed = async (page, at, holding, selector, words) => {
  await page.press(at, holding);
  return page.text(selector).join('\n').split(words).length - 1;
};

// pages::open::the_home_page_says_what_was_done_once, on layout B: made
// here, the page says so once, reads the groups once and the people
// before what they lend, and the address forgets it, so the page opened
// again there says it no more.
test('what was done here is said once, and the address forgets it', async () => {
  const answers = ada({ groups: [group({ name: 'house' })] });
  const made = await home(answers, '/?made=Trip');
  assert.ok(
    made.text('#home > p').includes('You made Trip. Open its group to invite someone.'),
    made.text('#home > p').join('\n'),
  );
  const reads = made.asked
    .map(([, path]) => path.split('?')[0])
    .filter((p) => p.startsWith('/v1/'));
  assert.equal(reads.filter((path) => path === '/v1/groups').length, 1, reads.join());
  assert.ok(reads.indexOf('/v1/rail/people') < reads.indexOf('/v1/lends'), reads.join());
  const [, to] = made.went.find(([how]) => how === 'replace') ?? assert.fail('the address kept');
  assert.ok(!to.includes('made'), to);
  const again = await home(answers, to);
  assert.ok(
    !again.text('#home > p').join().includes('You made Trip'),
    again.text('#home > p').join(),
  );
  assert.equal(again.asked.filter(([, path]) => path === '/v1/groups').length, 1);
});

// The groups /v1/groups answers, changed by a make or a join as home.rs
// `found` and `join` change them: each group's people and lends recorded,
// whichever are held now (`ada`), and its list read as it stands.
const growing = (first, more) => {
  const held = { groups: first };
  const answers = ada({ groups: [...first, ...more] });
  answers['GET /v1/groups'] = () => ({ groups: held.groups, key: ADA });
  return [answers, (g) => (held.groups = [...held.groups, g])];
};
const mine = { maker: null, since: 1_791_000_000 };
const LINK = 'svrngs://join/TRIPTOKEN';
const lab = group({ id: 'lab', name: 'Lab', person: 'Ada', made: mine, folders: ['/h/Lab'] });
const trip = group({
  id: 'trip',
  name: 'Trip',
  person: 'Dee',
  keeps: false,
  made: { maker: 'Sam', since: 1_791_000_000 },
  folders: ['/h/Trip'],
});
// Lab made, as home.rs `found` answers a make: the name, no folder
// refused, no namesakes; Trip joined, as `asked` and `join` answer.
const making = (add) => ({
  'GET /group?name=Lab': [],
  'POST /group': (body) => (add(lab), body[0] === 'Lab' ? ['Lab', null, []] : refused(409, 'no')),
  [`GET /join?${new URLSearchParams({ link: LINK })}`]: ['Trip', null],
  'POST /join': (body) => (add(trip), ['Trip', body.person, null]),
});
const make = async (page) => {
  await page.type('#make input[name=name]', 'Lab');
  await page.type('#make input[name=person]', 'Ada');
  return page.press('#make button', 'Make a group');
};
const join = async (page) => {
  await page.type('#join input[name=link]', LINK);
  await page.type('#join input[name=person]', 'Dee');
  return page.press('#join button', 'Join Trip as Dee');
};

// pages::join::a_group_made_is_drawn_with_no_reload, on layout B's make
// page: Ada makes Lab; the page says so and draws Lab, its links, having
// loaded once (home.js@19b63b2b:364 opened the page again).
test('a group made is said and drawn with its links, with no reload', async () => {
  const house = group({ name: 'house', made: mine });
  const [answers, add] = growing([house], [lab, trip]);
  const page = await home({ ...answers, ...making(add) }, '/?view=make');
  await make(page);
  assert.ok(page.text('#home > p').join().includes('You made Lab.'), page.text('#home > p').join());
  assert.ok(page.text('#home section button').includes('Invite someone to Lab'));
  assert.deepEqual(
    page.asked.filter(([how]) => how === 'POST').map(([, , body]) => body),
    [['Lab', 'Ada']],
  );
  assert.ok(
    page.lines().some((line) => line.startsWith('MAKE: made')),
    page.lines().join('\n'),
  );
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// pages::join::a_group_joined_is_drawn_with_no_reload, on layout B's join
// page: Ada pastes Sam's link to Trip and joins as Dee; the page says so
// and draws Trip, its shelf folded as just joined, having loaded once
// (home.js@19b63b2b:409 opened the page again).
test('a group joined is said and drawn, its shelf folded, with no reload', async () => {
  const house = group({ name: 'house', made: mine });
  const [answers, add] = growing([house], [lab, trip]);
  const page = await home({ ...answers, ...making(add) }, '/?view=join');
  await join(page);
  const said = page.text('#home > p').join();
  assert.ok(said.includes('You’re in Trip, as Dee.'), said);
  assert.ok(
    page.text('#home summary').includes('Bring more to Trip'),
    page.text('#home summary').join(),
  );
  assert.deepEqual(
    page.asked.filter(([how]) => how === 'POST').map(([, , body]) => body),
    [{ link: LINK, person: 'Dee' }],
  );
  assert.ok(
    page.lines().some((line) => line.startsWith('JOIN: joined')),
    page.lines().join('\n'),
  );
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// pages::join::a_group_made_or_joined_says_it_is_being_read_until_it_is
// (the operator, 2026-11-03): once drawn, every read of a group's people
// and lends is held; Lab made and Trip joined each say "Reading Lab…",
// "Reading Trip…" under its heading, while house keeps what it drew; let
// go, each is drawn read, its folder said, and none says it is read.
test('a group made or joined says it is being read until it is, and house keeps its drawing', async () => {
  const house = group({ name: 'house', made: mine, folders: ['/h/house'] });
  const [answers, add] = growing([house], [lab, trip]);
  const holds = [];
  const held = Object.fromEntries(
    Object.entries(answers)
      .filter(([key]) => /^GET \/v1\/(rail\/people|lends)\?/.test(key))
      .map(([key, body]) => [key, () => new Promise((give) => holds.push(() => give(body)))]),
  );
  const asked = { ...answers, ...making(add) };
  const page = await home(asked);
  Object.assign(asked, held);
  const section = (head) =>
    page.text('#home section[data-colour]').find((words) => words.startsWith(head)) ?? '';
  await make(page);
  const housed = section('house (yours');
  assert.ok(
    housed.includes('Invite someone to house') && !housed.includes('Reading house…'),
    housed,
  );
  assert.ok(section('Lab (yours').includes('Reading Lab…'), section('Lab'));
  await join(page);
  assert.ok(section('Trip (Sam’s').includes('Reading Trip…'), section('Trip'));
  for (const give of holds.splice(0)) give();
  await page.settle();
  assert.ok(!page.text('#home section[data-colour]').join().includes('Reading '));
  assert.ok(section('Lab (yours').includes('Lab’s folder is on this computer'), section('Lab'));
  assert.ok(section('Trip (Sam’s').includes('Trip’s folder is on this computer'), section('Trip'));
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// home_group::the_home_page_makes_a_group_another_computer_joins_with_no_restart,
// the page's half: Dee, in Lab, makes Trip; the page says so and where
// home.rs `found` placed its folder, and Invite shows the https link
// home.rs `link` answers on the page svrngs came from. Then trip, a name
// held already: said as it is typed and again once made, in question.js
// `namesakes`'s words. The node's half, that Trip is made, placed and
// joined by Bo, and the one rule traced: verbs/home.rs.
test('a group made says where its folder is and links on the page svrngs came from', async () => {
  const since = 1_791_000_000;
  const held = { name: 'Trip', maker: null, since };
  const made = group({ id: 'trip', name: 'Trip', person: 'Dee', made: { name: 'Trip', ...mine } });
  const twice = group({
    id: 'trip2',
    name: 'trip',
    person: 'Dee',
    made: { name: 'trip', ...mine },
  });
  const placed = (g) => ({ ...g, folders: [`/h/svrngs/${g.name}`] });
  const [answers, add] = growing([lab], [placed(made), placed(twice)]);
  const links = [];
  const page = await home({
    ...answers,
    'GET /group?name=Trip': [],
    'GET /group?name=trip': [held],
    'POST /group': ([name]) => (
      add(placed(name === 'Trip' ? made : twice)),
      [name, null, name === 'Trip' ? [] : [held]]
    ),
    'POST /link': (body) => (
      links.push(body),
      { client: 'svrngs://join/T', https: 'http://127.0.0.1:9/#T', device: false }
    ),
  });
  const said = () => page.text('#home > p').join('\n');
  await page.type('#make input[name=name]', 'Trip');
  await page.type('#make input[name=person]', 'Dee');
  await page.press('#make button', 'Make a group');
  assert.ok(said().includes('You made Trip.'), said());
  const section = () =>
    page.text('#home section[data-colour]').find((w) => w.startsWith('Trip (yours')) ?? '';
  assert.ok(section().includes('Trip’s folder is on this computer at /h/svrngs/Trip'), section());
  await page.press('#home section button', 'Invite someone to Trip');
  assert.deepEqual(links, [{ group: 'trip', device: false }]);
  const link = page.text('#home section p.link').find(Boolean) ?? '';
  assert.ok(
    link.startsWith('Send this link to the person you are inviting: http://127.0.0.1:9/#T '),
    link,
  );
  const day = q.day(since);
  await page.type('#make input[name=name]', 'trip');
  const before = page.text('#make p').join('\n');
  assert.ok(
    before.includes(
      `You’re already in Trip (yours, since ${day}). This makes a second group called trip.`,
    ),
    before,
  );
  await page.type('#make input[name=person]', 'Dee');
  await page.press('#make button', 'Make a group');
  assert.ok(
    said().includes(
      `You made trip. You’re already in Trip (yours, since ${day}). This is a second group called trip.`,
    ),
    said(),
  );
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// home_group::the_home_page_joins_another_group_from_its_link_while_svrngs_runs,
// the page's half: Dee, in Lab, pastes Sam's link to Trip; its button says
// what home.rs `asked` answered before it is pressed; joined, the page
// says so, heads Trip as Sam's beside Lab as Dee's own, folds Trip's
// shelf alone, as just joined (the operator, 2026-10-26), and says where
// its folder is. The node's half, the join and Sam's register:
// verbs/home.rs.
test('a group joined beside one held is headed by whose, its shelf alone folded', async () => {
  const [answers, add] = growing([lab], [trip]);
  const page = await home({ ...answers, ...making(add) });
  await page.type('#join input[name=link]', LINK);
  await page.type('#join input[name=person]', 'Dee');
  assert.ok(page.text('#join button').includes('Join Trip as Dee'), page.text('#join').join());
  await page.press('#join button', 'Join Trip as Dee');
  const said = page.text('#home > p').join('\n');
  assert.ok(said.includes('You’re in Trip, as Dee.'), said);
  const heads = page.text('#home section[data-colour] > h2');
  for (const whose of ['Trip (Sam’s', 'Lab (yours'])
    assert.ok(
      heads.some((h) => h.startsWith(whose)),
      `${whose}: ${heads.join()}`,
    );
  const folded = page.text('#home summary').filter((s) => s.startsWith('Bring more to'));
  assert.deepEqual(folded, ['Bring more to Trip']);
  const section = page.text('#home section[data-colour]').find((w) => w.startsWith('Trip (Sam’s'));
  assert.ok(section.includes('Trip’s folder is on this computer at /h/Trip'), section);
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// leave_page::leaving_on_the_home_page_is_said_as_a_leave_everywhere, the
// pages' half: Sam, keeping Trip's order of removals with Ana in it, is
// offered the hand-on in a person's words; Ana's page says before she
// leaves that this computer cannot rejoin yet, then, left (POST
// /v1/leaves, removal.rs `Outcome`), that she left, and lists Trip as
// Sam's under Groups you left; Sam's page says "Ana left on", never
// removed; Sam's new link, pasted on Ana's page, is refused in join.rs
// `NotJoined::Left`'s words, never offered as Join Trip. The node's
// half, each answer: verbs/leave.rs
// a_leave_on_the_home_page_is_held_as_a_leave_everywhere.
test('a leave is said as a leave on both pages, and a link to it again is refused', async () => {
  const people = [row('Sam', ADA, 'started'), row('Ana', EVE)];
  const trip = (more) => group({ id: 'trip', name: 'Trip', ...more });
  const sams = await home(ada({ people, groups: [trip({ person: 'Sam', made: mine })] }));
  const keeps =
    'You keep the order of removals in Trip. Choose who keeps it after you, then leave: ';
  assert.ok(
    sams.text('#home section p').some((p) => p.startsWith(keeps)),
    sams.text('#home section p').join('\n'),
  );
  const left = {
    said: 'left',
    by: 'Ana',
    by_key: EVE,
    by_joined: { at: 1 },
    of: 'Ana',
    of_key: EVE,
    of_joined: { at: 1 },
    at: 1_791_000_100,
  };
  const after = await home(
    ada({ people, groups: [trip({ person: 'Sam', made: mine, removals: [left] })] }),
  );
  const removals = after.text('#home .removals li');
  assert.ok(removals.length === 1 && removals[0].startsWith('Ana left on '), removals.join());
  assert.ok(!removals.join().includes('removed'), removals.join());

  const made = { maker: 'Sam', since: 1_791_000_000 };
  const state = { left: false };
  const refusal =
    'You’re no longer in Trip on this computer, and you can’t join it again from here yet. Join from your phone or another computer instead.';
  const answers = ada({
    people,
    groups: [trip({ person: 'Ana', keeps: false, made })],
    'POST /v1/leaves': () => ((state.left = true), { outcome: 'left' }),
    [`GET /join?${new URLSearchParams({ link: LINK })}`]: refused(409, refusal),
  });
  answers['GET /v1/groups'] = () => ({
    groups: [trip({ person: state.left ? null : 'Ana', keeps: false, made, left: state.left })],
    key: EVE,
  });
  const anas = await home(answers);
  const before = anas.text('#home section p').join('\n');
  assert.ok(
    before.includes('Rejoining needs a new invitation, and this computer can’t rejoin yet.'),
    before,
  );
  await anas.press('#home section button', 'Leave Trip');
  const said = ['#home > p', '#home section h2', '#home section p']
    .flatMap((at) => anas.text(at))
    .join('\n');
  for (const words of ['You left Trip.', 'Groups you left', 'Trip (Sam’s, since'])
    assert.ok(said.includes(words), `${words}: ${said}`);
  await anas.type('#join input[name=link]', LINK);
  const again = [...anas.text('#join p'), ...anas.text('#join button')].join('\n');
  assert.ok(again.includes(refusal), again);
  assert.ok(!again.includes('Join Trip'), again);
  assert.deepEqual([anas.went, anas.unanswered], [[], []]);
});

// leave_page::a_leave_reaches_the_leavers_other_computer, the page's
// half: three groups /v1/groups says were left are each listed under
// Groups you left by whose each is. That each reaches her other computer:
// verbs/leave.rs a_leave_reaches_the_leavers_other_computer_on_or_off.
test('every group left is listed under Groups you left, by whose it is', async () => {
  const made = { maker: 'Sam', since: 1_791_000_000 };
  const groups = ['Films', 'Trip', 'Walk'].map((name) =>
    group({ id: name, name, person: null, keeps: false, left: true, made }),
  );
  const page = await home(ada({ groups }));
  const gone = page.text('#left p');
  for (const name of ['Films', 'Trip', 'Walk'])
    assert.ok(
      gone.some((p) => p.startsWith(`${name} (Sam’s, since`)),
      `${name}: ${gone.join('\n')}`,
    );
});

// pages::rooms::play_in_one_index_is_said_on_in_every_list: cy, in ada's
// house and flat, lends `played`, a UDP port, so each index lists it with
// Play; Play pressed in house's, flat's says where to connect too, from
// PLAY's one answer (/v1/play, play.rs `Playing`); Stop in flat's, house's
// offers Play again (home.js@19b63b2b:91 drew the pressed cell alone).
test('Play in one group’s index is said on in every index, and its Stop too', async () => {
  const played = lend('played', {
    key: CY,
    person: 'cy',
    kind: 'echo',
    raw: 'udp',
    groups: ['house', 'flat'],
  });
  const on = [
    {
      person: 'cy',
      key: CY,
      name: 'played',
      port: 41000,
      said: 'Connect to 127.0.0.1:41000.',
      open: { is: 'at', words: null },
    },
  ];
  const turns = [];
  const people = [row('ada', ADA, 'started'), row('cy', CY)];
  const page = await home(
    ada({
      groups: [group({ name: 'house' }), group({ id: 'flat', name: 'flat' })],
      people,
      'GET /v1/lends?namespace=house': lends({ lent: [played] }),
      'GET /v1/lends?namespace=flat': lends({ lent: [played] }),
      'POST /v1/play': (body) => (turns.push(body), body.on ? on : []),
    }),
  );
  const cells = () =>
    page
      .all('#home section table.index td')
      .filter((td) => td.textContent.match(/(Play|Stop)$/))
      .map(words);
  assert.equal(cells().length, 2, cells().join('\n'));
  const [house, flat] = page.all('#home section table.index');
  const id = (table) => ancestorId(table);
  await page.press(`#${id(house)} table.index td button`, 'Play');
  assert.ok(
    cells().every((cell) => cell.includes('Connect to 127.0.0.1:') && cell.endsWith('Stop')),
    cells().join('\n'),
  );
  await page.press(`#${id(flat)} table.index td button`, 'Stop');
  assert.ok(
    cells().every((cell) => !cell.includes('Connect to') && cell.endsWith('Play')),
    cells().join('\n'),
  );
  assert.deepEqual(turns, [
    { on: { key: CY, name: 'played' } },
    { off: { key: CY, name: 'played' } },
  ]);
  assert.ok(page.lines().some((line) => line.startsWith('PLAY: turn: read -> turning')));
  assert.deepEqual(page.went, []);
});
const ancestorId = (el) => (el.id ? el.id : ancestorId(el.parentNode));

// pages::join::a_group_not_made_is_said_once,
// ::a_link_to_nothing_lent_is_said_once,
// ::a_link_s_copy_is_answered_once, pages::index::why_a_number_is_refused_is_said_once
// and pages::rooms::a_copied_address_is_said_once: each refusal or answer
// said once at each press, the second in the first's place.
test('a refusal or an answer is said once at each press, never one after another', async () => {
  // The node's words for a group with no name (svrngs-core lib.rs `Name`,
  // home.rs `found` and `namesakes`).
  const page = await home(
    ada({
      groups: [group({ name: 'house' })],
      lent: [lend('shelf')],
      'GET /group?name=': refused(400, 'a name cannot be empty'),
      'POST /group': refused(409, 'a name cannot be empty'),
      'POST /link': { client: 'svrngs://join/TOKEN', https: null, device: false },
    }),
  );
  const empty = 'a name cannot be empty';
  for (let n = 0; n < 2; n++)
    assert.equal(await pressed(page, '#make button', 'Make a group', '#make p', empty), 1);
  await page.type('#open input', 'http://page.example/#open=no-such');
  const nothing = 'That is not a link to something lent in your groups.';
  for (let n = 0; n < 2; n++)
    assert.equal(await pressed(page, '#open button', 'Open it', '#open p', nothing), 1);
  await page.type('#lent input', '0');
  const many = 'Say how many: 1 or more.';
  for (let n = 0; n < 2; n++)
    assert.equal(await pressed(page, '#lent button', 'Let in this many', '#lent li', many), 1);
  assert.equal(page.lines().filter((line) => line.startsWith('CHANGE: many: ')).length, 2);
  // The clipboard refuses, as the pages step's browser, granted none, did.
  navigator.clipboard.writeText = async () => Promise.reject(new Error('not allowed'));
  await page.press('#home section button', 'Invite someone to house');
  const select = ' Select it above to copy it.';
  assert.equal(page.text('#home p.link').join().split(select).length - 1, 0);
  for (let n = 0; n < 2; n++)
    assert.equal(
      await pressed(page, '#home p.link button', 'Copy the link', '#home p.link', select),
      1,
    );
  const copy = 'Copy its address for a program';
  for (let n = 0; n < 2; n++)
    assert.equal(
      await pressed(
        page,
        '#home section ul.wifi li button',
        copy,
        '#home section ul.wifi li',
        ' Copy it from here: ',
      ),
      1,
    );
  assert.ok(!page.text('#home section ul.wifi li').join().includes(' Copied: '));
  assert.deepEqual(page.went, []);
});

// pages::members::a_refused_leave_is_said_in_its_own_words and
// pages::acts::two_refusals_on_ask_are_said_once: a route refused for the
// page's lapsed session (home.rs:198, session.rs `refused`, a 401) is said
// as how to open the page again, beside its button; an ask's Yes refused
// twice is said after the buttons, the second in place of the first.
test('a lapsed session is said as how to open the page again, once, beside its button', async () => {
  const ended = { status: 401, body: undefined };
  const leaving = await home(
    ada({ groups: [group({ name: 'house', keeps: false })], 'POST /v1/leaves': ended }),
  );
  await leaving.press('#home section button', 'Leave house');
  const said =
    'house cannot be changed on this computer now: this page’s session has ended. Open svrngs from your Applications';
  assert.ok(
    leaving.text('#home .leave').join().includes(said),
    `${leaving.text('#home .leave')} ${leaving.unanswered}`,
  );
  const page = lend('page', { reads: ['books'] });
  const shelf = lend('shelf', { kind: 'books', asks: 'the page reads the shelf' });
  const asked = await home(
    ada({ lent: [page, shelf], 'POST /choices': ended }),
    `/ask?lend=${q.label(shelf)}&app=${q.label(page)}&back=/`,
  );
  const texts = [];
  for (let n = 0; n < 2; n++) {
    await asked.press('#home section button', 'Yes');
    texts.push(asked.text('#home section')[0]);
  }
  assert.ok(texts[0].split('Not now')[1].trim(), texts[0]);
  assert.equal(texts[0], texts[1]);
  assert.equal(asked.lines().filter((line) => line.startsWith('ASK: unsaid')).length, 2);
  assert.deepEqual([...leaving.went, ...asked.went], []);
});

// browser::the_home_page_says_a_room_it_cannot_read: a room.json that is
// not a list is said in the node's words (room/route.rs `turned`), never
// read as nothing open, and the lend's line still offers to open it.
test('a room this computer cannot read is said, and the line still offers to open it', async () => {
  const why =
    'This computer cannot read what you opened to this WiFi (invalid type: map, expected a sequence), so nothing is open to it now. Open a lend to this WiFi again, and it starts over from that one.';
  const page = await home(ada({ lent: [lend('shelf')], 'GET /v1/room': refused(500, why) }));
  const text = page.text().join('\n');
  assert.ok(text.includes('Open to this WiFi'), text);
  assert.ok(text.includes(why), text);
  assert.ok(page.text('#home section ul.wifi li button').includes('Open it to this WiFi'), text);
});

// pages::rooms::opened_in_one_foot_is_said_open_in_every_list: ada in
// house and flat lends `shelf` to both; opened to this WiFi in one
// group's foot, both lines say it is open, from ROOM's one answer
// (/v1/room, room.rs `InRoom`, its `said` room.rs `Served::said`), and
// closed in the other, both say it is closed.
test('opened to this WiFi in one group’s foot, every foot says so, and closed alike', async () => {
  const shelf = lend('shelf', { groups: ['house', 'flat'] });
  const at = '192.168.1.20:41234';
  const on = [
    {
      person: 'ada',
      key: ADA,
      name: 'shelf',
      served: { at },
      said: `Open to this WiFi: type http://${at}/ into an app or a browser on a device here, such as your TV, a phone or another computer. Anyone on this WiFi can open it there as you.`,
    },
  ];
  const turns = [];
  const page = await home(
    ada({
      groups: [group({ name: 'house' }), group({ id: 'flat', name: 'flat' })],
      lent: [shelf],
      'POST /v1/room': (body) => (turns.push(body), body.on ? on : []),
    }),
  );
  const row = '#home section ul.wifi li';
  assert.equal(page.all(row).length, 2);
  const [house, flat] = page.all('#home section ul.wifi');
  await page.press(`#${house.parentNode.id} ul.wifi li button`, 'Open it to this WiFi');
  assert.ok(
    page.text(row).every((line) => line.includes('Open to this WiFi: type http://')),
    page.text(row).join('\n'),
  );
  await page.press(`#${flat.parentNode.id} ul.wifi li button`, 'Close it to this WiFi');
  const shut = 'Closed to this WiFi: nobody on it can open it through this computer now.';
  assert.ok(
    page
      .text(row)
      .every((line) => line.includes(shut) && !line.includes('Open to this WiFi: type')),
    page.text(row).join('\n'),
  );
  assert.deepEqual(turns, [
    { on: { key: ADA, name: 'shelf' } },
    { off: { key: ADA, name: 'shelf' } },
  ]);
  assert.ok(page.lines().some((line) => line.startsWith('ROOM: turn: read -> turning')));
  assert.deepEqual(page.went, []);
});

// pages::members::two_folder_presses_are_said_once: "Open shared files"
// pressed twice, each answered (home.rs `open`), said opened once.
test('the shared folder opened twice is said opened once', async () => {
  const asked = [];
  const page = await home(
    ada({
      groups: [group({ name: 'house', folders: ['/home/ada/house'] })],
      'POST /folder': (body) => (asked.push(body), {}),
    }),
  );
  const button = '#home section p button';
  for (let n = 0; n < 2; n++) await page.press(button, 'Open shared files in ');
  const [said] = page.all(button).map((b) => words(b.parentNode));
  assert.ok(said.startsWith('Open shared files in Finder'), said);
  assert.ok(said.endsWith(' Opened, in a window of its own.'), said);
  assert.equal(said.split('Opened').length - 1, 1, said);
  assert.deepEqual(asked, [
    { group: 'house', dir: '/home/ada/house' },
    { group: 'house', dir: '/home/ada/house' },
  ]);
  assert.equal(page.lines().filter((line) => line.startsWith('FOLDER: opened')).length, 2);
  assert.deepEqual(page.went, []);
});
