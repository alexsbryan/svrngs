// The group's page on a phone opening its group, with no browser: PAGE
// composed from its flows (web/page/page.js `compose`, as web/app.js
// composes it), each effect run in turn by the flows' own io, but for what
// touches the browser's storage or leaves the page, answered here: `look`
// as enter.js's reads the address, `lock` held, `open` unsealing the key
// (dial.js `hold`), `keep` kept, among.js's `others` from the groups kept;
// a `hand` (hand.js, a navigation), `again` (five seconds on) and `cold` (a
// new address) are kept as what the page did, never run. The read is the
// page's own (enter.js `readLends`, dial.js `lends`), through the runtime
// stood in for by phone-core.mjs: each computer, by key, answers
// `/people/lends` with a body recorded in the shape node.rs `people_lends`
// writes (:1025-1034), refuses as the door does (door.rs `LendRefusal::say`),
// fails as web/src/lib.rs `reach` refuses, or never answers. What the page
// says is read as a person reads it: the heading (#status), the response
// (#response), and the places drawn by their own views, function by
// function.
//
// Each law holds a claim a test in crates/svrngs-node/tests/browser/main.rs
// held in WebKit or Chromium, named beside it; what is the browser's own,
// a dial over a real relay, a navigation, the clipboard, stays there.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { register } from 'node:module';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';

register('./layout.mjs', import.meta.url);
register('./phone-core.mjs', import.meta.url);
const { compose } = await import('../../page/page.js?v=__VERSION__');
const { hold } = await import('../../page/dial.js?v=__VERSION__');
const { GroupDoor } = await import('../../page/group-door.js?v=__VERSION__');
const { LentIndex } = await import('../../page/lent-index.js?v=__VERSION__');
const { Removals } = await import('../../page/removals.js?v=__VERSION__');
const { ComputerLine } = await import('../../page/computer-line.js?v=__VERSION__');
const { machine } =
  await import('../../../crates/svrngs-node/src/home/ui/machine.js?v=__VERSION__');
const { COPY, Copy, drawn } =
  await import('../../../crates/svrngs-node/src/home/ui/copy.js?v=__VERSION__');
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
const parts = compose(
  Object.fromEntries(
    await Promise.all(
      FLOWS.map(async (f) => [f, await import(`../../page/${f}.js?v=__VERSION__`)]),
    ),
  ),
);

// The edge's `issued` (enter.js), cut from its text, as io.look gives
// `looked` its `linked`.
const ENTER = readFileSync(new URL('../../page/enter.js', import.meta.url), 'utf8');
const issued = runInNewContext(`${/^const issued = [^]*?\n};\n/m.exec(ENTER)?.[0]}\nissued`, {
  Boolean,
});

// Each computer's key, 64 hex, as svrngs-core spells one.
const [ADA, ADA2, BO, CY, DEE, EVE] = ['a1', 'a2', 'b2', 'c3', 'd4', 'e5'].map((k) => k.repeat(32));
const VERSION = 5;
// A lend as /people/lends lists it: node.rs `Lending` (:923) and lend.rs
// `Offered` (:512) flattened into it; kind `k`, as main.rs `lend` declares.
const lend = (person, key, name, more = {}) => ({
  person,
  key,
  relay: null,
  version: VERSION,
  name,
  kind: 'k',
  limit: 3,
  using: 0,
  upload_bps: null,
  reads: [],
  groups: ['house'],
  ...more,
});
// A person standing, as rail.rs `people_json` (:166-171) writes one.
const row = (person, key, joined = { at: 1_791_000_000 }, device = 'computer') => ({
  person,
  key,
  person_key: key,
  joined,
  device,
});
// Whose computer did not say, as node.rs `Named` (:163) writes one.
const whom = (person, key) => ({ person, person_key: key });
// The answer to `/people/lends`, node.rs `people_lends` (:1025-1034): the
// four lists of node.rs `Unlisted` (:173), then what it adds.
const answer = (more = {}) => ({
  unanswered: [],
  unheard: [],
  refused: [],
  older: [],
  lent: [],
  removals: [],
  meets: null,
  visiting: [],
  people: [row('ada', ADA, 'started'), row('bo', BO)],
  shelf: [],
  changed: [],
  version: VERSION,
  ...more,
});
// A refusal as the door says it (door.rs `LendRefusal::say`, `said`): its
// status, its kind in the header and the body, and its words.
const refusal = (status, kind, error) => ({
  status,
  refused: kind,
  body: { error, kind },
});
// What web/src/lib.rs `reach` refuses with (:353, :351): a computer not
// answered in its time, and one no route reached, iroh's error after it
// naming the relay tried, as the browser test's was 127.0.0.1's.
const RELAY = 'http://127.0.0.1:4433/';
const unanswered = new Error('LAYER tls/alpn+relay: REFUSED — not answered in Some(4s): asleep');
const closed = new Error(
  `LAYER tls/alpn+relay: REFUSED — connect() failed on every route: no path to the computer through ${RELAY}`,
);
const SILENT = 'silent';

/**
 * The runtime as web/src/lib.rs answers, set for the page's modules
 * (phone-core.mjs). `tokens` each name the computer that made the link, by
 * key (`issuer_of`); `computers` each answer a dial, by key: a body, or
 * `{ status, refused, body }`, an Error the dial fails with, or SILENT, a
 * dial never answered. `relay_of` answers the first relay a link names,
 * none when `relayless`. What the page asked is kept:
 * each endpoint bound, each dial, each request.
 */
function runtime({ tokens = { TOKEN: ADA }, computers, relayless = false }) {
  const asked = { endpoints: [], dials: [], fetched: [], joins: [] };
  const reached = (key) => {
    asked.dials.push(key);
    const given = computers[key];
    if (given === SILENT || given === undefined) return new Promise(() => {});
    if (given instanceof Error) return Promise.reject(given);
    const { status = 200, refused, body } = 'body' in given ? given : { body: given };
    return Promise.resolve({
      fetch: async (method, path, headers) => {
        asked.fetched.push([key, method, path, headers]);
        return new Response(JSON.stringify(body), {
          status,
          headers: refused ? { 'x-svrngs-refused': refused } : {},
        });
      },
      close: async () => {},
    });
  };
  globalThis.__core = {
    relay_of: () => (relayless ? undefined : RELAY),
    relay_taken: () => true,
    issuer_of: (token) => {
      if (!(token in tokens)) throw new Error('not a link’s token');
      return tokens[token];
    },
    endpoint: async (key, token, meets) => {
      asked.endpoints.push([token, meets]);
      return {
        dial_link: () => reached(tokens[token]),
        dial: (key) => reached(key),
        close: async () => {},
      };
    },
    connect_to_join: (key, token) => {
      asked.joins.push(token);
      return new Promise(() => {});
    },
    current_version: () => VERSION,
    // version.rs `older`: an answer naming no version is an older computer's.
    older: (version) => version === undefined,
    // A lend's label, one DNS label per lender and name (svrngs-core
    // `Label::lent`), and `is_label` reading one back.
    label: (key, name) => `${name}0${key.slice(0, 6)}`,
    is_label: (raw) => /^[a-z]+0[0-9a-f]{6}$/.test(raw),
    actor: () => BO,
  };
  return asked;
}

// The groups bo's phone holds, as page.mjs `hold` keeps one (store.js `keep`).
const house = { id: 'house', name: 'house', person: 'bo', token: 'TOKEN' };
const KEY = new Uint8Array(32);
// The effects held at the edge, never run (see the head of this file).
const EDGE = ['hand', 'again', 'cold', 'forget'];

// Whether `promise` settles while nothing waits on a clock: every fake
// here answers within a few turns of the loop.
async function settled(promise) {
  let done = false;
  let value;
  promise.then(
    (v) => ((done = true), (value = { v })),
    (e) => ((done = true), (value = { e })),
  );
  for (let turn = 0; turn < 1000 && !done; turn++) await new Promise((r) => setImmediate(r));
  return done ? value : null;
}

/**
 * The page opened at `fragment` on a phone, or on a computer when `fine`,
 * holding `kept`, last opened first: each effect run in turn until none is
 * left. Answers what the page did and says, and `send`, which steps a
 * message as a press does.
 */
async function opened(fragment, { kept = [house], fine = false } = {}) {
  globalThis.location = new URL('https://page.example/');
  const written = [];
  globalThis.navigator = { clipboard: { writeText: async (text) => void written.push(text) } };
  const copy = machine('COPY', COPY, { at: { copy: 'idle' }, said: [] });
  const io = {
    ...parts.io,
    look: (f) => ({
      type: 'looked',
      fragment: f.fragment,
      kept,
      missing: false,
      cold: false,
      linked: issued(globalThis.__core, f.fragment.split('&')[0]),
    }),
    lock: () => ({ type: 'held', why: 'one tab here' }),
    open: ({ group }) => (hold(KEY), { type: 'opened', group }),
    keep: () => ({ type: 'kept' }),
    others: ({ open, id }) => ({
      type: 'others',
      open,
      label: globalThis.__core.is_label(open),
      others: kept.filter((group) => group.id !== id && group.addresses !== undefined),
    }),
  };
  const page = { state: parts.first({ fine }), ran: [], refused: [], pending: [], written };
  // Each effect after the step that gave it, in its order, as ui/machine.js
  // `machine` runs them; a refusal is kept, as its trace says it.
  const run = async (effects) => {
    for (let queue = [...effects]; queue.length;) {
      const effect = queue.shift();
      page.ran.push(effect);
      if (EDGE.includes(effect.run)) continue;
      const answered = await settled(Promise.resolve().then(() => io[effect.run](effect)));
      if (!answered) {
        page.pending.push(effect.run);
        continue;
      }
      const message = answered.v ?? {
        type: effect.failed,
        n: effect.n,
        why: String(answered.e?.message),
      };
      const r = parts.step(page.state, message);
      if (r.refused) page.refused.push(`${message.type}: ${r.refused}`);
      else [page.state, queue] = [r.state, [...queue, ...r.effects]];
    }
  };
  page.send = async (message) => {
    const r = parts.step(page.state, message);
    if (r.refused) {
      page.refused.push(`${message.type}: ${r.refused}`);
      return;
    }
    page.state = r.state;
    await run(r.effects);
  };
  await page.send({ type: 'start', fragment });
  // What a person reads of the page now: #status, #response, and a place.
  page.heading = () => parts.heading(page.state);
  page.response = () => parts.response(page.state);
  page.drawn = (View) => tree(View({ s: page.state, send: page.send }), copy);
  page.press = async (View, words) => {
    const [button] = all(page.drawn(View), (n) => n.tag === 'button' && text([n]) === words);
    assert.ok(button, `nothing to press saying ${words}`);
    // A press here steps PAGE through to its last effect (`send`); Copy's
    // own machine answers its write a turn or two on.
    await button.props.onClick();
    await settled(new Promise((r) => setImmediate(r)));
  };
  return page;
}

// A drawing as a reader meets it: each element `{ tag, props, kids }`, each
// component drawn, a Copy as its machine `copy` has it now.
const tree = (v, copy) => {
  if (v == null || typeof v === 'boolean') return [];
  if (Array.isArray(v)) return v.flatMap((one) => tree(one, copy));
  if (typeof v !== 'object') return [String(v)];
  if (v.type === Copy) return tree(drawn(copy.get(), copy.dispatch, v.props), copy);
  if (typeof v.type === 'function') return tree(v.type(v.props), copy);
  return [{ tag: v.type, props: v.props, kids: tree(v.props.children, copy) }];
};
// Its words, as `textContent` reads them.
const text = (nodes) => nodes.map((n) => (typeof n === 'string' ? n : text(n.kids))).join('');
// Each element under `nodes` that `holds`.
const all = (nodes, holds) =>
  nodes.flatMap((n) =>
    typeof n === 'string' ? [] : [...(holds(n) ? [n] : []), ...all(n.kids, holds)],
  );
const tagged = (nodes, tag) => all(nodes, (n) => n.tag === tag);
const byId = (nodes, id) => all(nodes, (n) => n.props.id === id)[0];

// browser::the_group_s_page_tells_a_removed_person: ada's computer serves a
// removed person's dial only to refuse it, 403 `removed` (node.rs:495-499,
// door::gone; tests/door/refusals.rs, "by dial"); the page says so.
test('a removed person’s read is refused as removed, and the page says they are no longer in the group', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const asked = runtime({
    computers: {
      [ADA]: refusal(403, 'removed', 'You are no longer in this group'),
    },
  });
  const page = await opened('TOKEN');
  assert.deepEqual(page.heading(), ['You are no longer in house.', 'bad']);
  assert.deepEqual(
    asked.fetched.map(([key, method, path]) => [key, method, path]),
    [[ADA, 'GET', '/people/lends']],
  );
});

// browser::the_group_s_page_shows_every_removal_and_ask: cy asked to
// remove dee, then ada, the keeper, removed dee, as svrngs-runtime
// register.rs `removals_in` (:182) says each (`RemovalSaid`, snake_case),
// listed in time's order.
test('every removal and ask to remove is listed on the group’s page, by name', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const at = 1_791_000_100;
  const removals = [
    {
      said: 'asked',
      by: 'cy',
      by_key: CY,
      by_joined: { at: 1_791_000_010 },
      of: 'dee',
      of_key: DEE,
      of_joined: { at: 1_791_000_020 },
      at,
    },
    {
      said: 'removed',
      by: 'ada',
      by_key: ADA,
      by_joined: 'started',
      of: 'dee',
      of_key: DEE,
      of_joined: { at: 1_791_000_020 },
      at: at + 5,
    },
  ];
  const people = [row('ada', ADA, 'started'), row('bo', BO), row('cy', CY)];
  runtime({ computers: { [ADA]: answer({ removals, people }) } });
  const page = await opened('TOKEN');
  assert.deepEqual(page.heading(), ['You’re in house, as bo', 'good']);
  const lines = tagged(page.drawn(Removals), 'li').map((li) => text([li]));
  assert.deepEqual(
    lines.map((line) => line.split(' on ')[0]),
    ['cy asked to remove dee', 'ada removed dee'],
  );
});

// browser::the_group_s_page_says_a_refusal_apart_from_no_answer: ada's
// computer names dee's as not answering, eve's as not having heard of
// hers, cy's as answering without saying (node.rs `Unlisted`, as
// tests/group_not_mesh.rs
// a_computer_that_refuses_is_said_apart_from_one_that_does_not_answer
// sorts three computers into them).
test('a computer that did not answer, one that has not heard of yours and one that did not say are each said apart', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const people = [
    row('ada', ADA, 'started'),
    row('bo', BO),
    row('cy', CY),
    row('dee', DEE),
    row('eve', EVE),
  ];
  runtime({
    computers: {
      [ADA]: answer({
        people,
        unanswered: [whom('dee', DEE)],
        unheard: [whom('eve', EVE)],
        refused: [whom('cy', CY)],
      }),
    },
  });
  const page = await opened('TOKEN');
  assert.deepEqual(page.heading(), ['You’re in house, as bo', 'good']);
  assert.equal(
    page.response(),
    [
      'dee’s computer did not answer, so what they lend is not listed.',
      'eve’s computer has not heard of the computer you reached yet, so what they lend is not listed yet.',
      'cy’s computer answered without saying what they lend, so it is not listed.',
    ].join('\n'),
  );
  // Answered by every computer, none lending: said once, as
  // browser::a_page_holding_a_standing_key_dials_the_node_and_gets_its_lends
  // reads it after its real dial.
  runtime({ computers: { [ADA]: answer() } });
  assert.equal((await opened('TOKEN')).response(), 'Nobody in house lends anything yet.');
});

// The group as bo's phone kept it after a read that every computer
// answered: ada's computer made the link and lends a pad, cy's a shelf,
// so the page dials both (dial.js `reach`). Its `keep` effect's group is
// what store.js keeps.
async function keptAfterReading(lent) {
  runtime({ computers: { [ADA]: answer({ lent, people: [...answer().people, row('cy', CY)] }) } });
  const page = await opened('TOKEN');
  assert.deepEqual(page.heading(), ['You’re in house, as bo', 'good']);
  return page.ran.findLast((effect) => effect.run === 'keep').group;
}
const pad = lend('ada', ADA, 'pad');
const shelf = lend('cy', CY, 'shelf');

// browser::a_computer_that_did_not_answer_is_named_by_its_person: with
// ada's and cy's computers both stopped, each is said by whose it is, and
// no word on the page names the relay the dials went through.
test('a computer that did not answer is named by its person, never by the relay', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const group = await keptAfterReading([pad, shelf]);
  const asked = runtime({ computers: { [ADA]: closed, [CY]: unanswered } });
  const page = await opened('TOKEN', { kept: [group] });
  assert.deepEqual(asked.dials.toSorted(), [ADA, CY], 'both dialled');
  const said =
    'ada’s computer did not answer, so it may be off. cy’s computer did not answer, so it may be off. This page keeps asking.';
  assert.deepEqual(page.heading(), [said, 'bad']);
  assert.deepEqual(
    page.ran.map((effect) => effect.run).slice(-1),
    ['again'],
    'asked again, the lock kept',
  );
  const words = [
    page.heading()[0],
    page.response(),
    text(page.drawn(GroupDoor)),
    text(page.drawn(LentIndex)),
  ].join('\n');
  for (const host of ['127.0.0.1', '4433', 'relay'])
    assert.ok(!words.includes(host), `${host} said: ${words}`);
});

// browser::the_phone_goes_through_the_link_s_computer_while_it_answers:
// cy's computer stopped, never answering, ada's, the link's, answers; the
// page reaches the group through it without waiting on cy's, and names no
// other computer.
test('while the link’s computer answers, the page goes through it, naming no one else’s', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const group = await keptAfterReading([pad, shelf]);
  const asked = runtime({ computers: { [ADA]: answer({ lent: [pad] }), [CY]: SILENT } });
  const page = await opened('TOKEN', { kept: [group] });
  assert.deepEqual(asked.dials.toSorted(), [ADA, CY], 'both dialled at once');
  assert.deepEqual(page.pending, [], 'nothing waits on cy’s');
  assert.deepEqual(page.heading(), ['You’re in house, as bo', 'good']);
});

// The decisions of browser::the_phone_reaches_whoever_s_awake, which stays
// for its time over a real relay and the shelf opened at its own origin:
// ada's, the link's, never answers, cy's does, naming ada's unanswered; the
// page reaches the group through cy's at once, no clock run, and says so.
test('with the link’s computer silent, the page goes through whoever answers, at once, and says whose', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const group = await keptAfterReading([pad, shelf]);
  const people = [...answer().people, row('cy', CY)];
  runtime({
    computers: {
      [ADA]: SILENT,
      [CY]: answer({ lent: [shelf], people, unanswered: [whom('ada', ADA)] }),
    },
  });
  const page = await opened('TOKEN', { kept: [group] });
  assert.deepEqual(page.pending, [], 'no dial waits out the link’s cap');
  assert.deepEqual(page.heading(), ['You’re in house, as bo, through cy’s computer', 'good']);
  assert.match(page.response(), /ada’s computer did not answer/);
  assert.deepEqual(
    tagged(page.drawn(LentIndex), 'button').map((b) => text([b])),
    ['cy’s shelf (k)'],
  );
});

// browser::a_link_naming_no_relay_is_refused_and_n0_never_reached: a token
// naming no relay, from a computer that had n0's lookup, reads as lib.rs
// `relay_of` answers it, none (:477-479); the page says so in a person's
// words and binds no endpoint, so nothing is dialled at all.
test('a link naming no relay is refused in a person’s words, and nothing is dialled', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const asked = runtime({ computers: { [ADA]: answer() }, relayless: true });
  const page = await opened('TOKEN');
  assert.deepEqual(page.heading(), [
    'This link was made by an older svrngs. Ask whoever sent it for a new one.',
    'bad',
  ]);
  assert.deepEqual(asked, { endpoints: [], dials: [], fetched: [], joins: [] });
});

// browser::the_group_s_page_draws_the_door_and_the_index: pia's phone,
// joined from sam's link on a day it keeps, through ada's computer: ada and
// cy on, bo's computer never answering, pia in from a phone; cy's shelf a
// line, its open and what opening it does; the heading saying only that
// pia is in, and no address to keep on a phone.
test('the door and the index on a phone: who is on, whose group since when, and each lend a line', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const people = [
    row('ada', ADA, 'started'),
    row('bo', BO),
    row('cy', CY),
    row('pia', EVE, { at: 1_791_000_500 }, 'browser'),
  ];
  runtime({
    computers: {
      [ADA]: answer({ people, lent: [shelf], unanswered: [whom('bo', BO)] }),
    },
  });
  const pias = { ...house, person: 'pia', maker: 'sam', since: 1_791_000_000 };
  const page = await opened('TOKEN', { kept: [pias] });
  assert.deepEqual(page.heading(), ['You’re in house, as pia', 'good']);
  const door = page.drawn(GroupDoor);
  assert.ok(text(door).startsWith('house (sam’s, since '), text(door));
  const cells = tagged(door, 'li').map((li) => [li.props.class, text([li])]);
  assert.deepEqual(
    cells.toSorted((a, b) => a[1].localeCompare(b[1])),
    [
      ['on', 'ada on now'],
      ['off', 'bo did not answer'],
      ['on', 'cy on now'],
      ['phone', 'pia in from a phone'],
    ],
  );
  const index = tagged(page.drawn(LentIndex), 'tr').map((tr) =>
    tagged([tr], 'td').map((td) => text([td])),
  );
  assert.deepEqual(index, [
    ['cy’s shelf (k) Opening it lets it write its own part of house’s record, as you.'],
  ]);
});

// browser::the_page_opened_with_no_link_offers_a_founder_s_line, both
// engines: with no link and no group, a computer is offered the line that
// gets svrngs, the page's address where the link goes, saying it is a
// founder's, and Copy writes it; a phone, no line; each says to open the
// group's link. And the computer half of
// browser::the_page_offers_a_computer_one_line_and_sends_the_token_nowhere:
// a link's line carries its `#` part, and a phone is offered none.
test('a computer is offered one line, a founder’s with no link and a joiner’s with one, and Copy writes it; a phone none', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const origin = 'https://page.example/';
  runtime({ tokens: { TOKEN: ADA }, computers: {} });
  for (const [fragment, says, line, words] of [
    ['', 'founds', `curl -fsSL ${origin}install.sh | sh -s -- '${origin}'`, 'make your group'],
    [
      'TOKEN',
      'joins',
      `curl -fsSL ${origin}install.sh | sh -s -- '${origin}#TOKEN'`,
      'it gets svrngs, joins you',
    ],
  ]) {
    const computer = await opened(fragment, { kept: [], fine: true });
    const drawn = computer.drawn(ComputerLine);
    const section = byId(drawn, 'computer');
    assert.equal(section.props.hidden, false, `${fragment}: offered`);
    assert.equal(text([byId(drawn, 'line')]), line);
    const shown = tagged(drawn, 'p').filter((p) => !p.props.hidden);
    assert.deepEqual(
      shown.map((p) => p.props.id),
      [says],
    );
    assert.ok(text(shown).includes(words), text(shown));
    assert.equal(text([byId(drawn, 'copy')]), 'Copy');
    await computer.press(ComputerLine, 'Copy');
    assert.deepEqual(computer.written, [line], 'Copy writes the line');
    assert.equal(text([byId(computer.drawn(ComputerLine), 'copy')]), 'Copied');
    const phone = await opened(fragment, { kept: [] });
    assert.equal(byId(phone.drawn(ComputerLine), 'computer').props.hidden, true, 'a phone, none');
    assert.equal(phone.state.line, undefined);
    assert.ok(
      phone.refused.includes('offer: no line offered: not a computer, or not a token'),
      phone.refused.join('\n'),
    );
    if (!fragment)
      for (const page of [computer, phone])
        assert.deepEqual(page.heading(), ['Open this page from your group’s link.', 'bad']);
  }
});

// browser::a_person_in_two_groups_opens_each_ones_lend_where_it_is_lent,
// both engines: bo's browser holds house, ada's, and club, another ada's,
// each lending `app`, club opened last. What it opens is decided here, by
// the groups' addresses as last read (enter.js `lendingIn`); its landing at
// the lend's origin is the hand's, which
// browser::a_lends_address_opens_cold_in_a_browser_that_joined holds.
test('in two groups, a lend’s address opens in the group that lends it, an alias of both is offered, and neither is said of both', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const tokens = { TOKEN: ADA, CLUB: ADA2 };
  const app = lend('ada', ADA, 'app');
  const theirs = lend('ada', ADA2, 'app', { groups: ['club'] });
  const computers = {
    [ADA]: answer({ lent: [app] }),
    [ADA2]: answer({ lent: [theirs], people: [row('ada', ADA2, 'started'), row('bo', BO)] }),
  };
  // Each group as the phone kept it once read.
  const club = { id: 'club', name: 'club', person: 'bo', token: 'CLUB' };
  const read = async (group) => {
    runtime({ tokens, computers });
    const page = await opened(group.token, { kept: [group] });
    return page.ran.findLast((effect) => effect.run === 'keep').group;
  };
  const kept = [await read(club), await read(house)];
  assert.deepEqual(
    kept.map((group) => group.addresses),
    [
      ['app0a2a2a2', 'app@ada'],
      ['app0a1a1a1', 'app@ada'],
    ],
  );
  const handed = (page) =>
    page.ran.filter((effect) => effect.run === 'hand').map((h) => [h.group.id, h.label, h.at]);

  // House's app's address, opened cold (among.js `cold` sends it so).
  runtime({ tokens, computers });
  const cold = await opened('open=app0a1a1a1&at=%2F%3Fn%3D1&cold', { kept });
  assert.deepEqual(handed(cold), [['house', 'app0a1a1a1', '/?n=1']], 'house’s, though club last');

  // An alias lent in both: each offered, none opened until one is picked.
  runtime({ tokens, computers });
  const both = await opened('open=app@ada', { kept });
  assert.deepEqual(both.heading(), [
    'What that address opens is lent in club and in house.',
    'good',
  ]);
  assert.equal(both.response(), 'Open it in the one you mean.');
  const offered = tagged(both.drawn(GroupDoor), 'button').map((b) => text([b]));
  assert.deepEqual(offered, ['Open it in club', 'Open it in house']);
  assert.deepEqual(
    both.ran.filter((effect) => ['lock', 'read', 'hand'].includes(effect.run)),
    [],
    'nothing opened before a pick',
  );
  await both.press(GroupDoor, 'Open it in house');
  assert.deepEqual(handed(both), [['house', 'app0a1a1a1', '/']], 'picked, house’s app');

  // A label lent in neither: said of both.
  runtime({ tokens, computers });
  const neither = await opened('open=nothing0e5e5e5', { kept });
  assert.deepEqual(handed(neither), []);
  assert.equal(
    neither.response(),
    'What that address opens is not lent in club now, nor in house when you last opened it.',
  );
});
