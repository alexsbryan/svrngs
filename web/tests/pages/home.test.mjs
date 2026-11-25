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
import { open, shown, words } from './twin.mjs';

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
  share = null,
  ...more
} = {}) => ({
  'GET /v1/groups': { groups, key: ADA, share },
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
  'GET /v1/direct': { on: false, said: 'Direct connections are off for this WiFi.' },
  'GET /v1/play': [],
  'GET /update': { state: 'current' },
  ...more,
});

const home = (answers, path = '/?ui=a') => open(`${AT}${path}`, answers);

// What a person reads of the page: its words less what B hides (`hidden`,
// grouped.js), each run of space one.
const read = (page) => {
  const seen = (at) =>
    at.attributes?.hidden === undefined ? (at.data ?? at.childNodes.map(seen).join(' ')) : '';
  return seen(page.root).replace(/\s+/g, ' ').trim();
};
// The page reloaded after a write: opened again at the address it holds,
// the last it put in place of the one it was opened at (`replaceState`),
// else that one. It reads as the page did, less what was done, which is
// said once (home.js `kept`).
const reloaded = async (page, opened, answers) => {
  const said = page.text('#home > p').find((p) => p.startsWith('You')) ?? '';
  const was = read(page).replace(said, '').replace(/\s+/g, ' ').trim();
  const to = page.went.filter(([how]) => how === 'replace').at(-1)?.[1] ?? opened;
  const again = await home(answers, to);
  assert.equal(read(again), was, `reloaded at ${to}`);
  return again;
};
// Where layout A keeps the address after a write there: `/?ui=a`, the
// whole page (home.js `kept`).
const KEPT_A = ['replace', '/?ui=a'];

// Where a thing runs, so when it opens, said before what opening it lets it
// do: of an app the group carries, and of what a person lends. The same
// words for one thing and for many. Planted, watched failing: index.js
// `says` saying of a lend what is true of a carried app.
const CARRIED =
  'What is here is in your group’s folder on every computer in your group, and opens while any of them is on. ';
const LENT =
  'What is here runs on the computer of whoever lends it, and opens only while that computer is on. ';

// The index's cells, each by its words less its popover's (wifi.js
// `Address`): what a person reads in the cell before pressing Address.
const cells = (page) => page.all('#home .index > ul > li').map(shown);

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

// A release whose signature is refused (update.rs `Unsigned`, its words),
// as home.rs `update_check` answers it, `held`: said where the update would
// be offered, as the page opens, with nothing to press. Planted, watched
// failing: update-line.js SHOWN without `held` (nothing drawn); update.js
// reading `held` as unread (nothing drawn).
test('a release whose signature is refused is said where the update would be offered', async () => {
  const why =
    'No update: group.svrnme.sh offers a new svrngs without the svrngs release key’s signature (get/SHA256SUMS.sig), so it can’t be told from anyone’s. Whoever runs that page can add the signature from the release.';
  const page = await home(ada({ 'GET /update': { state: 'held', why } }));
  assert.deepEqual(page.text('#update p'), [why]);
  assert.deepEqual(page.all('#update button'), []);
});

// The same words when the page changed between the check and the press:
// the update refused says them once, as one sentence. Planted, watched
// failing: update-line.js wrapping the why as given ("…from the release..").
test('an update refused for its signature says the why as one sentence', async () => {
  const why =
    'No update: group.svrnme.sh offers a new svrngs without the svrngs release key’s signature (get/SHA256SUMS.sig), so it can’t be told from anyone’s. Whoever runs that page can add the signature from the release.';
  const page = await home(
    ada({ 'GET /update': { state: 'available' }, 'POST /update': refused(502, why) }),
  );
  await page.press('#update button', 'Update and restart');
  assert.equal(page.text('#update p')[0], `The update did not finish: ${why}`);
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

// The share's line (share-refusal; RELAY_COMMONS design 3, "The words"):
// while /v1/groups says `share` — the unix seconds the door last refused a
// large answer over the public way, and only while its share is spent
// (surface.rs) — the page says the day, and that large things wait until
// this computer reconnects to it; with no share, nothing. Planted, watched
// failing: `shareSaid` answering the words whatever `share` says, where
// the quiet page carried them.
test('the lender is told the day its share was used up, while it is spent', async () => {
  const at = 1_791_000_000;
  const spent = await home(ada({ share: at }));
  assert.deepEqual(
    spent.text('#home > p').filter((p) => p.includes('share of the public way')),
    [
      `This computer's share of the public way was used up on ${q.day(at)}; large things you lend wait until it connects afresh.`,
    ],
  );
  const quiet = await home(ada());
  assert.deepEqual(
    quiet.text('#home > p').filter((p) => p.includes('share of the public way')),
    [],
    'nothing while the share is not spent',
  );
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

// A person in the door is pressed: the name is a button, named with how
// they stand, that opens the browser's popover by its id, saying only what
// the page holds of them. Someone on: how they stand, when they joined, and
// what they share here, each a jump to that thing's way in under its
// caption (`#at-…`, the id of the link, the yes's button or the Play), never
// a second way in: no link of the popover opens anything. Someone whose
// computer did not answer: that what they lend is not listed. You: that it
// is you, and nothing more. Planted, watched failing: people.js `Person`'s
// link going to `/to` (a second way in).
const PEOPLE = [
  row('ada', ADA, 'started'),
  row('bo', BO),
  row('cy', CY),
  row('eve', EVE, { at: 1_791_000_001 }),
  row('eve', CY2, { at: 1_791_000_002 }),
];
// House with those people, ada keeping its removals or not, bo lending
// three things, cy's computer silent, and what a removal is answered.
const door = async (keeps, removal) => {
  const theirs = { key: BO, person: 'bo' };
  const lent = [
    lend('shelf', theirs),
    lend('page', { ...theirs, reads: ['films'] }),
    lend('game', { ...theirs, raw: 'udp' }),
    lend('mine'),
  ];
  const answers = ada({ people: PEOPLE, groups: [group({ name: 'House', keeps })] });
  const silent = [{ person: 'cy', person_key: CY }];
  answers['GET /v1/lends?namespace=house'] = lends({ lent, unanswered: silent });
  answers['POST /v1/removals'] = removal;
  return Object.assign(await home(answers), { answers });
};
const said = (pop) => pop.childNodes.filter((kid) => kid.nodeType === 1).map(words);

test('a person in the door is pressed for what the page holds of them', async () => {
  const page = await door(true);
  const buttons = page.all('#home .door > li > button');
  const pops = page.all('#home .door > li > div');
  assert.deepEqual(buttons.map((button) => button.attributes['aria-label']).slice(0, 3), [
    'ada, on now',
    'bo, on now',
    'cy, did not answer',
  ]);
  assert.deepEqual(
    buttons.map((button) => button.attributes.popovertarget),
    pops.map((pop) => pop.id),
  );
  for (const [at, pop] of pops.entries()) {
    assert.deepEqual(
      [pop.attributes.popover, pop.attributes.role, pop.attributes['aria-label']],
      ['auto', 'dialog', buttons[at].attributes['aria-label']],
    );
    const close = pop.childNodes.filter((kid) => kid.nodeType === 1).at(-1);
    assert.deepEqual(
      [words(close), close.attributes.popovertarget, close.attributes.popovertargetaction],
      ['Close', pop.id, 'hide'],
    );
  }
  const [you, bo, cy] = pops;
  assert.deepEqual(said(you), ['This is you.', 'Close']);
  assert.deepEqual(said(bo).slice(0, 3), [
    'bo, on now, joined 3 Oct.',
    'Shared here:',
    'shelfpagegame',
  ]);
  // Each thing shared is a jump to its own way in, in the index, under its
  // caption; the popover holds no way in of its own.
  const links = page.all('#home .door > li > div a');
  const ways = links.map((link) =>
    page.all('#home .index *').find((at) => `#${at.id}` === link.attributes.href),
  );
  assert.deepEqual(
    links.map((link) => [words(link), link.attributes.target]),
    [
      ['shelf', undefined],
      ['page', undefined],
      ['game', undefined],
    ],
  );
  assert.deepEqual(
    ways.map((way) => [way.localName, way.attributes['aria-label'] ?? words(way)]),
    [
      ['a', 'bo’s shelf (k)'],
      ['button', 'Open bo’s page (k)'],
      ['button', 'Play bo’s game'],
    ],
  );
  for (const way of ways) assert.ok(way.attributes['aria-describedby'], 'under its caption');
  assert.equal(page.all('#home .door form').length, 0);
  assert.deepEqual(said(cy).slice(0, 2), [
    'cy, did not answer, joined 3 Oct.',
    'cy’s computer did not answer, so what they lend is not listed.',
  ]);
});

// Removing someone is two presses in their popover, never one: a fold,
// closed, whose summary names the person and the group, and in it what it
// does and the button. The press asks the node what `svrngs remove` asks
// (removal.rs `RemovalAsk`): the group, the person, and when they joined,
// so of two people of one name the one pressed, never a guess. Whoever
// keeps the removals removes; anyone else asks the keeper, and the words
// say which. Nothing of it is drawn for yourself. Each answer the route
// can give is said in the popover, in a line a screen reader is told of,
// and a removal or an ask reads the groups again. Planted, each watched
// failing: the body sent without `joined`; the fold drawn open; the fold
// drawn for yourself; leaving.js saying nothing of `not_in_group`; and
// ui/door.js giving a person's cell no key, so that the person after the
// one removed took it, its fold open.
test('removing a person is two presses in their popover, and every answer is said', async () => {
  const asked = [];
  const answer =
    (outcome, status = 200) =>
    (body) => {
      asked.push(body);
      return { status, body: { outcome } };
    };
  const fold = (pop) => pop.childNodes.find((kid) => kid.localName === 'details');
  const pressed = async (keeps, name, removal) => {
    const page = await door(keeps, removal);
    const pop = page.all('#home .door > li > div').find((at) => words(at).startsWith(name));
    assert.equal(fold(pop).attributes.open, undefined, 'the button waits in a closed fold');
    const reads = () => page.lines().filter((line) => line.startsWith('HOME: again')).length;
    const before = reads();
    await page.press(`#${pop.id} details button`);
    const status = pop.childNodes.find((kid) => kid.attributes?.role === 'status');
    return [said(fold(pop)), words(status), reads() - before, page.text('#home section > p')];
  };
  const [keeper, out, again, byDoor] = await pressed(true, 'bo', answer('out'));
  // Said by the door too, since the popover goes with the cell.
  const is =
    'bo is no longer in House. What they already have stays with them; nothing new reaches them.';
  assert.ok(byDoor.includes(is), byDoor.join('\n'));
  assert.deepEqual(keeper, [
    'Remove bo from House',
    'This puts bo out of House on every computer and phone of theirs. What they already have stays with them; nothing new reaches them. This page cannot undo it.',
    'Remove bo',
  ]);
  assert.deepEqual([out, again], ['', 1], 'removed: the groups are read again, which draw it');
  assert.deepEqual(asked.pop(), { group: 'house', person: 'bo', joined: { at: 1_791_000_000 } });
  // Of two people of one name, the one pressed, by when they joined.
  await pressed(true, 'eve (joined', answer('out'));
  assert.deepEqual(asked.pop(), { group: 'house', person: 'eve', joined: { at: 1_791_000_001 } });
  // Who decides is named when the node names them.
  const named = (body) => (asked.push(body), { outcome: 'asked', decides: 'cy' });
  assert.equal(
    (await pressed(false, 'bo', named))[1],
    'You asked cy to remove bo from House. bo stays in it until cy does.',
  );
  const [other, asking, reread] = await pressed(false, 'bo', answer('asked'));
  assert.deepEqual(other, [
    'Ask to remove bo from House',
    'This asks whoever decides who is removed from House. bo stays in it until they do.',
    'Ask to remove bo',
  ]);
  assert.deepEqual(
    [asking, reread],
    [
      'You asked whoever decides who is removed from House to remove bo from House. bo stays in it until whoever decides who is removed from House does.',
      1,
    ],
  );
  const SAID = {
    decides:
      'bo decides who is removed from House, so cannot be removed until they hand that on, under Manage this group on their own page.',
    uncounted: 'House’s record would not count that from this computer, so nothing changed.',
    not_in_group: 'Nobody called bo is in House.',
    which:
      'This computer cannot tell which person called bo in House you mean, so nothing changed.',
    not_in: 'You are no longer in House.',
    older: 'House cannot be changed on this computer now.',
    fault: 'House cannot be changed on this computer now.',
  };
  for (const [outcome, words] of Object.entries(SAID))
    assert.deepEqual(
      (await pressed(false, 'bo', answer(outcome, 409))).slice(1, 3),
      [words, 0],
      outcome,
    );
  const failed = await pressed(true, 'bo', { status: 503, body: { error: 'off' } });
  assert.equal(failed[1], 'House cannot be changed on this computer now: off');
  // Removed, bo's cell is the next person's, and nothing of bo's popover is
  // theirs: no fold left open for a press meant for bo to land on.
  const gone = await door(true, (body) => {
    gone.answers['GET /v1/rail/people?namespace=house'] = {
      people: PEOPLE.filter((person) => person.person !== body.person),
    };
    return { outcome: 'out' };
  });
  const mine = gone.all('#home .door > li > div').find((at) => words(at).startsWith('bo'));
  fold(mine).attributes.open = '';
  await gone.press(`#${mine.id} details button`);
  await gone.shown();
  assert.deepEqual(
    gone.all('#home .door > li > button').map((button) => button.attributes['aria-label'])[1],
    'cy, did not answer',
  );
  assert.equal(gone.all('#home .door details').filter((at) => at.attributes.open === '').length, 0);
  // Yourself: no fold, so no button.
  const page = await door(true);
  const [you] = page.all('#home .door > li > div');
  assert.equal(fold(you), undefined);
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

// browser::the_home_page_says_a_refusal_apart_from_no_answer: the
// computers whose lends are not listed are said in one line for them all,
// a sentence for each how, from /v1/lends' `unanswered`, `unheard`,
// `refused` and `older` (node.rs `Unlisted`), each naming whose, every
// one however many: the door marks only who did not answer, so a name left
// out of the others would be said nowhere. Planted, watched failing:
// question.js `computers` saying how many past four, "5 computers have …".
// Planted, watched failing: index.js `unsaid` joining nothing, a sentence a
// computer as before (its `whose` mapped one by one).
test('a computer that refused is said apart from one that did not answer', async () => {
  const named = (person, key) => ({ person, person_key: key, joined: { at: 1_791_000_000 } });
  const others = [
    ['bo', BO],
    ['cy', CY],
    ['dee', DEE],
    ['eve', EVE],
    ['fay', CY2],
  ];
  const people = [row('ada', ADA, 'started'), ...others.map(([p, k]) => row(p, k))];
  const answers = ada({ people });
  answers['GET /v1/lends?namespace=house'] = lends({
    unanswered: [named('bo', BO), named('dee', DEE)],
    unheard: [named('eve', EVE)],
    refused: [named('cy', CY)],
    older: [named('fay', CY2)],
  });
  const page = await home(answers);
  assert.deepEqual(page.text('#home .index > p'), [
    'bo’s and dee’s computers did not answer, so what they lend is not listed. eve’s computer has not heard of yours yet, so what they lend is not listed yet. cy’s computer answered without saying what they lend, so it is not listed. fay’s computer runs an older svrngs, so what they lend is not listed.',
  ]);
  answers['GET /v1/lends?namespace=house'] = lends({
    unheard: others.map(([p, k]) => named(p, k)),
  });
  const many = await home(answers);
  assert.deepEqual(many.text('#home .index > p'), [
    'bo’s, cy’s, dee’s, eve’s and fay’s computers have not heard of yours yet, so what they lend is not listed yet.',
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

// browser::the_home_page_names_what_the_terminal_does, now that nothing
// is: removing someone, the one thing the page sent a person to a terminal
// for, is a press in that person's popover, so the page as served has no
// section of lines to type.
test('nothing the page does is left to a terminal', () => {
  assert.ok(!INDEX.includes('id="terminal"'));
  assert.ok(!/svrngs (remove|leave|new|share)/.test(INDEX), 'no line to type');
});

test('home can stop a managed app without a group', async () => {
  const page = await home({
    ...ada({ groups: [] }),
    'GET /run': {
      rows: [
        { name: 'coder', port: 8101, text: 'Run by svrngs', action: 'stop', button: 'Stop coder' },
      ],
    },
  });
  assert.deepEqual(page.text('#runs p button'), ['Stop coder']);
  assert.doesNotMatch(page.text().join('\n'), /Look for programs running on this computer/);
});

// browser::the_home_page_s_links_say_what_they_open and
// ::the_home_page_shows_less_to_puzzle_over: the index's link names the
// lend and its kind, in a tab of its own, under what opening it does; its
// WiFi and address behind "Address", never in the cell's own words; the
// box's words beside it; no "What apps use" while nothing uses anything.
test('a lend’s link says what it opens and whose, in a tab of its own, and less to puzzle over', async () => {
  const shelf = lend('shelf');
  const page = await home(ada({ lent: [shelf] }));
  assert.deepEqual(page.text('#home .index > ul > li > a'), ['ada’s shelf (k)']);
  assert.deepEqual(page.text('#home .index > p'), [
    `${LENT}Opening it lets it write its own part of your group’s record, as you.`,
  ]);
  assert.deepEqual(cells(page), ['ada’s shelf (k) Address']);
  const [link] = page.all('#home .index > ul > li > a');
  assert.equal(link.attributes.target, '_blank');
  const labels = page.all('#lent label').map((label) => label.textContent);
  assert.ok(labels.includes('How many people at once: '), labels.join());
  assert.ok(!page.text('#home h2').includes('What apps use'), page.text('#home h2').join());
});

// The index is grids of cells, each grid under the one sentence true of
// everything in it, which names none of them; a cell is a lend's name and
// only what differs, so the cell that differs is the one with a sentence.
// Nothing a row said is lost: each sentence is a caption or in its cell.
// One person's things sit together, the group's own first, then what opens
// in a tab, the one that asks a yes last among them, as its cell takes a
// row of its own; then ports, this computer's own port last, with no
// caption: it has no Play. A lend that opens friends' lends as you keeps "Open" on its
// button, where the press is the yes. Another computer's lend at an address
// has Play after its Address (lend-name.js `played`); this computer's own,
// and an app the group's folder carries, have none.
test('each grid of the index is under the sentence its cells share, and a cell says only what differs', async () => {
  const bo = { key: BO, person: 'bo', raw: 'udp' };
  const lent = [
    lend('shelf'),
    lend('page', { reads: ['films'] }),
    lend('Money-house', { kind: 'Money', group: 'house' }),
    lend('game', bo),
    lend('echo', bo),
    lend('mine', { raw: 'tcp' }),
    lend('notes', { key: BO, person: 'bo' }),
  ];
  const people = [row('ada', ADA, 'started'), row('bo', BO)];
  const page = await home(ada({ lent, people }));
  assert.deepEqual(page.text('#home .index > p'), [
    `${CARRIED}Opening it lets it read and write only its own entries, as you in your group.`,
    `${LENT}Opening one of these lets it write its own part of your group’s record, as you.`,
    'Play opens one of these on this computer, for a program here to connect to.',
  ]);
  assert.equal(page.all('#home .index > ul').length, 4);
  assert.equal(page.all('#home .index table').length, 0);
  assert.deepEqual(cells(page), [
    'Money This app’s shared entries. Address',
    'ada’s shelf (k) Address',
    'bo’s notes (k) Address Play',
    'Open ada’s page (k) It also opens your friends’ films as you, and they can see what it asks. Address',
    'bo’s echo (k) Play',
    'bo’s game (k) Play',
    'ada’s mine (k) A port this computer lends.',
  ]);
  assert.deepEqual(
    [
      ...page.text('#home .index > ul > li > form button'),
      ...page.text('#home .index > ul > li > span button'),
    ],
    ['Open ada’s page (k)', 'Play', 'Play', 'Play'],
  );
  const full = await home(ada({ lent: [lend('shelf', { using: 3 })] }));
  assert.deepEqual(cells(full), ['ada’s shelf (k) Full now. Address']);
});

// The index for someone who cannot see where a thing sits. Each grid is a
// list named by its caption. Each way in, a link, a yes's button or a Play,
// is described by its grid's caption, and a way in that opens in a tab by
// what its own cell says too, so reached from a list of links it still
// says what opening it lets it do: the consent wording, said without
// sight. And each button says what it acts on, so no two in the index have
// one name. Planted, watched failing: lend-name.js's link with no
// `aria-describedby`.
test('each way in is described by its caption, and each button is named with what it acts on', async () => {
  const bo = { key: BO, person: 'bo', raw: 'udp' };
  const lent = [
    lend('shelf'),
    lend('notes', { using: 3 }),
    lend('page', { reads: ['films'] }),
    lend('Money-house', { kind: 'Money', group: 'house' }),
    lend('game', bo),
    lend('echo', bo),
  ];
  const page = await home(ada({ lent, people: [row('ada', ADA, 'started'), row('bo', BO)] }));
  const byId = (id) => page.all('#home .index *').find((at) => at.id === id);
  const grids = page.all('#home .index > ul');
  const captions = page.all('#home .index > p');
  assert.equal(grids.length, 3);
  for (const [at, grid] of grids.entries()) {
    assert.equal(grid.attributes.role, 'list');
    assert.equal(grid.attributes['aria-labelledby'], captions[at].id);
    assert.ok(captions[at].id, 'a caption has an id');
  }
  const described = (way) =>
    way.attributes['aria-describedby'].split(' ').map((id) => words(byId(id)));
  const ways = [
    ...page.all('#home .index > ul > li > a'),
    ...page.all('#home .index > ul > li > form button'),
    ...page.all('#home .index > ul > li > span button'),
  ];
  assert.deepEqual(
    ways.map((way) => [words(way), ...described(way)]),
    [
      ['Money', words(captions[0]), 'This app’s shared entries.'],
      ['ada’s notes (k)', words(captions[1]), 'Full now.'],
      ['ada’s shelf (k)', words(captions[1]), ''],
      [
        'Open ada’s page (k)',
        words(captions[1]),
        'It also opens your friends’ films as you, and they can see what it asks.',
      ],
      ['Play', words(captions[2])],
      ['Play', words(captions[2])],
    ],
  );
  const named = (button) => button.attributes['aria-label'] ?? words(button);
  const names = page.all('#home .index > ul > li > button').map(named);
  assert.deepEqual(names, [
    'Address of Money',
    'Address of ada’s notes (k)',
    'Address of ada’s shelf (k)',
    'Address of ada’s page (k)',
  ]);
  assert.deepEqual(
    [...ways.slice(-3), ...page.all('#home .index > ul > li > div > button')].map(named),
    [
      'Open ada’s page (k)',
      'Play bo’s echo',
      'Play bo’s game',
      ...names.map((name) => `Close: ${name}`),
    ],
  );
  const pops = page.all('#home .index > ul > li > div');
  assert.deepEqual(
    pops.map((pop) => [pop.attributes.role, pop.attributes['aria-label']]),
    names.filter((name) => name.startsWith('Address')).map((name) => ['dialog', name]),
  );
});

// What a group shares has no bound, so the words outside the index's cells
// do not grow with it: with 8 things and with 40 they are the same, word
// for word, and name no lend. Planted, watched failing: index.js's caption
// listing its cells' names (`${caption} ${cells.length}` stood in for it).
test('the words outside the index’s cells are the same for 8 things and for 40', async () => {
  const kinds = [{ reads: [] }, { key: BO, person: 'bo' }, { key: BO, person: 'bo', raw: 'udp' }];
  const people = [row('ada', ADA, 'started'), row('bo', BO)];
  const drawn = async (count) => {
    const lent = Array.from({ length: count - 2 }, (_, i) => lend(`thing${i}`, kinds[i % 3]));
    for (const kind of ['Money', 'Doc']) lent.push(lend(`${kind}-house`, { kind, group: 'house' }));
    const page = await home(ada({ lent, people }));
    const whole = page.text('#home .index > ul > li');
    assert.equal(whole.length, count);
    // The index's words, less each cell's, are its captions': whatever
    // element holds a word, it is in a cell or it is one of these.
    const said = page.text('#home .index > p');
    const outside = whole.reduce(
      (rest, cell) => rest.replace(cell, ''),
      page.text('#home .index')[0],
    );
    assert.equal(outside.replace(/\s/g, ''), said.join('').replace(/\s/g, ''));
    return said;
  };
  const [few, many] = [await drawn(8), await drawn(40)];
  assert.deepEqual(few, [
    `${CARRIED}Opening one of these lets it read and write only its own entries, as you in your group.`,
    `${LENT}Opening one of these lets it write its own part of your group’s record, as you.`,
    'Play opens one of these on this computer, for a program here to connect to.',
  ]);
  assert.deepEqual(many, few);
});

// A group's page is what is, then what is done: under the banner what
// needs the person, then the door, what the group shares and its files;
// after those who you have over, who else is visiting, and inviting; then
// the folds. Planted, watched failing: grouped.js `GroupPlaces` drawing
// who is visiting and the invitations under People, above Shared apps, as
// it did.
test('a group’s page is its banner, what needs you, its people, what they share, then inviting', async () => {
  const visiting = [
    { id: 'v1', guest: 'jo', host: 'ada', host_key: ADA, until: 1_893_000_000, yours: true },
    { id: 'v2', guest: 'sy', host: 'bo', host_key: BO, until: 1_893_000_000, yours: false },
  ];
  const people = [row('ada', ADA, 'started'), row('bo', BO)];
  const answers = ada({ lent: [lend('shelf')], people, groups: [group({ visiting })] });
  const changed = [{ group: 'house', app: 'Money', by: 'bo', by_key: BO, at: 1_791_000_900 }];
  answers['GET /v1/lends?namespace=house'] = lends({ lent: [lend('shelf')], changed });
  const page = await home(answers, '/?group=house');
  const [banner] = page.all('#group-house > h1');
  assert.equal(banner.nextSibling.attributes.class, 'grouped-notice');
  const heads = page
    .all('#group-house *')
    .filter((at) => at.parentNode.id === 'group-house')
    .filter(
      (at) => ['h2', 'h3', 'details'].includes(at.localName) || at.className === 'publication',
    )
    .map((at) => (at.localName === 'details' ? words(at.firstChild) : words(at)));
  assert.deepEqual(heads, [
    'People',
    'Shared apps',
    'Shared files',
    'Who you have over',
    'Visiting now',
    'Invite people',
    'Add an app',
    'Manage this group',
  ]);
});

// "Add an app" on a group's page opens on what an app here is, a folder
// anyone can put in the group's shared files, and where that folder is
// opened from; then what is in the group already, in one line; then
// svrngs's own apps, said to be examples, with how to bring one, as the
// legend of their boxes; then the button; and last the other way to add
// something, a link to it. With nothing left to tick it is the same fold
// less the boxes and the button, so never a dead end. Planted, each
// watched failing: grouped.js `GroupPlaces` drawing the shelf before what
// an app is; page/shelf.js linking to the overview where it says This
// computer.
test('Add an app leads with what an app is, then svrngs’s examples, then the other way', async () => {
  const carried = (kind) => lend(`${kind}-house`, { kind, group: 'house' });
  const shelf = [
    { name: 'Money', words: 'who owes whom.' },
    { name: 'Events', words: 'nights out.' },
  ];
  const opened = async (lent) => {
    const answers = ada({ groups: [group({ name: 'House' })] });
    answers['GET /v1/lends?namespace=house'] = lends({ lent, shelf });
    const page = await home(answers, '/?group=house');
    const fold = page
      .all('#group-house > details')
      .find((at) => words(at.firstChild) === 'Add an app');
    const within = (at) => at && (at === fold || within(at.parentNode));
    const parts = page
      .all('#group-house > details *')
      .filter((at) => within(at) && ['p', 'legend'].includes(at.localName));
    const links = page.all('#group-house > details section a');
    return [parts.map(words).filter(Boolean), links.map((link) => link.attributes.href)];
  };
  const IS =
    'An app here is a folder in this group’s shared files, under apps: a page, index.html, and whatever it needs. Put one there, and everyone in the group can open it under Shared apps. Shared files, on this page, opens the group’s folder.';
  const OTHER = 'A program already running on this computer is lent from This computer instead.';
  const [some, links] = await opened([carried('Money')]);
  assert.deepEqual([some[0], some[1], some.at(-1)], [IS, 'Money is already in House.', OTHER]);
  assert.ok(some[2].startsWith('These are svrngs’s examples, to use or to change. Tick '), some[2]);
  assert.deepEqual(some.slice(3, 6), [
    'Events: nights out.',
    'See how they’re made, and how to change one.',
    'Bring the ticked ones to House',
  ]);
  assert.deepEqual(links, [
    'https://github.com/alexsbryan/svrngs/blob/main/docs/guide/examples.md',
    '/?view=computer',
  ]);
  const [none] = await opened([carried('Money'), carried('Events')]);
  assert.deepEqual(none, [IS, 'Money and Events are already in House.', OTHER]);
});

// Under "What acts as you", each Stop of an app that opens friends' things
// as you is named with the app it stops: several read "Stop" alike in a
// screen reader's list of buttons. Planted, watched failing: acting.js's
// Stop given no name.
test('a Stop under What acts as you is named with the app it stops', async () => {
  const page = lend('page', { reads: ['films'] });
  const answers = ada({ lent: [page] });
  answers['GET /reaching'] = { [q.label(page)]: { said: 'yes', at: 1_791_000_000 } };
  const drawn = await home(answers, '/?view=computer');
  const stops = drawn.all('#acts form button');
  assert.deepEqual(
    stops.map((stop) => [words(stop), stop.attributes['aria-label']]),
    [['Stop', 'Stop ada’s page']],
  );
});

// "What you lend" is a line a lend, its name, who can open it and how many
// use it now, its form folded under the line; what is seldom changed is
// folded again, under More settings; and no button there is stretched:
// look.css's full-width button is the phone page's index's alone
// (`tbody#lent`, web/page/lent-index.js), where the home page's section,
// also `#lent`, once took it. Planted, watched failing: lent-row.js drawing
// the form with no <details>; Let in this many drawn outside More settings.
test('each lend is one line with its form folded under it, and the wide button is the phone’s', async () => {
  const page = await home(ada({ lent: [lend('shelf')] }));
  assert.deepEqual(page.text('#lent li > details > summary'), [
    'shelf: lent to a group whose name this computer cannot read, 0 of 3 using it now.',
  ]);
  const folded = page.text('#lent li details button');
  for (const words of ['Let in this many', 'Save who can open this', 'Stop lending'])
    assert.ok(folded.includes(words), folded.join());
  assert.equal(page.text('#lent button').length, folded.length, 'no button outside the fold');
  assert.deepEqual(page.text('#lent li details details > summary'), ['More settings']);
  const more = page.text('#lent li details details button');
  for (const words of ['Let in this many', 'Sign people in with svrngs'])
    assert.ok(more.includes(words), more.join());
  for (const words of ['Save who can open this', 'Stop lending'])
    assert.ok(!more.includes(words), `${words}: ${more.join()}`);
  const look = readFileSync(new URL('look.css', HOME), 'utf8');
  assert.deepEqual(look.match(/^.*#lent button.*$/gm), [
    'tbody#lent button { width: 100%; text-align: left; }',
  ]);
  const phone = readFileSync(new URL('../../page/lent-index.js', import.meta.url), 'utf8');
  assert.ok(phone.includes('<tbody id="lent">'));
});

// A lend held for what answers on its port says so first on its lender's
// line, Closed, then why and every program that answers now (door.rs
// `answering` and `offered`, lent-row.js `HELD`; SHARE_BINDING §5.5), and
// Keep sharing it approves what its line showed, by the token it was listed
// with, with the first process's pid and start, at
// /v1/lends/<name>/approve and nothing else
// (lend-change.js `approve`; SHARE_BINDING_REVIEWS §6 #6); one listed with
// no token, whatever holds it, offers no approval. Planted, each watched
// failing: lent-row.js not reading `answers` (no "Now answering"); the
// approval posted with no token (`{ pid, start }`).
test('a held lend says what answers now, and Keep sharing it approves that', async () => {
  const answers = [
    { pid: 812, start: 1_763_600_000, program: 'python3', folder: '~/dev/garden' },
    { account: '_resolve' },
  ];
  const approved = [];
  const page = async (/** @type {string} */ held, token = '00c0ffee00c0ffee') =>
    home(
      ada({
        lent: [lend('shelf', { answering: true, held, answers, token })],
        'POST /v1/lends/shelf/approve': (body) => (approved.push(body), { name: 'shelf' }),
      }),
    );
  const unapproved = await page('unapproved');
  assert.ok(
    unapproved
      .text('#lent li details summary')[0]
      .includes(
        'shelf: Closed. Nobody can open it until you approve what answers on its port now. Now answering: python3 in ~/dev/garden and a program of the account _resolve.',
      ),
  );
  await unapproved.press('#lent li button', 'Keep sharing it');
  assert.deepEqual(approved, [{ token: '00c0ffee00c0ffee', pid: 812, start: 1_763_600_000 }]);
  const another = (await page('another')).text('#lent li details summary')[0];
  assert.ok(
    another.includes(
      'Closed. Something you didn’t approve was on its port. Now answering: python3 in ~/dev/garden and a program of the account _resolve.',
    ),
    another,
  );
  const untokened = await page('another', '');
  assert.deepEqual(
    untokened.text('#lent li button').filter((b) => b === 'Keep sharing it'),
    [],
  );
  const unjudged = await page('unjudged', '');
  assert.ok(
    unjudged
      .text('#lent li details summary')[0]
      .includes('Closed. svrngs can’t tell what answers on its port. Start it again.'),
  );
  assert.deepEqual(
    unjudged.text('#lent li button').filter((b) => b === 'Keep sharing it'),
    [],
  );
});

// A held lend names the jobs without the manager's vocabulary, including
// a description from a unit written before the availability page.
test('a kept lend whose setup changed says so, and names the programs', async () => {
  const unit = 'svrngs-keeper-coder.service';
  const answers = [
    { unit, description: 'coder, kept by svrngs-keeper' },
    { unit: 'recipes.service' },
  ];
  const page = await home(
    ada({
      lent: [lend('coder', { answering: true, held: 'changed', answers, token: '00c0ffee' })],
    }),
  );
  const line = page.text('#lent li details summary')[0];
  assert.ok(
    line.includes(
      'coder: Closed. What runs it changed since you chose to keep sharing it when it starts again. Now answering: coder and recipes.',
    ),
    line,
  );
  assert.ok(page.text('#lent li button').includes('Keep sharing it'));
});

// A lend friends can't open leads its line with that, marked `.bad` and in
// words, so the color is not the only sign: Closed where it is held, before
// its program not running (door.rs `answering`), else Not running; then
// who it is lent to, with no count of who uses it. An open lend's line is
// as it was. Planted, watched failing: lent-row.js drawing a held lend's
// line without the `.bad` mark.
test('a lend friends can’t open says Closed or Not running first, marked', async () => {
  const drawn = async (/** @type {object} */ more) => {
    const page = await home(ada({ lent: [lend('shelf', more)] }));
    return [page.text('#lent li details summary')[0], page.text('#lent summary strong.bad')];
  };
  const group = 'a group whose name this computer cannot read';
  assert.deepEqual(await drawn({ answering: false, held: 'another', answers: [] }), [
    `shelf: Closed. Something you didn’t approve was on its port. Nothing answers on its port now. Lent to ${group}.`,
    ['Closed'],
  ]);
  assert.deepEqual(await drawn({ answering: false }), [
    `shelf: Not running. Start it, and the groups you chose can open it again. Lent to ${group}.`,
    ['Not running'],
  ]);
  assert.deepEqual(await drawn({ answering: true }), [
    `shelf: lent to ${group}, 0 of 3 using it now.`,
    [],
  ]);
});

// Keep sharing it is offered by the token alone, which svrngs lists only
// where approving it would be taken (share.rs `approve`): a lend held
// unjudged, listed with one, offers it; another account's program first,
// as Linux shows it, is approved by the token alone. Nothing answering and
// a read svrngs couldn't make (`unread`) are said apart, neither with a
// token. Planted, each watched failing: Keep judging the hold itself (no
// button on the unjudged lend); the line reading no `unread` (a read that
// failed said as nothing answering).
test('Keep sharing it follows the token, and a read that failed is said apart', async () => {
  const approved = [];
  const page = async (/** @type {object} */ more) =>
    home(
      ada({
        lent: [lend('shelf', { answering: true, held: 'unjudged', ...more })],
        'POST /v1/lends/shelf/approve': (body) => (approved.push(body), { name: 'shelf' }),
      }),
    );
  const keep = (/** @type {any} */ drawn) =>
    drawn.text('#lent li button').filter((/** @type {string} */ b) => b === 'Keep sharing it');
  const theirs = await page({ answers: [{ account: '_resolve' }], token: '00c0ffee' });
  await theirs.press('#lent li button', 'Keep sharing it');
  assert.deepEqual(approved, [{ token: '00c0ffee' }]);
  const nothing = await page({ answers: [] });
  assert.ok(
    nothing
      .text('#lent li details summary')[0]
      .includes('Start it again. Nothing answers on its port now.'),
  );
  assert.deepEqual(keep(nothing), []);
  const unread = await page({ unread: { why: 'failed', detail: 'the read stopped' } });
  const said = unread.text('#lent li details summary')[0];
  assert.ok(
    said.includes(
      'Start it again. svrngs couldn’t read what answers on its port. Reload to look again.',
    ),
    said,
  );
  assert.deepEqual(keep(unread), []);
});

// The overview's card says a line of what the group's hidden section read:
// who is on, how many apps, versions to review. A read that failed is said
// as that, never counted, and a group not yet told says nothing. Planted,
// watched failing: grouped.js `Live` not asking `told.fresh`, the lends'
// failed read then counted as "On now: 1 of 2."
test('a group’s card on the overview says who is on and what is shared, or that it cannot be read', async () => {
  const people = [row('ada', ADA, 'started'), row('bo', BO)];
  const answers = ada({ people, lent: [lend('shelf'), lend('films')] });
  const changed = [{ group: 'house', app: 'Money', by: 'bo', by_key: BO, at: 1_791_000_900 }];
  const read = lends({
    lent: [lend('shelf'), lend('films')],
    unanswered: [{ person: 'bo', person_key: BO }],
  });
  answers['GET /v1/lends?namespace=house'] = { ...read, changed };
  const page = await home(answers, '/');
  assert.deepEqual(page.text('.group-list small'), [
    'On now: 1 of 2. Shared apps: 2. New versions to review: 1.',
  ]);
  // The next round's read of the lends fails: GROUP keeps the answer before.
  answers['GET /v1/lends?namespace=house'] = { status: 503, body: { error: 'off' } };
  await page.shown();
  assert.deepEqual(page.text('.group-list small'), ['This computer cannot read who is on now.']);
});

// The overview says who the person is once, above its cards, when they are
// the same person in every group; a card says it only when it differs
// there, or cannot be read. Planted, watched failing: grouped.js
// `GroupLinks` saying it on every card whatever the others say (`same`
// never true).
test('who you are is said once above the cards, and on a card only where it differs', async () => {
  const flat = { id: 'flat', name: 'Flat' };
  const said = async (more) => {
    const page = await home(
      ada({ groups: [group({ name: 'House' }), group({ ...flat, ...more })] }),
      '/',
    );
    const once = page.text('.grouped-nav p').filter((words) => words.startsWith('You are'));
    return [once, page.text('.group-list small').filter((words) => !words.startsWith('On now'))];
  };
  assert.deepEqual(await said({}), [['You are ada here.'], []]);
  assert.deepEqual(await said({ person: 'addie' }), [
    [],
    ['You are ada here.', 'You are addie here.'],
  ]);
  assert.deepEqual(await said({ person: null }), [
    [],
    ['You are ada here.', 'This computer cannot say who you are here.'],
  ]);
});

// "What acts as you" on This computer says a thing once: groups with the
// same deciders share a line, each named, and a group nothing reads and
// writes for you in has no heading there. Planted, watched failing:
// acting.js `readsIn` answering its heading whatever is under it.
test('what acts as you names groups with the same deciders in one line, and heads no empty group', async () => {
  const groups = [group({ name: 'House' }), group({ id: 'flat', name: 'Flat' })];
  const answers = ada({ groups });
  const money = lend('Money-house', { kind: 'Money', group: 'house' });
  answers['GET /v1/lends?namespace=house'] = lends({ lent: [money] });
  const page = await home(answers, '/?view=computer');
  assert.deepEqual(page.text('#acts h4'), ['House']);
  const said = page.text('#acts p');
  assert.ok(said.includes('House and Flat: This computer.'), said.join('\n'));
  assert.ok(!said.includes('Flat: This computer.'), said.join('\n'));
});

// browser::the_home_page_copies_a_lend_s_address_for_a_program: a lend's
// address to keep, and its Copy, which writes that address and says where
// a program takes it, are in its own cell, behind "Address": a button that
// opens the browser's popover by its id (`popovertarget`), which holds what
// opening it to this WiFi does over the button it is about. A port has
// none; and a lend two groups share has a popover in each, each with an id
// of its own, or one group's button would open the other's. That the
// clipboard then holds what the page wrote is Chromium's
// (browser::the_page_offers_a_computer_one_line_and_sends_the_token_nowhere).
// Planted, watched failing: lend-name.js naming the popover by its lend
// alone, with no group, so both groups' popovers shared one id; and, for
// its Close, wifi.js `Address` drawing Close with no `popovertarget`.
test('a lend’s address for a program is kept behind Address in its cell, and its Copy writes it and says so', async () => {
  const shelf = lend('shelf');
  const groups = [group(), group({ id: 'flat', name: 'Flat' })];
  const page = await home(ada({ lent: [shelf, lend('port', { raw: 'tcp' })], groups }));
  const foot = page.text('#home section ul.wifi li');
  assert.ok(foot[0].includes(`${shelf.origin} Bookmark this.`), foot[0]);
  const pops = page.all('#home .index > ul > li > div');
  const opens = page.all('#home .index > ul > li > button');
  assert.deepEqual(opens.map(words), ['Address', 'Address'], 'one a lend with an address, a group');
  assert.deepEqual(
    opens.map((button) => button.attributes.popovertarget),
    pops.map((pop) => pop.id),
  );
  assert.equal(new Set(pops.map((pop) => pop.id)).size, 2, 'an id of its own in each group');
  for (const pop of pops) {
    assert.equal(pop.attributes.popover, 'auto');
    const said = pop.childNodes.filter((kid) => kid.nodeType === 1).map(words);
    assert.equal(
      said[0],
      'Open it to this WiFi, and anyone on it can open it as you while svrngs runs.',
    );
    assert.ok(said[1].includes('Open it to this WiFi'), said[1]);
    assert.ok(said[1].includes('Copy its address for a program'), said[1]);
    // Last, its Close, the browser's own: it names the popover it hides.
    const close = pop.childNodes.filter((kid) => kid.nodeType === 1).at(-1);
    assert.deepEqual(
      [words(close), close.attributes.popovertarget, close.attributes.popovertargetaction],
      ['Close', pop.id, 'hide'],
    );
  }
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
  assert.deepEqual(page.all('#home .door > li').map(shown), ['ada asking…', 'bo asking…']);
  assert.equal(page.all('#home .door > li.off').length, 0);
  answer(lends({ unanswered: [{ person: 'bo', person_key: BO, joined: { at: 1_791_000_000 } }] }));
  await page.settle();
  assert.deepEqual(page.all('#home .door > li.off').map(shown), ['bo did not answer']);
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
// loaded once (home.js@19b63b2b:364 opened the page again); its address is
// All your groups, `/`, so a reload lands where the page is, not the form.
test('a group made is said and drawn with its links, with no reload', async () => {
  const house = group({ name: 'house', made: mine });
  const [answers, add] = growing([house], [lab, trip]);
  const asked = { ...answers, ...making(add) };
  const page = await home(asked, '/?view=make');
  await make(page);
  assert.ok(page.text('#home > p').join().includes('You made Lab.'), page.text('#home > p').join());
  assert.ok(page.text('#home section button').includes('Invite people'));
  assert.deepEqual(
    page.asked.filter(([how]) => how === 'POST').map(([, , body]) => body),
    [['Lab', 'Ada']],
  );
  assert.ok(
    page.lines().some((line) => line.startsWith('MAKE: made')),
    page.lines().join('\n'),
  );
  await reloaded(page, '/?view=make', asked);
  assert.deepEqual([page.went, page.unanswered], [[['replace', '/']], []]);
});

// pages::join::a_group_joined_is_drawn_with_no_reload, on layout B's join
// page: Ada pastes Sam's link to Trip and joins as Dee; the page says so
// and draws Trip, its shelf folded as just joined, having loaded once
// (home.js@19b63b2b:409 opened the page again); its address is All your
// groups, `/`, so a reload lands where the page is, not the form.
test('a group joined is said and drawn, its shelf folded, with no reload', async () => {
  const house = group({ name: 'house', made: mine });
  const [answers, add] = growing([house], [lab, trip]);
  const asked = { ...answers, ...making(add) };
  const page = await home(asked, '/?view=join');
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
  await reloaded(page, '/?view=join', asked);
  assert.deepEqual([page.went, page.unanswered], [[['replace', '/']], []]);
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
  assert.ok(housed.includes('Invite people') && !housed.includes('Reading house…'), housed);
  assert.ok(section('Lab (yours').includes('Reading Lab…'), section('Lab'));
  await join(page);
  assert.ok(section('Trip (Sam’s').includes('Reading Trip…'), section('Trip'));
  for (const give of holds.splice(0)) give();
  await page.settle();
  assert.ok(!page.text('#home section[data-colour]').join().includes('Reading '));
  assert.ok(section('Lab (yours').includes('Lab’s folder is on this computer'), section('Lab'));
  assert.ok(section('Trip (Sam’s').includes('Trip’s folder is on this computer'), section('Trip'));
  assert.deepEqual([page.went, page.unanswered], [[KEPT_A, KEPT_A], []]);
});

// home_group::the_home_page_makes_a_group_another_computer_joins_with_no_restart,
// the page's half: Dee, in Lab, makes Trip; the page says so and where
// home.rs `found` placed its folder, and Invite people shows the https link
// /publication answers on the page svrngs came from (publication.rs `shown`,
// the link home.rs `link` spells). Then trip, a name
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
    'POST /publication': (body) => (
      links.push(body),
      { id: 'p1', state: 'pending', address: null, until: since, link: 'http://127.0.0.1:9/#T' }
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
  await page.press('#group-trip button', 'Invite people');
  assert.deepEqual(links, [{ group: 'trip' }]);
  const link = page.text('#group-trip .publication p').find(Boolean) ?? '';
  assert.ok(link.startsWith('Paste this link in a chat: http://127.0.0.1:9/#T '), link);
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
  assert.deepEqual([page.went, page.unanswered], [[KEPT_A, KEPT_A], []]);
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
  assert.deepEqual([page.went, page.unanswered], [[KEPT_A], []]);
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
  assert.deepEqual([anas.went, anas.unanswered], [[KEPT_A], []]);
});

// Leaving from the group's own page in layout B, the one people get
// (home-reading.js `first`): Leave is under Manage this group, so the
// page is at `?group=trip` when it is pressed. Left, the page is All your
// groups: the leave said once, the groups still yours, and the way to
// Groups you left, never "This group is not available here" over a page
// with every place hidden; and its address, so a reload lands there too.
// The plant is the fault this law was written for: watched failing with
// leave.js keeping `?group=` on a leave.
test('leaving a group from its page in layout B lands on All your groups', async () => {
  const trip = (left) => group({ id: 'trip', name: 'Trip', keeps: false, left });
  const state = { left: false };
  const answers = ada({
    groups: [group({ name: 'House' }), trip(false)],
    'POST /v1/leaves': () => ((state.left = true), { outcome: 'left' }),
  });
  answers['GET /v1/groups'] = () => ({
    groups: [group({ name: 'House' }), trip(state.left)],
    key: ADA,
  });
  const page = await home(answers, '/?group=trip');
  await page.press('#group-trip button', 'Leave Trip');
  const drawn = read(page);
  assert.ok(drawn.includes('You left Trip.'), drawn);
  assert.ok(!drawn.includes('not available'), drawn);
  assert.deepEqual(page.text('nav a[aria-current=page]'), ['All your groups']);
  assert.deepEqual(page.text('.group-list a'), ['House']);
  assert.ok(drawn.includes('Groups you left'), drawn);
  const again = await reloaded(page, '/?group=trip', answers);
  assert.deepEqual(again.text('nav a[aria-current=page]'), ['All your groups']);
  assert.deepEqual([page.went, page.unanswered], [[['replace', '/']], []]);
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
      at: '127.0.0.1:41000',
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
      .all('#home section .index li')
      .filter((td) => td.textContent.match(/(Play|Stop)$/))
      .map(words);
  assert.equal(cells().length, 2, cells().join('\n'));
  const [house, flat] = page.all('#home section .index');
  const id = (table) => ancestorId(table);
  await page.press(`#${id(house)} .index li button`, 'Play');
  assert.ok(
    cells().every((cell) => cell.includes('Connect to 127.0.0.1:') && cell.endsWith('Stop')),
    cells().join('\n'),
  );
  await page.press(`#${id(flat)} .index li button`, 'Stop');
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
  await page.type('#lent input[type=number]', '0');
  const many = 'Say how many: 1 or more.';
  for (let n = 0; n < 2; n++)
    assert.equal(await pressed(page, '#lent button', 'Let in this many', '#lent li', many), 1);
  assert.equal(page.lines().filter((line) => line.startsWith('CHANGE: many: ')).length, 2);
  // The clipboard refuses, as the pages step's browser, granted none, did.
  navigator.clipboard.writeText = async () => Promise.reject(new Error('not allowed'));
  await page.press('#home section button', 'Add your phone or another computer');
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
