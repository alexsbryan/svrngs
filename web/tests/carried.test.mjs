// A carried app's write on a phone, from the page's POST at its own origin
// to the computer it is carried to: lend.js `lent`, `actsPosted`, `writing`,
// `writeSigned` and what it calls, the entry approved by the group's signer
// (web/sign.js, its frame's listener and `entryApproved`), signed as the
// chain's next line, and carried (`carried`) to whichever computer lending
// the app answers. Both are a browser's scripts, so their functions are cut
// from their text (as lost-answer.test.mjs and proxied.test.mjs cut
// lend.js's), each dial a stand-in answering `/carry` as rail.rs `carry`
// answers it: `{ id, counted }`, or a refusal in door.rs `LendRefusal::said`'s
// shape, its kind in `x-svrngs-refused`. Moved from
// crates/svrngs-node/tests/browser/carried.rs, each test named beside the
// law that holds it now.
//
// Plants, each watched failing in lend.js or sign.js, then put back: named
// in the commit that brought these laws.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { openMessage } from '../authority.js';
import { silent, unreached } from '../../crates/svrngs-node/src/home/question.js?v=__VERSION__';

// A browser script's top-level declarations, cut from its text.
const cutter = (text, file) => ({
  defined: (name) => {
    const at = text.search(new RegExp(`^(async )?function ${name}\\(`, 'm'));
    assert.notEqual(at, -1, `${file} declares ${name}`);
    return text.slice(at, text.indexOf('\n}\n', at) + 2);
  },
  constant: (name) => {
    const found = new RegExp(`^(const|class) ${name}\\b[^]*?(;|\\{\\})\\n`, 'm').exec(text)?.[0];
    assert.ok(found, `${file} declares ${name}`);
    return found;
  },
});
const LEND = cutter(readFileSync(new URL('../lend.js', import.meta.url), 'utf8'), 'lend.js');
const SIGN_TEXT = readFileSync(new URL('../sign.js', import.meta.url), 'utf8');
const SIGN = cutter(SIGN_TEXT, 'sign.js');
// The frame's one listener, which says each refusal to the app's page.
const LISTENER = /^addEventListener\('message', [^]*?\n\}\);\n/m.exec(SIGN_TEXT)?.[0];
assert.ok(LISTENER, 'sign.js listens for the app page’s asks');

const WORKER = [
  // The worker's state as lend.js keeps it: the pass and its line, the
  // writes signed one after another, a page that keeps the signer out.
  'let handed = null;',
  'let signing = Promise.resolve();',
  'let walled = null;',
  'let loaded = true;',
  'async function holding() { return handed; }',
  ...['APPROVING', 'failed', 'posts', 'authority', 'confirmed'].map(LEND.constant),
  ...['lent', 'owned', 'actsPosted', 'writing', 'signed', 'signerAnswer', 'approved'].map(
    LEND.defined,
  ),
  ...['writeSigned', 'setWritten'].map(LEND.defined),
  ...['posted', 'carried', 'judged', 'lenderOf', 'dialAt', 'letGo', 'dialFor'].map(LEND.defined),
  `({
    lent,
    get handed() { return handed; },
    set handed(to) { handed = to; },
  })`,
].join('\n');

const SIGNER = [
  'let loaded = true;',
  ...['decode', 'Told', 'AGAIN', 'tooLarge'].map(SIGN.constant),
  ...['framer', 'answered', 'entryApproved', 'entryAsked'].map(SIGN.defined),
  LISTENER,
].join('\n');

// A carried app's label, 62 letters, as sign.js `framer` reads one.
const LABEL = 'm'.repeat(62);
const GROUP_PAGE = 'https://group.example';
const APP = `https://${LABEL}.group.example`;
const b64 = (json) =>
  Buffer.from(JSON.stringify(json))
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');

// The group's signer, framed under the app, its listener as sign.js runs
// it. Its approver is never reached in these laws: an entry it would
// approve is the browser's (authority.test.mjs, and browser/carried.rs
// `an_app_s_entry_is_approved_by_the_group_s_signer_and_carried`), so a
// pass naming one is refused as an open the signer holds no approver for.
function signer() {
  const parent = {};
  let listener = null;
  runInNewContext(SIGNER, {
    parent,
    location: new URL(`${GROUP_PAGE}/`),
    addEventListener: (type, heard) => type === 'message' && (listener = heard),
    approver: async () => {
      throw new Error('no open adopted for this app');
    },
    console: { debug: () => {} },
    atob,
    URL,
    Error,
    JSON,
    Math,
    Date,
  });
  // A client of the worker's (`scope.clients`), the app's page, which hands
  // the worker's ask to its frame (shim.js) and its port with it.
  return {
    url: `${APP}/`,
    postMessage: (data, [port]) => listener({ source: parent, origin: APP, ports: [port], data }),
  };
}

// The worker at the app's origin holding `handed`, each computer's dial
// answering `/carry` as `carry(lender, [group, line])` says: a Response,
// or thrown. The page (`write`), the lines each dial was asked, the trace.
function worker(handed, carry, page = signer(), door = undefined) {
  const [asked, traced, kept, ports] = [[], [], [], []];
  const dial = (lender) => ({
    fetch: async (method, path, _headers, body) => {
      const sent = JSON.parse(await new Response(body).text());
      asked.push({ lender, method, path, seq: sent[1].seq });
      return carry(lender, sent);
    },
  });
  const scope = {
    clients: { matchAll: async () => [page] },
  };
  const run = runInNewContext(WORKER, {
    scope,
    SCOPE: new URL(`${APP}/`),
    OWN: LABEL,
    SIGNER: GROUP_PAGE,
    dials: new Map(),
    directs: new Map(),
    dataLinks: new Map(),
    dialled: async (lender) => dial(lender),
    trace: (line) => traced.push(line),
    store: {
      lendKeep: async (key, value) => kept.push([key, value]),
      hex: (bytes) => Buffer.from(bytes).toString('hex'),
    },
    // The runtime's two, web/src/lib.rs: an act under the app's kind, and a
    // line's text and the bytes signed, its seq and the id before it.
    app_act: (kind, bytes) => {
      const { op } = JSON.parse(new TextDecoder().decode(bytes));
      return JSON.stringify({ kind: `${kind}/${op.kind}`, description: op.description });
    },
    unsigned_line: (holder, group, ts, seq, prev, act, by) => [
      JSON.stringify({ id: `line-${seq}-${ts}-${act.length}`, seq, prev, holder, group, by }),
      new TextEncoder().encode(`${seq}`),
    ],
    crypto: { subtle: { sign: async () => new Uint8Array([7]).buffer } },
    links: () => true,
    openMessage,
    silent,
    unreached,
    closed: () => assert.fail('a write of a held pass is not closed'),
    unheldAt: () => assert.fail('the pass is held'),
    opening: () => assert.fail('no navigation here'),
    proxied: door ?? (() => assert.fail('a write is not proxied')),
    jarOf: async () => (door ? [] : assert.fail('a write keeps no cookies')),
    MessageChannel: class extends MessageChannel {
      constructor() {
        super();
        ports.push(this.port1, this.port2);
      }
    },
    Blob,
    URL,
    Request,
    Response,
    TextEncoder,
    TextDecoder,
    Uint8Array,
    Promise,
    Error,
    JSON,
    Math,
    Date,
    setTimeout,
    clearTimeout,
  });
  run.handed = handed;
  // The page's POST at `svrngs/acts`, as Money's script sends it, and what
  // the page is answered: its status, its refusal's kind, its body.
  const write = async (description) => {
    const request = new Request(`${APP}/svrngs/acts`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ op: { kind: 'expense', amount_cents: 1200, description } }),
    });
    // Asked by a page of this origin, which may sign: authority.js
    // `maySign` (authority.test.mjs), as lend.js `answered` asks it.
    const where = { label: LABEL, path: '/svrngs/acts' };
    const res = await run.lent(request, new URL(request.url), where, true);
    for (const port of ports.splice(0)) port.close();
    return {
      status: res.status,
      refused: res.headers.get('x-svrngs-refused'),
      said: await res.json(),
    };
  };
  return { write, asked, traced, run, ports };
}

// The pass as hand.js signs it (web/src/lib.rs `pass`, svrngs-core
// pass.rs `Pass`, whose `approver` is left out when none is named), and
// the open it was handed with, Money carried in house by ada, and by cy.
function opened({ approver = 'a'.repeat(64), lenders = [{ key: 'cy' }] } = {}) {
  const pass = b64({
    app: LABEL,
    reads: [],
    holder: 'h',
    until: 2e9,
    sig: 's',
    ...(approver && { approver }),
  });
  const lend = {
    label: LABEL,
    key: 'ada',
    lenders,
    group: 'house',
    kind: 'Money',
    person: 'ada',
    name: 'Money',
    version: 9,
  };
  return {
    label: LABEL,
    lends: [lend],
    pass,
    writes: null,
    group: 'house',
    holder: 'h',
    proof: 'p',
    key: {},
    unsent: [],
  };
}
const taken = (_lender, [, line]) => Response.json({ id: line.id, counted: true });
const limit = () =>
  Response.json(
    { error: 'Money is writing faster than an app may here.', kind: 'limit' },
    { status: 429, headers: { 'x-svrngs-refused': 'limit', 'retry-after': '18' } },
  );
const approving = () => {
  // A signer that approves each entry it is asked, as sign.js does for an
  // open it adopted: its pass in the reply.
  const page = { url: `${APP}/` };
  page.postMessage = (_data, [port]) => port.postMessage({ pass: 'approved' });
  return page;
};

// browser::carried::a_phone_s_line_past_the_budget_is_let_go_and_its_chain_rewound:
// bo's two writes past Money's budget are each refused `limit`, 429, as
// ada's computer answers (carried.rs `budget_refused`, held by
// tests/carried.rs a_phone_s_line_past_the_app_s_budget_is_refused), and
// the second is signed where the first left off, line 0 with one to carry:
// a refused line neither waits for a next write nor leaves a hole.
test('a line past its app’s budget is let go, and the next is signed in its place', async () => {
  const phone = worker(opened({ lenders: [] }), limit, approving());
  for (const words of ['the bins', 'the milk']) {
    const wrote = await phone.write(words);
    assert.deepEqual([wrote.status, wrote.refused], [429, 'limit'], words);
  }
  const signed = phone.traced.filter((l) => /signed as line/.test(l));
  assert.deepEqual(signed, [
    'RUNTIME: an act of Money signed as line 0, 1 to carry',
    'RUNTIME: an act of Money signed as line 0, 1 to carry',
  ]);
  assert.deepEqual(
    phone.asked.map(({ lender, path, seq }) => [lender, path, seq]),
    [
      ['ada', '/carry', 0],
      ['ada', '/carry', 0],
    ],
  );
  // The worker's own realm's values, read into this one.
  assert.deepEqual([phone.run.handed.line, phone.run.handed.unsent.length], [undefined, 0]);
});

// browser::carried::the_same_page_writes_through_the_next_computer_once_the_first_stops:
// one page, one worker, never loaded again: its first write carried to
// ada's computer and taken; ada's stops, and its second is carried to cy's,
// the next computer lending Money, and taken there, both 200. That a line
// is taken at any computer of the group and counts as bo's is
// tests/carried.rs a_phone_s_line_is_carried_and_counts_as_its_person; that
// Money is one origin on both, a_member_writes_on_their_own_computer_s_copy_alone.
test('the same page writes through the next computer once the first stops', async () => {
  let stopped = false;
  const carry = (lender, line) => {
    if (lender === 'ada' && stopped) throw new Error('connection lost');
    return taken(lender, line);
  };
  const phone = worker(opened(), carry, approving());
  const first = await phone.write('the bins');
  stopped = true;
  const second = await phone.write('the milk');
  assert.deepEqual([first.status, second.status], [200, 200]);
  assert.deepEqual([first.said.counted, second.said.counted], [true, true]);
  assert.deepEqual(
    phone.asked.map(({ lender, seq }) => [lender, seq]),
    [
      ['ada', 0],
      ['ada', 1],
      ['cy', 1],
    ],
    'each line taken once, the second at cy’s once ada’s did not answer',
  );
  assert.equal(phone.run.handed.line.seq, 1, 'the chain goes on from the one page');
  assert.equal(phone.run.handed.unsent.length, 0);
});

// browser::carried::a_carried_app_that_read_friends_lends_writes_on_a_phone:
// Reader, once let read a friend's lend, is opened with passes naming no
// approver (web/page/hand.js `writesBlocked`, through its Stop:
// hand.test.mjs 'a carried app once let read lends has no entry approved,
// through its Stop'). Its save, under the pass it holds and under the next,
// is told why by the group's signer, and no line is signed or carried, so
// nothing of bo's counts on ada's computer. A pass naming an approver is
// not told so: the refusal is the missing approver's.
test('a carried app that read friends’ lends is told it can’t write, and carries nothing', async () => {
  const told = 'An app that reads your friends’ apps can’t write in the group’s record.';
  const reader = worker(opened({ approver: null }), taken);
  for (const words of ['after a read', 'after the Stop, its pass held']) {
    const saved = await reader.write(words);
    assert.deepEqual([saved.status, saved.said], [403, { error: told }], words);
  }
  const next = worker(opened({ approver: null }), taken);
  const stopped = await next.write('after the Stop, a new pass');
  assert.deepEqual([stopped.status, stopped.said], [403, { error: told }]);
  for (const phone of [reader, next]) {
    assert.deepEqual(phone.asked, [], 'nothing carried to a computer');
    assert.deepEqual(
      phone.traced.filter((l) => /signed as line/.test(l)),
      [],
      'no line signed',
    );
  }
  const approvable = worker(opened(), taken);
  const refused = await approvable.write('with an approver');
  assert.equal(refused.status, 403);
  assert.equal(refused.said.error, 'Open this app again from your group’s page to write in it.');
});

// A carried app's write to its database on a phone (svrn-docs
// APP_STATE §4, "Phones"): lend.js `owned` and `setWritten`. The page's
// `{write}` is prepared at the computer that answers (`{prepare}`), handed
// to the group's signer with the open, and each set the signer keeps is
// carried (`{carry}`), the same bytes until a computer keeps one; the
// places kept are told the signer with the next write, so it lets them go.
// The signer here answers as sign.js `setsSigned` does, keeping each set
// it signs until it is told it was kept. A carry the computer couldn't keep
// is said to the page, and the set goes again, first, with the next write;
// a signer's refusal is said to the page, and nothing is carried.
// Plant, watched failing: the worker telling the signer no place kept (the
// third write's ask names none).
test('a phone’s write to its database is prepared, signed by the group’s signer, and carried until a computer keeps it', async () => {
  const [asks, doors, held] = [[], [], []];
  let [keeps, refusing] = [503, false];
  const door = async ({ request }) => {
    const body = await request.json();
    doors.push(body.carry ?? Object.keys(body)[0]);
    if (body.prepare)
      return Response.json({ prepared: JSON.stringify({ count: 1 }), by: 'ada', sig: 's' });
    return Response.json({ took: keeps === 200 ? 'taken' : 'fault' }, { status: keeps });
  };
  const page = {
    url: `${APP}/`,
    postMessage: ({ approve }, [port]) => {
      // As the page's postMessage copies what it hands on.
      asks.push(structuredClone(approve.set));
      if (refusing) return port.postMessage({ error: 'This phone didn’t sign it.' });
      held.splice(0, held.length, ...held.filter(([at]) => !approve.set.confirmed.includes(at)));
      held.push([`ada/0/${asks.length}`, `set ${asks.length}`]);
      port.postMessage({ sets: [...held] });
    },
  };
  const phone = worker(opened(), taken, page, door);
  const write = async () => {
    const request = new Request(`${APP}/svrngs/sql`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ write: [{ insert: 'magnets', row: { id: 'moon', position: 2 } }] }),
    });
    const where = { label: LABEL, path: '/svrngs/sql' };
    const res = await phone.run.lent(request, new URL(request.url), where, true);
    for (const port of phone.ports.splice(0)) port.close();
    return [res.status, await res.json()];
  };
  assert.deepEqual(await write(), [503, { took: 'fault' }], 'not kept: said to the page');
  keeps = 200;
  assert.deepEqual(await write(), [200, { count: 1 }]);
  assert.deepEqual(await write(), [200, { count: 1 }]);
  assert.deepEqual(doors, ['prepare', 'set 1', 'prepare', 'set 1', 'set 2', 'prepare', 'set 3']);
  assert.deepEqual(
    asks.map((ask) => [ask.prepared, ask.by, ask.ops[0].row.id, ask.confirmed.length]),
    [
      ['{"count":1}', 'ada', 'moon', 0],
      ['{"count":1}', 'ada', 'moon', 0],
      ['{"count":1}', 'ada', 'moon', 2],
    ],
  );
  assert.equal(asks[0].authority.proof, 'p', 'the open, for the signer to check');
  refusing = true;
  const refused = await write();
  assert.deepEqual(refused, [403, { refused: 'unsigned', words: 'This phone didn’t sign it.' }]);
  assert.equal(doors.length, 8, 'prepared, and nothing carried');
});

// The group's signer picks each set's place (sign.js `setsSigned`; svrn-docs
// app-state-diligence/invariants-2026-11-22/, change 5), in one record with
// its store's id: at the computer that prepared it, the seq after the last
// it signed there. The app's page relays every message, so it can send a
// prepare again, or two at once: a prepare kept already is no second set;
// two at once sign two places, the later signed again past the first; each
// computer counts apart; a place the page names confirmed is let go; a
// record cleared, as Safari may clear it, signs at a store of its own; and
// a prepare that changes nothing is no set, and keeps nothing. Plant,
// watched failing: the record put whether or not it moved since it was read
// (the two at once both signed at ada/3).
test('the group’s signer picks each place, so it never signs one twice', async () => {
  const records = new Map();
  // As IndexedDB runs a transaction, a turn later and whole, so two at once
  // both read before either writes.
  const stored = async (_name, _key, _mode, act) => {
    await Promise.resolve();
    const objects = {
      get: (id) => ({ result: structuredClone(records.get(id)) }),
      put: (record) => records.set(record.id, structuredClone(record)),
    };
    const read = act(objects);
    read.onsuccess?.();
    return read.result;
  };
  let drawn = 0;
  const { setsSigned } = runInNewContext(
    [
      SIGN.constant('Told'),
      SIGN.constant('signedSets'),
      SIGN.defined('lendOf'),
      SIGN.defined('setsSigned'),
      '({ setsSigned })',
    ].join('\n'),
    {
      approver: async () => ({ key: 'approver' }),
      // set.rs `signed`, its set its place, its store's first byte and its
      // stamp; none for a prepare that changes nothing.
      set_checked: (_label, text) => {
        const { prepared, ts, by, store, seq } = JSON.parse(text);
        if (JSON.parse(prepared).none) return [];
        return ['message', `${by}/${seq} of ${store[0]} at ${ts}`, `${by}/${seq}`];
      },
      set_sealed: (made, sig) => `${made} by ${sig}`,
      crypto: {
        subtle: { sign: async () => new Uint8Array([7]) },
        getRandomValues: (bytes) => bytes.fill(++drawn),
      },
      store: { stored, hex: (bytes) => Buffer.from(bytes).toString('hex') },
      Error,
      JSON,
      Object,
    },
  );
  const lends = [
    { label: LABEL, key: 'ada', group: 'g', kind: 'board', lenders: [{ key: 'ben' }] },
  ];
  // Its answer's places, read back in this realm, as the port copies it.
  const sign = async (by, sig, confirmed = [], prepared = '{}') => {
    const asked = { ops: [], prepared, by, sig, confirmed, authority: { lends, pass: 'p' } };
    return structuredClone(await setsSigned(asked, LABEL, 1000)).sets.map(([at, set]) => [at, set]);
  };
  assert.deepEqual(await sign('ada', 's1'), [['ada/1', 'ada/1 of 1 at 1000 by 07']]);
  assert.deepEqual(
    await sign('ada', 's1'),
    [['ada/1', 'ada/1 of 1 at 1000 by 07']],
    'no second set',
  );
  const [three, four] = await Promise.all([sign('ada', 's3'), sign('ada', 's4')]);
  assert.deepEqual(
    [three.at(-1)[0], four.at(-1)[0]].sort(),
    ['ada/2', 'ada/3'],
    'two at once, two places',
  );
  assert.deepEqual(
    (await sign('ben', 's5', ['ada/1', 'ada/2', 'ada/3'])).map(([at]) => at),
    ['ben/1'],
  );
  records.clear();
  assert.deepEqual(
    await sign('ada', 's6'),
    [['ada/1', 'ada/1 of 2 at 1000 by 07']],
    'a store of its own',
  );
  const kept = structuredClone(records.get('g/board'));
  assert.deepEqual(await sign('ada', 's7', [], '{"none":true}'), [], 'no set');
  assert.deepEqual(records.get('g/board'), kept, 'and nothing kept for it');
});
