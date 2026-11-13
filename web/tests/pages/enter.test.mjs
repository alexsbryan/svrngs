// The phone's enter (page-open (b); web/page/enter.js), with no browser: PAGE
// composed from the flows, each `looked` routed as web/app.js `opening`
// routes it (:667-706), the tab's lock held by one tab, and the read's
// refusals said by whose computer gave them, an older answer refused by
// name. The edge's io is not run here: each step's effects are read as data.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { register } from 'node:module';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';

register('./layout.mjs', import.meta.url);
const { compose } = await import('../../page/page.js?v=__VERSION__');
const { response } = await import('../../page/enter.js?v=__VERSION__');
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

const house = { id: 'g1', name: 'house', person: 'bo', token: 'TOKEN', addresses: ['notes@ada'] };
const lake = {
  id: 'g2',
  name: 'lake',
  person: 'bo',
  token: 'LAKE',
  addresses: ['notes@ada'],
  maker: 'cy',
};
const first = parts.first({ fine: false });
// What `looked` answers for `fragment`: each effect as [run, its message's type, its words].
const looked = (fragment, more = {}) =>
  parts
    .step(first, { type: 'looked', fragment, kept: [house], linked: true, ...more })
    .effects.map((f) => [f.run, f.m?.type, f.m?.said ?? f.m?.token ?? f.id]);

test('each fragment is routed as web/app.js opening routes it', () => {
  assert.deepEqual(
    looked('TOKEN'),
    [
      ['send', 'offer', 'TOKEN'],
      ['lock', undefined, 'g1'],
    ],
    'a held link: its line offered, its group held',
  );
  assert.deepEqual(looked(''), [['lock', undefined, 'g1']], 'no link: the group last opened');
  assert.deepEqual(
    looked('NEW'),
    [
      ['send', 'offer', 'NEW'],
      ['send', 'join', 'NEW'],
    ],
    'a new link joins',
  );
  assert.deepEqual(looked('', { kept: [] }), [
    ['send', 'offer', ''],
    ['send', 'failed', 'Open this page from your group’s link.'],
  ]);
  assert.deepEqual(looked('TOKEN&x=1&y=2'), [
    [
      'send',
      'failed',
      'This address asks for something this page can’t read: x, y. Open your group’s link again.',
    ],
  ]);
  assert.deepEqual(looked('TOKEN', { linked: false }), [
    ['send', 'failed', 'This link isn’t one svrngs can read. Ask whoever sent it for a new one.'],
  ]);
  assert.deepEqual(looked('', { missing: true }), [
    [
      'send',
      'failed',
      'There’s no page at this address. Open your group’s link to reach your group.',
    ],
  ]);
  assert.deepEqual(looked('visit=V'), [['send', 'visit', 'V']]);
  assert.deepEqual(looked('', { cold: true }), [['send', 'cold', undefined]]);
  assert.deepEqual(
    looked('open=notes@ada&at=/', { kept: [house, lake] }),
    [['send', 'among', undefined]],
    'lent in two groups: each offered',
  );
  assert.deepEqual(looked('open=films@ada', { kept: [] }), [
    [
      'send',
      'failed',
      'This browser hasn’t joined your group yet, so it can’t open this. Open your group’s link from the chat here first, then this again.',
    ],
  ]);
});

// `s` stepped through each message in turn, each step taken.
const through = (s, ...ms) =>
  ms.reduce((at, m) => {
    const r = parts.step(at, m);
    assert.equal(r.refused, undefined, `${m.type}: ${r.refused}`);
    return r.state;
  }, s);

test('one tab holds the group: another says so, and takes it here when asked', () => {
  const held = parts.step(first, { type: 'hold', group: lake });
  assert.deepEqual(held.effects, [{ run: 'lock', id: 'g2', failed: 'failed' }]);
  const there = through(held.state, { type: 'elsewhere' });
  assert.deepEqual(parts.heading(there), ['You have lake (cy’s) open in another tab', 'good']);
  assert.equal(parts.step(there, { type: 'held' }).refused, 'no lock was asked for');
  const taking = parts.step(there, { type: 'take' });
  assert.deepEqual(taking.effects, [{ run: 'lock', id: 'g2', steal: true, failed: 'failed' }]);
  const holding = through(taking.state, { type: 'held' });
  assert.deepEqual(holding.at, { ...first.at, tab: 'holding' });
  assert.equal(
    parts.step(holding, { type: 'take' }).refused,
    'the group is not open in another tab',
  );
  assert.equal(
    parts.step(first, { type: 'reread' }).refused,
    'the list is unasked, in a tab unheld',
  );
});

const body = {
  version: 3,
  lent: [],
  unanswered: [{ person: 'cy', person_key: 'kc' }],
  unheard: [],
  refused: [],
  older: [],
  removals: [],
  visiting: [],
  people: [],
  shelf: [],
  changed: [],
};
const ada = { key: 'ka', person: 'ada', relay: null };

test('the read: its refusals said, an older answer refused by name, and what it read', () => {
  const opened = through(first, { type: 'hold', group: house }, { type: 'held' });
  const reading = parts.step(opened, { type: 'opened', group: house });
  assert.deepEqual(reading.effects, [{ run: 'read', group: house, n: 1, failed: 'unread' }]);
  assert.deepEqual(parts.heading(reading.state), ['Reaching house…']);
  const said = (m) =>
    parts
      .step(reading.state, { type: 'read', n: 1, body, lent: [], ...m })
      .effects.map((f) => f.m?.said ?? f.run);
  assert.deepEqual(said({ ok: false, refusal: 'removed' }), [
    'release',
    'You are no longer in house.',
  ]);
  assert.deepEqual(said({ ok: false, via: ada }), [
    'release',
    'ada’s computer said no when you asked what house lends.',
  ]);
  assert.deepEqual(said({ ok: true, older: true }), [
    'release',
    'The computer that sent your link runs an older svrngs, so this page cannot show what house lends. Installing svrngs again there updates it.',
  ]);
  const again = parts.step(
    through(reading.state, { type: 'read', n: 1, ok: true, via: ada, body, lent: [] }),
    { type: 'reread' },
  );
  assert.deepEqual(
    parts.heading(again.state),
    ['You’re in house, as bo, through ada’s computer', 'good'],
    'the last read stands while it is read again',
  );
  assert.equal(
    parts.response(again.state),
    'cy’s computer did not answer, so what they lend is not listed.',
  );
  assert.equal(
    parts.step(again.state, { type: 'read', n: 1, ok: true, body, lent: [] }).refused,
    'list 1 answered after list 2 was asked',
  );
});

test('back on screen the list is read again, and a read again that fails keeps the list, saying why', () => {
  const read = through(
    first,
    { type: 'hold', group: house },
    { type: 'held' },
    { type: 'opened', group: house },
    { type: 'read', n: 1, ok: true, body, lent: [] },
  );
  assert.equal(
    parts.step(first, { type: 'shown' }).refused,
    'the list is unasked, in a tab unheld',
    'nothing to read again before the first',
  );
  const again = parts.step(read, { type: 'shown' });
  assert.deepEqual(again.effects, [{ run: 'read', group: read.group, n: 2, failed: 'unread' }]);
  const failed = parts.step(again.state, {
    type: 'unread',
    n: 2,
    said: 'ada’s computer did not answer. Try again later.',
  });
  assert.deepEqual(
    failed.effects,
    [],
    'the tab keeps the group, and nothing is said over the heading',
  );
  assert.deepEqual(parts.heading(failed.state), ['You’re in house, as bo', 'good']);
  assert.equal(
    parts.response(failed.state),
    'What house lends cannot be read again just now: ada’s computer did not answer. Try again later.\ncy’s computer did not answer, so what they lend is not listed.',
  );
  const back = through(
    failed.state,
    { type: 'shown' },
    {
      type: 'read',
      n: 3,
      ok: true,
      body: { ...body, unanswered: [] },
      lent: [{ key: 'ka', name: 'notes', person: 'ada', label: 'l' }],
    },
  );
  assert.equal(
    parts.response(back),
    '',
    'read again: the line goes, and what is lent now is in the list',
  );
  assert.deepEqual(back.group.addresses, ['l', 'notes@ada']);
});

test('a read no computer answers keeps the tab and is asked again until one does', () => {
  const reading = through(
    first,
    { type: 'hold', group: house },
    { type: 'held' },
    { type: 'opened', group: house },
  );
  const quiet = 'ada’s computer did not answer, so it may be off.';
  const silent = parts.step(reading, {
    type: 'unread',
    n: 1,
    said: `${quiet} Try again later.`,
    silence: quiet,
  });
  assert.deepEqual(
    silent.effects,
    [{ run: 'again', failed: 'failed' }],
    'the lock is kept and nothing fails the page',
  );
  assert.deepEqual(parts.heading(silent.state), [`${quiet} This page keeps asking.`, 'bad']);
  const again = parts.step(silent.state, { type: 'reread', why: 'no computer answered' });
  assert.deepEqual(again.effects, [{ run: 'read', group: house, n: 2, failed: 'unread' }]);
  assert.deepEqual(parts.heading(again.state), [`${quiet} This page keeps asking.`, 'bad']);
  const answered = through(again.state, { type: 'read', n: 2, ok: true, body, lent: [] });
  assert.deepEqual(parts.heading(answered), ['You’re in house, as bo', 'good']);
  const stale = parts.step(answered, { type: 'shown' }).state;
  assert.deepEqual(
    parts.step(stale, { type: 'unread', n: 3, said: quiet, silence: quiet }).effects,
    [{ run: 'again', failed: 'failed' }],
    'a read again no computer answers is asked again too',
  );
  const refused = 'This browser refused what svrngs needs to reach your group.';
  assert.deepEqual(
    parts
      .step(reading, { type: 'unread', n: 1, said: refused })
      .effects.map((f) => f.m?.said ?? f.run),
    ['release', refused],
    'a refusal is said, never asked again',
  );
});

// Picked, the offer and its line go in the click's own step, as v1's click
// emptied #door and #said before `hold` (web/app.js@19b63b2b:586): the browser
// driver reads the next line once `hold`'s lock answers (cold.mjs `landed`).
test('a group picked from the offer leaves it in the same step', () => {
  const offered = through(first, {
    type: 'among',
    groups: [house, lake],
    wants: { open: 'notes@ada' },
  });
  assert.equal(parts.response(offered), 'Open it in the one you mean.');
  const r = parts.step(offered, { type: 'pick', group: house, wants: { open: 'notes@ada' } });
  assert.equal(r.refused, undefined, r.refused);
  assert.equal(parts.response(r.state), '', 'no line left to read as the answer');
  assert.deepEqual(
    r.effects.map((f) => [f.run, f.m?.type, f.m?.group?.id]),
    [['send', 'hold', 'g1']],
  );
  assert.match(parts.step(r.state, { type: 'pick', group: lake }).refused, /nothing is offered/);
});

// KEEP_GOING_AT_HOME 4-6: only what a reply names is kept, each relay the
// page's own check takes and at most eight, so a flood cannot slow a phone;
// a later reply that names fewer drops the rest; an older computer's reply,
// which names none, leaves the phone as it was. `kept` is what the phone
// dials beside its link.
test('the relays a reply names are kept, capped and checked, and a later reply drops the rest', async () => {
  const { MEETS_KEPT, meeting } = await import('../../page/enter.js?v=__VERSION__');
  const said = [];
  const core = { relay_taken: (one) => one.startsWith('https://') };
  globalThis.__trace = (line) => said.push(line);
  const flood = Array.from({ length: 500 }, (_, n) => `https://liar${n}.example/`);
  assert.equal(MEETS_KEPT, 8);
  assert.deepEqual(meeting(core, flood), flood.slice(0, 8), 'a flood is cut to the cap');
  assert.ok(
    said.some((line) => line.includes('8 of 500')),
    said.join('\n'),
  );
  assert.deepEqual(
    meeting(core, ['http://plain.example/', 7, 'https://home.example/']),
    ['https://home.example/'],
    'each is checked, what is not a relay with them',
  );
  assert.equal(meeting(core, []), undefined, 'a reply naming none leaves what is kept');
  assert.equal(meeting(core, ['http://plain.example/']), undefined, 'as does one it refuses');
  const twice = ['https://home.example/', 'https://home.example/', 'https://home.example/'];
  assert.deepEqual(meeting(core, twice), ['https://home.example/'], 'a relay named again is one');
  assert.equal(meeting(core, undefined), undefined, 'an older computer says nothing of them');
  assert.equal(meeting(core, null), undefined, 'nor one whose record cannot be read');
  assert.equal(meeting(core, 'https://home.example/'), undefined);
  delete globalThis.__trace;

  const reading = (group) =>
    parts.step(through(first, { type: 'hold', group }, { type: 'held' }), {
      type: 'opened',
      group,
    }).state;
  const kept = (group, m) => {
    const { effects } = parts.step(reading(group), {
      type: 'read',
      n: 1,
      ok: true,
      body,
      lent: [],
      ...m,
    });
    return effects.find((f) => f.run === 'keep').group.meets;
  };
  const home = ['https://home.example/'];
  assert.deepEqual(kept(house, { meets: home }), home, 'learned, with no new link');
  assert.deepEqual(kept({ ...house, meets: home }, { meets: [] }), [], 'dropped when unnamed');
  assert.deepEqual(kept({ ...house, meets: home }, {}), home, 'an older reply keeps what is kept');
  assert.equal(kept(house, {}), undefined, 'and a phone that learned none has none');
});

test('what keeps a phone from going on at home is said under what its group lends', () => {
  const unready = ['films is not saved here: open it once while online.'];
  const lent = [{ name: 'films', label: 'l' }];
  const none = { unanswered: [], unheard: [], refused: [], behind: [], people: [] };
  const state = { ...none, group: { name: 'house' }, lent, stale: '', unready };
  assert.deepEqual(response.enter(state), unready);
});

// Read and kept, what `fragment` names is handed to the lend's origin
// (index.js `hand`), as the group's page opens it in a browser: each step
// the page takes from the address to the hand, its effects run as the
// page's io would answer them, `apart` as readLends applies it.
// Moved from browser/main.rs
// `a_links_token_with_what_it_opens_opens_it_and_a_garbled_fragment_says_why`;
// that the hand lands at its path stays in the browser's cold-open test.
const app = { key: 'ka', name: 'app', person: 'ada', kind: 'k', label: 'app-label' };
const founder = {
  person: 'ada',
  key: 'ka',
  person_key: 'ka',
  joined: 'started',
  device: 'computer',
};
const handed = (fragment, lent = [app], people = [founder], state = first) => {
  const kept = parts.step(state, { type: 'looked', fragment, kept: [house], linked: true });
  assert.equal(kept.refused, undefined, kept.refused);
  const effects = kept.effects.map((f) => [f.run, f.m?.type]);
  // The line offered is the computer flow's, which a phone's page refuses.
  const read = through(
    kept.state,
    { type: 'held' },
    { type: 'opened', group: house },
    { type: 'read', n: 1, ok: true, body: { ...body, unanswered: [], people }, lent },
  );
  const wanted = parts.step(read, { type: 'kept' });
  const sent = wanted.effects.filter((f) => f.run === 'send').map((f) => f.m);
  const opened = sent.reduce((at, m) => {
    const r = parts.step(at.state, m);
    assert.equal(r.refused, undefined, `${m.type}: ${r.refused}`);
    return r;
  }, wanted);
  const open = opened.effects.find((f) => f.m?.type === 'open')?.m;
  const hand = open ? parts.step(opened.state, open).effects : [];
  return { effects, hand: hand.map((f) => [f.run, f.label, f.at]), state: opened.state };
};

test('a link’s token with what it opens opens it, at its path', () => {
  const opened = handed('TOKEN&open=app@ada');
  assert.deepEqual(
    opened.effects,
    [
      ['send', 'offer'],
      ['lock', undefined],
    ],
    'its line offered, its group held with what it opens',
  );
  assert.deepEqual(opened.hand, [['hand', 'app-label', '/']], 'read, it opens');
  assert.deepEqual(
    handed('TOKEN&open=app@ada&at=/a/path?n=1').hand,
    [['hand', 'app-label', '/a/path?n=1']],
    'and at its path',
  );
  assert.deepEqual(handed('TOKEN&open=app-label&at=/x').hand, [['hand', 'app-label', '/x']]);
});

// The edge's `issued` (enter.js), cut from its text: what io.look gives
// `looked` as `linked`, with a core that answers as web/src/lib.rs
// `issuer_of` does, refusing what svrngs-core `Token::decode` cannot read
// (link_tests.rs, an unknown format byte: "not-a-token" decodes to 0x9e).
const ENTER = readFileSync(new URL('../../page/enter.js', import.meta.url), 'utf8');
const issued = runInNewContext(`${/^const issued = [^]*?\n};\n/m.exec(ENTER)?.[0]}\nissued`, {
  Boolean,
});
const core = {
  issuer_of: (token) => {
    if (token === 'TOKEN') return 'ka';
    throw new Error('not a link’s token');
  },
};
// The heading a fragment draws, as io.look reads it: its token the part
// before the first `&`.
const headed = (fragment) => {
  const linked = issued(core, fragment.split('&')[0]);
  const looked = parts.step(first, { type: 'looked', fragment, kept: [house], linked });
  return parts.heading(through(looked.state, ...looked.effects.map((f) => f.m)));
};

test('a fragment the page cannot read says why in the heading, never the browser’s line', () => {
  assert.equal(issued(core, 'TOKEN'), true);
  assert.deepEqual(headed('not-a-token'), [
    'This link isn’t one svrngs can read. Ask whoever sent it for a new one.',
    'bad',
  ]);
  assert.deepEqual(headed('opne=app@ada'), [
    'This address asks for something this page can’t read: opne. Open your group’s link again.',
    'bad',
  ]);
});

// Moved from browser/main.rs `a_lends_alias_is_one_in_every_time_zone`, in
// both engines: two people called ada each lend `app`, the second joined at
// `AT`, 4 Oct in UTC (`date -u -r 1791100000`), 4 Oct at UTC+14 and 3 Oct
// at UTC-11. In a browser whose clock reads in either zone, the page's
// addresses, the alias it gives a computer to keep and what `#open=<alias>`
// hands are one. That the alias opens the app in a browser stays in
// `a_lends_alias_opens_it_from_the_groups_page`, in both engines.
test('a lend’s alias is one in every time zone: the addresses kept and what it opens', async () => {
  const { apart } = await import('../../question.js?v=__VERSION__');
  const { view } = await import('../../page/index.js?v=__VERSION__');
  const AT = 1_791_100_000;
  const other = { key: 'kb', name: 'app', person: 'ada', kind: 'k', label: 'other-label' };
  const people = [founder, { ...founder, key: 'kb', person_key: 'kb', joined: { at: AT } }];
  // Each `href` the drawing `v` links to, its components drawn.
  const hrefs = (v) => {
    if (Array.isArray(v)) return v.flatMap(hrefs);
    if (!v || typeof v !== 'object') return [];
    if (typeof v.type === 'function') return hrefs(v.type(v.props));
    return [...(v.type === 'a' ? [v.props.href] : []), ...hrefs(v.props?.children)];
  };
  globalThis.location = { origin: 'https://page.example', pathname: '/' };
  // The zone this process had, given back after, so no other law reads in
  // these (an unset TZ assigned `undefined` would read as "undefined").
  const zone = process.env.TZ;
  const seen = [];
  try {
    for (const TZ of ['Pacific/Kiritimati', 'Pacific/Pago_Pago']) {
      process.env.TZ = TZ;
      const lent = [{ ...app }, { ...other }];
      apart(lent, people);
      const { state } = handed('', lent, people, parts.first({ fine: true }));
      const opens = handed('open=app@ada+(joined+4+Oct)', lent, people).hand;
      const keep = hrefs(view.lent({ s: state, send: () => {} }));
      seen.push({
        TZ,
        day: new Date(AT * 1000).getDate(),
        addresses: state.group.addresses,
        keep,
        opens,
      });
    }
  } finally {
    if (zone === undefined) delete process.env.TZ;
    else process.env.TZ = zone;
    delete globalThis.location;
  }
  assert.deepEqual(
    seen.map((s) => s.day),
    [4, 3],
    'each browser’s own day differs',
  );
  for (const { TZ, addresses, keep, opens } of seen) {
    assert.deepEqual(
      addresses,
      ['app-label', 'other-label', 'app@ada (started the group)', 'app@ada (joined 4 Oct)'],
      TZ,
    );
    assert.deepEqual(
      keep,
      [
        'https://page.example/#open=app@ada+(started+the+group)',
        'https://page.example/#open=app@ada+(joined+4+Oct)',
      ],
      TZ,
    );
    assert.deepEqual(opens, [['hand', 'other-label', '/']], TZ);
  }
});

// join_page::a_phone_opening_sams_device_link_is_added_as_sam_asked_no_name,
// the page's half: a link Sam's computer made for Sam's devices, its
// issuer answering the person it adds (join.rs `Asked::person`), is
// redeemed as Sam's with no name asked, the heading saying whose the phone
// is being added as; joined, the group held is read and the page is in as
// Sam. That the redeem writes one Admit of a browser's key, named Sam by
// Sam's first key: verbs/join.rs
// a_browser_redeeming_a_link_for_sams_devices_stands_as_sam.
test('a link for Sam’s devices adds the phone as Sam’s, asking no name', () => {
  const asked = through(first, { type: 'join', token: 'DEVICE', kept: [], wants: null });
  const group = { id: 'g3', name: 'house', person: 'Sam' };
  const adding = parts.step(asked, { type: 'asked', group, told: 'TOLD' });
  assert.equal(adding.refused, undefined);
  assert.deepEqual(adding.effects, [
    { run: 'admit', token: 'DEVICE', told: 'TOLD', group, name: 'Sam', failed: 'unjoined' },
  ]);
  assert.equal(adding.state.at.join, 'redeeming', 'no name is asked');
  assert.deepEqual(parts.heading(adding.state)?.[0], 'Adding this phone to house as Sam’s…');
  const held = { ...house, id: 'g3', person: 'Sam', token: 'DEVICE' };
  const joined = parts.step(adding.state, { type: 'admitted', group: held });
  assert.deepEqual(
    joined.effects.map((f) => [f.run, f.m?.type, f.m?.group?.person]),
    [['send', 'hold', 'Sam']],
  );
  const read = through(
    joined.state,
    { type: 'hold', group: held, joined: true },
    { type: 'held' },
    { type: 'opened', group: held },
    { type: 'read', n: 1, ok: true, body: { ...body, unanswered: [] }, lent: [] },
  );
  assert.deepEqual(parts.heading(read), ['You’re in house, as Sam', 'good']);
});
