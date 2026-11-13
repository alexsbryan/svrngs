// The home page's acts, drawn whole with no browser (docs/CHECKS.md
// harness 5): what a press sends and what the page draws from each answer,
// over twin.mjs, each route answered in the shape its writer gives it
// (twin.mjs names each writer). Each law holds the page's half of a
// browser step test (crates/svrngs-node/tests/browser/main.rs) named
// beside it; the node's half, what the route does on disk or on the wire,
// is held under crates/svrngs-node/tests, named beside it too. The
// recorded-answer builders are home.test.mjs's, copied, not imported:
// importing a test file runs its tests again here. Layout A (`/?ui=a`), as
// those tests' driver opened it (web/tests/page.mjs `longPage`).
//
// Plants, each watched failing in the page's own module, then put back:
// named in the commit that brought these laws.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { register } from 'node:module';
import { test } from 'node:test';
import { open, words } from './twin.mjs';

const HOME = new URL('../../../crates/svrngs-node/src/home/', import.meta.url);
const q = await import(new URL('question.js?v=__VERSION__', HOME).href);

const AT = 'http://svrngs.localhost:4000';
const [ADA, BO, CY, CY2, DEE, PIA] = ['a1', 'b2', 'c3', 'c4', 'd5', 'f7'].map((k) => k.repeat(32));
const row = (person, key, joined = { at: 1_791_000_000 }, device = 'computer') => ({
  person,
  person_key: key,
  key,
  device,
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

// browser::the_home_page_changes_a_lend_s_limit_in_place, the page's half:
// ada's shelf lets one in at once; her row says so, and four, then
// everyone, then Stop lending, each posted to /lent (home.rs `change`,
// 204) and read back from /lent as the next read answers it. That the
// declaration takes each and is gone once stopped is home.rs's:
// tests/home.rs a_lend_s_limit_is_changed_on_the_home_page_and_the_door_takes_it.
test('a lend’s limit is changed in place: four, everyone, then stopped, each read back', async () => {
  let shelf = lend('shelf', { limit: 1 });
  const posted = [];
  const page = await home(
    ada({
      lent: [shelf],
      'GET /lent': () => ({ lent: shelf ? [shelf] : [], signing: [], readers: {} }),
      'POST /lent': (body) => {
        posted.push(body);
        shelf = body.to === 'stop' ? null : { ...shelf, ...body.to };
        return { status: 204 };
      },
    }),
  );
  const first = () => page.text('#lent li')[0] ?? '';
  assert.ok(first().startsWith('shelf: up to 1 person at once, 0 using it now.'), first());
  await page.type('#lent li input', '4');
  await page.press('#lent li button', 'Let in this many');
  assert.ok(first().startsWith('shelf: up to 4 people at once'), first());
  await page.press('#lent li button', 'Let everyone in');
  assert.ok(first().startsWith('shelf: everyone in the groups you chose at once'), first());
  await page.press('#lent li button', 'Stop lending');
  assert.deepEqual(page.text('#lent li'), []);
  assert.deepEqual(page.text('#lent p'), ['You lend nothing yet.']);
  assert.deepEqual(posted, [
    { lend: 'shelf', to: { limit: 4 } },
    { lend: 'shelf', to: { limit: null } },
    { lend: 'shelf', to: 'stop' },
  ]);
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// browser::the_home_page_turns_on_signing_people_in_with_svrngs, the
// page's half: the row's button says what it does; pressed, it posts
// `sign-in` to /lent and shows the issuer, the client and its secret and
// the deviations as home.rs `sign_in` answers them (oidc.rs `DEVIATIONS`),
// with what is left to set up; the secret leaves the page in no request,
// link, copy or tab; opened again, /lent says only that it signs people
// in (`signing`), so neither the secret nor the button is drawn, and the
// setup reminder stays. That the secret answered is the one the
// declaration keeps, owner-only, and /lent never says it, is home.rs's:
// tests/home.rs a_lend_is_signed_in_with_svrngs_from_the_home_page.
test('signing people in is turned on once: the secret shown on the page alone, then never', async () => {
  const secret = 'Sx9-secret.of_the~client';
  const shelf = lend('shelf', { kind: 'books' });
  const answered = {
    issuer: 'http://127.0.0.1:4000/oidc/shelf',
    client_id: 'shelf-client',
    client_secret: secret,
    deviations:
      'It is plain http on this computer, not https; it signs tokens with ES256; and it tells the app no email address, because svrngs has none.',
  };
  const answers = ada({
    lent: [shelf],
    'POST /lent': (body) => (body.to === 'sign-in' ? answered : { status: 400, body: {} }),
  });
  const page = await home(answers);
  const row = () => page.text('#lent li')[0] ?? '';
  assert.ok(row().includes('Sign people in with svrngs'), row());
  await page.press('#lent li button', 'Sign people in with svrngs');
  const said = row();
  for (const shown of [
    `issuer ${answered.issuer}`,
    'Sign-in credentials created; app setup still required.',
    'install any required OpenID Connect plugin',
    'from the app server on this computer',
    `client id ${answered.client_id}`,
    `client secret ${secret}`,
    'plain http',
    'ES256',
    'no email',
  ])
    assert.ok(said.includes(shown), `${shown}: ${said}`);
  assert.deepEqual(
    page.asked.filter(([how]) => how === 'POST'),
    [['POST', '/lent', { lend: 'shelf', to: 'sign-in' }]],
  );
  const left = JSON.stringify([
    page.asked,
    page.all('a').map((a) => a.attributes.href),
    page.copied,
    page.opened,
    page.went,
    page.lines(),
  ]);
  assert.ok(!left.includes(secret), left);

  const again = await home({
    ...answers,
    'GET /lent': { lent: [shelf], signing: ['shelf'], readers: {} },
  });
  const reread = again.text('#lent li')[0] ?? '';
  assert.ok(!reread.includes(secret), reread);
  assert.ok(!reread.includes('Sign people in with svrngs'), reread);
  assert.ok(reread.includes('Sign-in credentials created; app setup still required.'), reread);
  assert.ok(reread.includes('app’s OpenID Connect setup guide'), reread);
});

// browser::two_people_of_one_name_are_told_apart_on_the_home_page, the
// page's half: a second ada, let in by hand, changed Money's files, as
// carried.rs `changed` names her, by name and first key. Ada's page says
// that ada changed them, told apart by when she joined, never "you"; and
// its hand-on of the order of removals offers her once, the same way, as
// someone else (leave-group.js `Leave`).
test('two people of one name: the other changed an app, and is offered the hand-on, told apart', async () => {
  const ADA2 = 'a2'.repeat(32);
  const at = 1_792_000_000;
  const people = [row('ada', ADA, 'started'), row('ada', ADA2, { at: 1_791_000_000 })];
  const answers = ada({ people });
  answers['GET /v1/lends?namespace=house'] = lends({
    changed: [{ group: 'house', app: 'Money', by: 'ada', by_key: ADA2, at }],
  });
  const page = await home(answers);
  const changed = page.text('#acts p').find((line) => line.startsWith('Money: '));
  const named = changed?.match(/^Money: ada \(joined ([^)]+)\) changed its files on (.+)$/);
  assert.ok(named, `the other ada, told apart: ${page.text('#acts p').join('\n')}`);
  const options = page.text('#home section select option');
  assert.equal(options.length, 1, options.join());
  assert.ok(options[0].startsWith('ada (joined '), options.join());
  assert.equal(options[0], `ada (joined ${named[1]})`);
});

// The door's cells, each its class and its words, sorted by its words, as
// main.rs `drew_the_door` read them (the browser's `className`).
const door = (page) =>
  page
    .all('#home .door li')
    .map((li) => [li.attributes.class ?? '', words(li)])
    .sort(([, a], [, b]) => a.localeCompare(b));
// One not answering, by name and first key, as node.rs `Unlisted` names
// them in /v1/lends' `unanswered`.
const silent = (person, key) => ({ person, person_key: key });

// browser::whose_computer_did_not_answer_and_who_asked_are_told_by_key,
// the page's half: two people are in house as cy, the first's computer
// answering, the second's not, and the second asks to remove dee. The
// door draws the first cy on and the second off; the row saying whose
// computer did not answer, and the ask, name that cy, told apart by when
// they joined; bo's and dee's computers do not answer either. That a
// computer that does not answer is named so in /v1/lends, by name and
// first key, is surface.rs's: tests/surface.rs
// an_app_asks_what_the_group_s_people_lend_its_own_included_and_none_stale.
test('whose computer did not answer, and who asked, are told by key', async () => {
  const people = [
    row('ada', ADA, 'started'),
    row('bo', BO),
    row('cy', CY, { at: 1_791_000_000 }),
    row('cy', CY2, { at: 1_791_000_002 }),
    row('dee', DEE, { at: 1_791_000_003 }),
  ];
  const ask = {
    said: 'asked',
    by: 'cy',
    by_key: CY2,
    by_joined: { at: 1_791_000_002 },
    of: 'dee',
    of_key: DEE,
    of_joined: { at: 1_791_000_003 },
    at: 1_791_000_100,
  };
  const answers = ada({ people, groups: [group({ removals: [ask] })] });
  answers['GET /v1/lends?namespace=house'] = lends({
    unanswered: [silent('bo', BO), silent('cy', CY2), silent('dee', DEE)],
  });
  const page = await home(answers);
  const cys = door(page)
    .filter(([, said]) => said.startsWith('cy (joined '))
    .map(([drawn, said]) => [drawn, said.split(') ').at(-1)])
    .sort();
  assert.deepEqual(cys, [
    ['off', 'did not answer'],
    ['on', 'on now'],
  ]);
  const second = q.called(people, CY2);
  const rows = page.text('#home table.index td[colspan]');
  const unheard = rows.filter(
    (line) =>
      line.startsWith('cy') &&
      line.endsWith("'s computer did not answer, so what they lend is not listed."),
  );
  assert.equal(unheard.length, 1, rows.join('\n'));
  assert.ok(unheard[0].startsWith(`${second}'s computer`), unheard[0]);
  assert.ok(second.startsWith('cy (joined '), second);
  const asked = page
    .text('#home .removals li')
    .find((line) => line.includes('asked to remove dee'));
  assert.ok(asked?.startsWith(`${second} asked`), page.text('#home .removals li').join('\n'));
});

// browser::the_door_draws_who_is_on_now, the page's half: cy's computer
// answers, bo's does not, and pia is in from a phone. The door lights ada
// and cy "on now", draws bo off, "did not answer", once the lends are
// answered and never lit before, and marks pia in from a phone, claiming
// nothing of when; nobody is left out, and no word on the page is one a
// reader must look up. The browser's bar, bo off within 10 s, is here an
// order (CHECKS.md seam C): unjudged while /v1/lends is asked, judged once
// it answers. That bo's computer is named unanswered within the round is
// the node's: tests/surface.rs
// an_app_asks_what_the_group_s_people_lend_its_own_included_and_none_stale
// and a_lent_page_is_not_held_by_a_computer_that_never_answers; pia's
// `device` is rail.rs `standing_json`'s, src/rail/tests.rs
// the_people_door_answers_the_register_with_keys.
test('the door draws who is on now: lit, off once answered, and in from a phone', async () => {
  let answer;
  const held = new Promise((resolve) => (answer = resolve));
  const people = [
    row('ada', ADA, 'started'),
    row('bo', BO),
    row('cy', CY, { at: 1_791_000_001 }),
    row('pia', PIA, { at: 1_791_000_002 }, 'browser'),
  ];
  const answers = ada({ people });
  answers['GET /v1/lends?namespace=house'] = () => held;
  const page = await home(answers);
  assert.equal(page.all('#home .door li.off').length, 0, door(page).join('\n'));
  assert.equal(page.all('#home .door li.on').length, 0, door(page).join('\n'));
  answer(lends({ unanswered: [silent('bo', BO)] }));
  await page.settle();
  assert.deepEqual(door(page), [
    ['on', 'ada on now'],
    ['off', 'bo did not answer'],
    ['on', 'cy on now'],
    ['phone', 'pia in from a phone'],
  ]);
  // The page as the browser's `innerText` read it: home/index.html's body
  // as served around #home, and what the page drew into it. "What acts as
  // you" is the verb, the heading capability-what-acts names; "act"
  // guarded below is the record's noun, said nowhere else.
  const served = readFileSync(new URL('index.html', HOME), 'utf8')
    .split(/<\/?body[^>]*>/)[1]
    .replace(/<main id="home">.*?<\/main>/s, '')
    .replace(/<[^>]+>/g, ' ');
  const said = new Set(
    [served, ...page.text()]
      .join(' ')
      .toLowerCase()
      .replace('what acts as you', '')
      .split(/[^\p{L}\p{N}]+/u),
  );
  const looked = ['key', 'member', 'record', 'act', 'hash', 'origin', 'port', 'relay'].filter(
    (word) => said.has(word) || said.has(`${word}s`),
  );
  assert.deepEqual(looked, [], page.text().join('\n'));
});

// browser::the_home_page_makes_a_link_to_invite_someone, the page's half:
// each button says what it makes; pressed, it posts /link for the group,
// a device's or not, and shows the link home.rs `link` answers, saying who
// it is for and how long it works; Copy the link writes the link shown and
// says so beside it. That the link names this computer as its issuer, and
// lets someone in, is home.rs's: tests/home.rs
// a_link_the_page_makes_before_a_home_names_the_home_it_comes_to (Bo
// joins by it); its token parses as svrngs-core link_tests.rs holds.
test('a link to invite someone, and one for your own device, made, shown and copied', async () => {
  const made = [];
  const page = await home(
    ada({
      'POST /link': (body) => {
        made.push(body);
        const client = `svrngs://join/TOKEN${made.length}`;
        return { client, https: null, device: body.device };
      },
    }),
  );
  const last =
    'It works for a day, while this computer is on. A new link stops this one letting anyone new in; everyone it let in stays. Copy the link';
  const said = () => page.text('#home section p.link')[0] ?? '';
  for (const [button, starts, n] of [
    [
      'Invite someone to ',
      'Send this link to the person you are inviting: svrngs://join/TOKEN1 ',
      1,
    ],
    [
      'Add your phone or another computer',
      'Open this link on your other computer or phone; it joins as you, so send it to nobody else: svrngs://join/TOKEN2 ',
      2,
    ],
  ]) {
    await page.press('#home section button', button);
    assert.equal(made.length, n);
    assert.ok(said().startsWith(starts), said());
    assert.ok(said().endsWith(last), said());
  }
  assert.deepEqual(made, [
    { group: 'house', device: false },
    { group: 'house', device: true },
  ]);
  await page.press('#home section p.link button', 'Copy the link');
  assert.ok(said().endsWith('Copy the link Copied.'), said());
  assert.deepEqual(page.copied, ['svrngs://join/TOKEN2']);
  assert.deepEqual([page.went, page.unanswered], [[], []]);
});

// browser::the_home_page_opens_a_lend_to_this_wifi_and_closes_it, the
// page's half: ada's shelf's line, below her group's index, says what its
// button does and whom it lets in before it is pressed; pressed, it turns
// the room on (POST /v1/room) and says the address to type, in the node's
// words (room.rs `Served::said`); opened again, the line says it is open,
// and "Open to this WiFi" says who can open it; pressed again, it says it
// is closed. That the room serves the shelf at that address on this
// host's network, and refuses once off, is room.rs's: tests/room.rs
// a_lend_turned_on_is_served_to_this_network_until_it_is_turned_off.
test('a lend opened to this WiFi says the address, and closed says so', async () => {
  const shelf = lend('shelf');
  const at = '192.168.1.20:41234';
  const open = [
    {
      person: 'ada',
      key: ADA,
      name: 'shelf',
      served: { at },
      said: `Open to this WiFi: type http://${at}/ into an app or a browser on a device here, such as your TV, a phone or another computer. Anyone on this WiFi can open it there as you.`,
    },
  ];
  const turns = [];
  const turn = (body) => (turns.push(body), body.on ? open : []);
  const line = '#home section ul.wifi li';
  const page = await home(ada({ lent: [shelf], 'POST /v1/room': turn }));
  const [wifi] = page.all('#home section ul.wifi');
  const kin = wifi.parentNode.childNodes;
  const [index] = page.all('#home section table.index');
  assert.ok(kin.indexOf(index) >= 0 && kin.indexOf(index) < kin.indexOf(wifi), 'below the index');
  const [offered] = page.text(line);
  assert.ok(offered.startsWith('ada’s shelf (k):'), offered);
  assert.ok(
    offered.includes(
      'Open it to this WiFi, and anyone on it can open it as you while svrngs runs.',
    ),
    offered,
  );
  await page.press(`${line} button`, 'Open it to this WiFi');
  assert.ok(page.text(line)[0].includes(open[0].said), page.text(line)[0]);

  const again = await home(ada({ lent: [shelf], 'GET /v1/room': open, 'POST /v1/room': turn }));
  assert.ok(again.text(line)[0].includes(open[0].said), again.text(line)[0]);
  await again.press(`${line} button`, 'Close it to this WiFi');
  const shut = 'Closed to this WiFi: nobody on it can open it through this computer now.';
  assert.ok(again.text(line)[0].includes(shut), again.text(line)[0]);
  assert.ok(again.text('#home h3').includes('Open to this WiFi'), again.text('#home h3').join());
  const who =
    'Anyone on this WiFi can open these as you while svrngs runs: your TV, a phone, another computer. Each stays on the WiFi where you turned it on.';
  assert.ok(again.text('#home p').includes(who), again.text('#home p').join('\n'));
  assert.deepEqual(turns, [
    { on: { key: ADA, name: 'shelf' } },
    { off: { key: ADA, name: 'shelf' } },
  ]);
  assert.deepEqual([page.went, again.went], [[], []]);
});

// browser::an_older_computer_is_said_as_that_on_both_pages, the home
// page's half: ada's computer names cy's as one running an older svrngs,
// as node.rs `answered` names a computer whose answer carries an older
// version (/v1/lends' `older`, by name and first key, its lends not
// listed): one row says so, once. That an older version is named so is
// the node's: src/node/tests.rs a_computer_answering_an_older_version_is_named_older.
test('an older computer is said once on the home page, and nothing it lends listed', async () => {
  const people = [row('ada', ADA, 'started'), row('cy', CY)];
  const answers = ada({ people });
  answers['GET /v1/lends?namespace=house'] = lends({ older: [silent('cy', CY)] });
  const page = await home(answers);
  const cells = page.text('#home table.index td');
  const said = "cy's computer runs an older svrngs, so what they lend is not listed.";
  assert.equal(cells.filter((cell) => cell === said).length, 1, cells.join('\n'));
  assert.equal(cells.join('').split('runs an older svrngs').length - 1, 1, cells.join('\n'));
});

// The same claim's group's page half, both engines' in the browser: the
// phone's PAGE composed from web/page/*'s flows, as pages/enter.test.mjs
// composes it, its read stepped with what web/page/enter.js `read`
// answers. The computer that sent the link answers with no version
// (svrngs-core `older`, version.rs): the page says once that it runs an
// older svrngs, and reads nothing of its answer, keeping nothing. Through
// a computer of this version that names cy's older, the page says so of
// cy's in one line, and lists nothing of cy's.
register('./layout.mjs', import.meta.url);
const { compose } = await import('../../page/page.js?v=__VERSION__');
const FLOWS = [
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
const phone = compose(
  Object.fromEntries(
    await Promise.all(
      FLOWS.map(async (f) => [f, await import(`../../page/${f}.js?v=__VERSION__`)]),
    ),
  ),
);
test('an older computer is said once on the group’s page, by whose it is, and nothing of it read', () => {
  const house = { id: 'g1', name: 'house', person: 'pia', token: 'TOKEN', addresses: [] };
  const through = (state, ...messages) =>
    messages.reduce((at, message) => {
      const stepped = phone.step(at, message);
      assert.equal(stepped.refused, undefined, `${message.type}: ${stepped.refused}`);
      return stepped.state;
    }, state);
  const reading = through(
    phone.first({ fine: false }),
    { type: 'hold', group: house },
    { type: 'held' },
    { type: 'opened', group: house },
  );
  const people = [row('ada', ADA, 'started'), row('cy', CY), row('pia', PIA, undefined, 'browser')];
  const body = { version: 9, ...lends(), removals: [], visiting: [], people };
  const older = 'runs an older svrngs';
  const theirs = lend('shelf', { key: CY, person: 'cy', label: 'l' });

  const behind = phone.step(reading, {
    type: 'read',
    n: 1,
    ok: true,
    older: true,
    body,
    lent: [theirs],
  });
  assert.deepEqual(
    behind.effects.map((effect) => effect.m?.said ?? effect.run),
    [
      'release',
      'The computer that sent your link runs an older svrngs, so this page cannot show what house lends. Installing svrngs again there updates it.',
    ],
  );
  assert.equal(behind.state.lent, undefined, 'nothing of its answer read');
  assert.equal(phone.response(behind.state), '');

  const cys = phone.step(reading, {
    type: 'read',
    n: 1,
    ok: true,
    body: { ...body, older: [silent('cy', CY)] },
    lent: [],
  });
  const said = phone.response(cys.state);
  assert.equal(said.split('\n').filter((line) => line.includes(older)).length, 1, said);
  assert.ok(
    said.includes('cy’s computer runs an older svrngs, so what they lend is not listed.'),
    said,
  );
  assert.equal([...phone.heading(cys.state), said].join(' ').split(older).length - 1, 1);
  assert.deepEqual(cys.state.lent, []);
});

// browser::the_screen_shows_what_is_lent_after_it_opened, the page's half:
// ada's page opened as house's screen, `/?screen=house`. Cy lent radio
// before it opened, so it is in cy's window and no line says it arrived;
// bo's computer does not answer. Once drawn, ada lends shelf, and the
// screen, reading again by itself on its next tick (on-screen.js SCREEN,
// a second after its last read began), says "ada lent shelf" under "Since
// this screen opened", marked new there and in ada's window, the only
// marks; each lend opens in a tab; and its count says three people, two on
// now, two things lent. Ada's window comes first, as she started the
// group. That a lend posted to /running is declared and listed in
// /v1/lends is the node's: tests/folders.rs
// the_home_page_s_lend_keeps_its_group_s_terms_as_drawn and tests/surface.rs
// an_app_asks_what_the_group_s_people_lend_its_own_included_and_none_stale.
// Kept last in this file: the screen's timer reads through the global
// `fetch` the twin set, which the next page opened would set anew.
test('the screen says what was lent after it opened, marked new, and counts it', async () => {
  const people = [row('ada', ADA, 'started'), row('bo', BO), row('cy', CY, { at: 1_791_000_001 })];
  const radio = lend('radio', { key: CY, person: 'cy' });
  const shelf = lend('shelf');
  const listed = { lent: [radio] };
  const answers = ada({ people });
  answers['GET /v1/lends?namespace=house'] = () =>
    lends({ lent: listed.lent, unanswered: [silent('bo', BO)] });
  const page = await home(answers, '/?screen=house');
  try {
    const windows = () =>
      page.all('#home .door li').map((li) => ({
        drawn: [li.attributes.class ?? '', words(li)],
        links: page.all('#home .door li a').filter((a) => li.childNodes.includes(a)),
      }));
    const since = () => page.text('#home aside ol li');
    assert.deepEqual(since(), [], 'nothing said of what was there before it opened');
    assert.ok(
      windows().some(({ drawn }) => drawn[1] === 'cy radio on now'),
      page.text().join('\n'),
    );
    listed.lent = [radio, shelf];
    // Its next read is its own, on a timer of the page's: waited for here,
    // never pressed.
    for (let waited = 0; !since().length && waited < 10_000; waited += 100) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      await page.settle();
    }
    const drawn = windows().map(({ drawn }) => drawn);
    assert.deepEqual(drawn[0], ['on', 'ada shelf new on now']);
    assert.deepEqual(
      drawn.sort(([, a], [, b]) => a.localeCompare(b)),
      [
        ['on', 'ada shelf new on now'],
        ['off', 'bo did not answer'],
        ['on', 'cy radio on now'],
      ],
    );
    assert.equal(since().length, 1, since().join('\n'));
    assert.ok(since()[0].endsWith(' ada lent shelf new'), since()[0]);
    assert.equal(page.all('#home mark').length, 2);
    const links = windows().flatMap(({ links }) => links);
    assert.deepEqual(
      links.map((a) => [a.attributes.href, a.attributes.target]),
      [
        [shelf.origin, '_blank'],
        [radio.origin, '_blank'],
      ],
    );
    assert.equal(page.text('#home section > p')[0], '3 people · 2 on now · 2 things lent');
    assert.deepEqual([page.went, page.unanswered], [[], []]);
  } finally {
    // Its next read never answered, so it ticks no more and the run ends.
    answers['GET /v1/lends?namespace=house'] = () => new Promise(() => {});
  }
});
