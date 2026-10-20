// House rules' laws (row canon-house; svrn-docs rings/current/CANON_IN_SVRNGS.md
// §3, §8): what the glue (house.js) hands canon, and what it hands back, held
// against canon itself — the vendored build, run under node as the page runs
// it. Each law is red on a plant its commit names.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { bind } from './vendor/call.mjs';
import * as canonJs from './vendor/canon.js';
import { CHOICES, OFFERS, ROOT, agree, decide, fold, human, membership, object, payload, propose, recorded } from './house.js';

const call = bind(canonJs);
const G = 'g1';
const DAY = 86_400;
const span = (person, joined = null, left = null) => ({ person, joined, left });
const founders = (...people) => people.map((p) => span(p));
let n = 0;
/** A record of House rules, `act` written by `person`'s computer for `as`. */
const rec = (ts, person, act, as = person) => ({ id: `r${++n}`, ts_unix: ts, person, on_behalf_of: as, voided: false, payload: payload(act) });
/** The same, without the page's check of what it writes: what a changed page could record. */
const raw = (ts, person, act, as = person) => ({ id: `r${++n}`, ts_unix: ts, person, on_behalf_of: as, voided: false, payload: { kind: 'act', app: 'canon', act } });
const read = (spans, ops) => ({ you: spans[0].person, group: G, people: spans.filter((s) => s.left == null).map((s) => s.person), spans, ops });
/** canon's id for `act` recorded at `ts` for `by`. */
const idOf = (ts, by, act) => JSON.parse(call({ call: 'act', kind: act, ts_unix: ts, actor: human(by) })).id;
const QUIET = propose('Quiet hours are 10pm to 7am');
const status = (view, id) => view.rules.find((r) => r.id === id)?.status.status;

test('canon’s actor is the record’s person, never the act’s text; another computer’s writing is said as via', () => {
  const spans = founders('ana', 'bo', 'kim');
  const q = idOf(2000, 'ana', QUIET);
  const ops = [
    rec(1000, 'ana', decide('everyone')),
    rec(2000, 'ana', QUIET),
    rec(2100, 'kim', agree(q)),
    raw(2200, 'kim', { ...agree(q), actor: human('bo') }),
  ];
  const view = fold(call, read(spans, ops), 3000);
  assert.equal(status(view, q), 'proposed', 'kim’s act naming bo is not bo’s agreement');
  assert.deepEqual(view.dropped.map((d) => [d.by, d.why]), [['kim', 'it speaks as human:bo, not as kim, who wrote it']]);
  const through = fold(call, read(spans, [...ops, rec(2300, 'kim', agree(q), 'bo')]), 3000);
  assert.equal(status(through, q), 'active', 'bo’s agreement, written through kim’s computer, is bo’s');
  assert.deepEqual(through.rules[0].said.map((s) => [s.by, s.via]), [['kim', null], ['bo', 'kim']]);
});

test('a rule’s standing is the build’s, never counted in the page', () => {
  const spans = founders('ana', 'bo', 'kim');
  const q = idOf(2000, 'ana', QUIET);
  const ops = [
    rec(1000, 'ana', decide('everyone')),
    rec(2000, 'ana', QUIET),
    rec(2010, 'kim', object(q, 'It is too early for the night shift.')),
    rec(2020, 'bo', agree(q)),
    rec(2030, 'kim', agree(q)),
  ];
  const view = fold(call, read(spans, ops), 3000);
  // Three of three agree by the end, and canon refuses it: kim objected first.
  assert.deepEqual(view.rules.map((r) => r.status), [{ status: 'refused', at: 2010, by: 'human:kim', why: 'It is too early for the night shift.' }]);
});

test('a person outside the house has no say', () => {
  const spans = founders('ana', 'bo');
  const q = idOf(2000, 'ana', QUIET);
  const ops = [rec(1000, 'ana', decide('consent')), rec(2000, 'ana', QUIET), rec(2100, 'dee', object(q, 'I never agreed to this.'))];
  const view = fold(call, read(spans, ops), 2000 + 3 * DAY);
  assert.equal(status(view, q), 'active', 'dee’s objection is said and does not count');
  assert.equal(view.rules[0].said.length, 1);
});

test('the first question’s answer is the whole house’s rule', () => {
  assert.deepEqual(Object.keys(CHOICES), ['consent', 'most', 'everyone']);
  assert.deepEqual(decide('consent').rule, { rule: 'consent', days: 2 });
  assert.deepEqual(decide('most').rule, { rule: 'majority' });
  assert.deepEqual(decide('everyone').rule, { rule: 'unanimous' });
  const spans = founders('ana', 'bo', 'kim', 'dee');
  const before = fold(call, read(spans, []), 3000);
  assert.equal(before.rule, null, 'no rule nobody chose');
  const view = fold(call, read(spans, [rec(1000, 'ana', decide('most'))]), 3000);
  assert.deepEqual(view.rule, { rule: 'majority' });
});

test('an act that speaks as the house, or as anyone but its writer, is refused', () => {
  const spans = founders('ana', 'bo');
  const q = idOf(2000, 'ana', QUIET);
  const ops = [
    rec(1000, 'ana', decide('consent')),
    rec(2000, 'ana', QUIET),
    raw(2000 + 3 * DAY, 'dee', { op: 'retract', target: q, actor: `house:${G}` }),
  ];
  const view = fold(call, read(spans, ops), 2000 + 4 * DAY);
  assert.equal(status(view, q), 'active');
  assert.deepEqual(view.dropped.map((d) => d.why), [`it speaks as house:${G}, not as dee, who wrote it`]);
});

test('under “unless someone objects”, a person’s objection counts', () => {
  const spans = founders('ana', 'bo');
  const q = idOf(2000, 'ana', QUIET);
  const ops = [rec(1000, 'ana', decide('consent')), rec(2000, 'ana', QUIET), rec(2100, 'bo', object(q, 'Not on Fridays.'))];
  const view = fold(call, read(spans, ops), 2000 + 3 * DAY);
  assert.deepEqual(view.rules[0].status, { status: 'refused', at: 2100, by: 'human:bo', why: 'Not on Fridays.' });
});

test('a past rule keeps its agreement when someone joins later', () => {
  const q = idOf(2000, 'ana', QUIET);
  const ops = [rec(1000, 'ana', decide('most')), rec(2000, 'ana', QUIET), rec(2100, 'bo', agree(q))];
  const then = fold(call, read(founders('ana', 'bo', 'kim'), ops), 3000);
  assert.equal(status(then, q), 'active');
  const later = fold(call, read([...founders('ana', 'bo', 'kim'), span('cy', 5000)], ops), 6000);
  assert.equal(status(later, q), 'active', 'cy joining does not undo it');
  assert.deepEqual(later.deciding, ['ana', 'bo', 'kim', 'cy']);
});

test('a name with a dot is a person, never a corner of someone else’s', () => {
  const spans = founders('ana', 'jo.b');
  const q = idOf(2000, 'ana', QUIET);
  const ops = [rec(1000, 'ana', decide('everyone')), rec(2000, 'ana', QUIET), rec(2100, 'jo.b', agree(q))];
  const view = fold(call, read(spans, ops), 3000);
  assert.equal(status(view, q), 'active', 'jo.b’s agreement counts');
  assert.deepEqual(view.deciding, ['ana', 'jo.b']);
  const scopes = membership(call, G, spans).map((l) => JSON.parse(l).scope);
  assert.ok(scopes.every((s) => s === ROOT), `the house writes one scope, never a person’s own: ${scopes}`);
});

test('the house’s acts are the same, ids included, however its people are listed', () => {
  const spans = [span('ana'), span('bo', 1500), span('cy', 1600, 1700), span('cy', 1800)];
  const once = membership(call, G, spans);
  assert.deepEqual(membership(call, G, [...spans].reverse()), once);
  assert.deepEqual(once.map((l) => JSON.parse(l)).map((a) => [a.ts_unix, a.op, a.holder]), [
    [0, 'grant', `house:${G}`],
    [1, 'grant', 'human:ana'],
    [1500, 'grant', 'human:bo'],
    [1600, 'grant', 'human:cy'],
    [1700, 'withdraw', 'human:cy'],
    [1800, 'grant', 'human:cy'],
  ].sort((a, b) => a[0] - b[0]));
});

test('the house’s seat is the only one taken open', () => {
  const lines = membership(call, G, founders('ana', 'bo'));
  const acts = lines.map((l) => JSON.parse(l));
  const canon = call({ call: 'derive', acts: [...lines, call({ call: 'act', kind: decide('consent'), ts_unix: 1000, actor: 'human:ana' }).trim()].join('\n'), now: 3000 });
  assert.deepEqual(canon.bootstrap.map(([id]) => id), [acts[0].id], 'the house’s own grant, and nothing else');
  assert.ok(acts.slice(1).every((a) => a.ts_unix > acts[0].ts_unix), 'every person’s grant after the house’s');
  assert.equal(recorded(read(founders('ana'), [rec(1, 'ana', QUIET)])).dropped[0].why, 'it is dated before your house began');
});

// A key canon's own CLI bound to human:bo (`canon identify`, canon 0.3.0),
// signed by a key made for this test alone.
const BO_KEY = { id: 'can-90fed5154eb2', v: 3, ts_unix: 1792264278, actor: 'human:bo', op: 'key', keys: ['ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAhBEC1SeADABEZM6+h800BS4f3kJ9S20Dc+V7nkEB7x'], sig: 'U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAgCEEQLVJ4AMAERkzr6HzTQFLh/eQn1LbQNz5XueQQHvEAAAAFY2Fub24AAAAAAAAABnNoYTUxMgAAAFMAAAALc3NoLWVkMjU1MTkAAABAgk1OJMq5U0ADTsuBrFWOpV8SArnpkM0/7hhLGaa5CBjhPT7K8NlB7zADnd1E3vCx44wFO72xxRGruKvZAMrNCQ==' };

test('no key is bound: the page writes none, and one in the record is not counted', () => {
  assert.throws(() => payload({ op: 'key', keys: BO_KEY.keys }), /never writes key/);
  const spans = founders('ana', 'bo');
  const q = idOf(2000, 'ana', QUIET);
  const ops = [rec(1000, 'ana', decide('everyone')), rec(2000, 'ana', QUIET), raw(BO_KEY.ts_unix, 'bo', BO_KEY), rec(2100, 'bo', agree(q))];
  const view = fold(call, read(spans, ops), BO_KEY.ts_unix + 1);
  assert.equal(status(view, q), 'active', 'bo’s agreement still counts');
  assert.deepEqual(view.dropped.map((d) => d.why), ['it binds a key to a name, which House rules never does']);
});

test('the house is never said as a person', () => {
  const spans = founders('ana', 'bo');
  const view = fold(call, read(spans, [rec(1000, 'ana', decide('consent')), rec(2000, 'ana', QUIET)]), 3000);
  assert.deepEqual(view.deciding, ['ana', 'bo']);
  for (const list of [view.unattended, view.bootstrap, view.ungoverned]) assert.deepEqual(list, []);
});

test('no draw, rotation or subsidiarity is offered: canon 0.5.0 counts the house in their pools', () => {
  for (const op of ['draw_commit', 'draw_secret', 'draw_reveal', 'allot', 'allocation', 'policy']) {
    assert.ok(!OFFERS.includes(op), `${op} is offered`);
    assert.throws(() => payload({ op }), /never writes/);
  }
});

test('no say over the house is granted or taken but by joining and leaving', () => {
  for (const op of ['grant', 'withdraw']) assert.ok(!OFFERS.includes(op), `${op} is offered`);
  const spans = founders('ana', 'bo');
  const q = idOf(3000, 'ana', QUIET);
  const bos = membership(call, G, spans).map((l) => JSON.parse(l)).find((a) => a.holder === human('bo')).id;
  const ops = [
    rec(1000, 'ana', decide('consent')),
    raw(1500, 'ana', { op: 'grant', holder: human('dee'), scope: ROOT }),
    raw(1600, 'ana', { op: 'revert', targets: [bos] }),
    raw(1700, 'ana', { op: 'withdraw', holder: human('bo'), scope: ROOT }),
    rec(3000, 'ana', QUIET),
    rec(3100, 'dee', object(q, 'Too strict.')),
  ];
  const view = fold(call, read(spans, ops), 3000 + 3 * DAY);
  assert.equal(status(view, q), 'active', 'dee was given no say');
  assert.deepEqual(view.deciding, ['ana', 'bo'], 'bo was stood down by nobody');
  assert.equal(view.dropped.length, 3);
});

for (const [chose, to] of [['everyone', 'most'], ['most', 'everyone']]) {
  test(`a house that shrinks can still change how it decides (${CHOICES[chose].words})`, () => {
    // Four answer; kim leaves; the three left propose another way and all
    // three agree. A change is judged by the rule in force (canon
    // ratify.rs:19-29), so a count of four would hold it for ever.
    const spans = [...founders('ana', 'bo', 'dee'), span('kim', null, 5000)];
    const change = decide(to);
    const c = idOf(6000, 'ana', change);
    const ops = [rec(1000, 'ana', decide(chose)), rec(6000, 'ana', change), rec(6100, 'bo', agree(c)), rec(6200, 'dee', agree(c))];
    const view = fold(call, read(spans, ops), 7000);
    assert.equal(view.changes.find((x) => x.id === c)?.verdict.verdict, 'ratified', 'the three change their rule');
    assert.deepEqual(view.rule, change.rule);
  });
}

test('someone who joins after a rule is proposed is not counted toward it', () => {
  // Five answer "most of you agree"; eve and fay leave; ana proposes, and the
  // three in the house then decide it. cy joins after and objects: more than
  // half of the three agreed, so it holds.
  const spans = [...founders('ana', 'bo', 'dee'), span('eve', null, 1500), span('fay', null, 1600), span('cy', 2500)];
  const q = idOf(2000, 'ana', QUIET);
  const ops = [rec(1000, 'ana', decide('most')), rec(2000, 'ana', QUIET), rec(2550, 'cy', object(q, 'I was not asked.')), rec(2600, 'bo', agree(q))];
  const view = fold(call, read(spans, ops), 3000);
  assert.equal(status(view, q), 'active', 'ana and bo are more than half of the three');
  assert.deepEqual(view.deciding, ['ana', 'bo', 'dee', 'cy']);
});

test('one answer has one clock: an agreement timed ahead of the browser’s clock is in force when the page asks', () => {
  // svrngs times a new act after every act it holds (crates/svrngs-core/src/
  // command.rs:243-250), so bo's agreement, written in the second of ana's
  // change, is timed a second ahead of the browser's clock; the page asks
  // right after, at that clock (index.js, `Date.now()`), and says the rule
  // the agreement passed (#206; demo/runs/canon/2026-10-17-house/same_second.mjs).
  const spans = founders('ana', 'bo', 'cy');
  const change = decide('everyone');
  const c = idOf(2000, 'ana', change);
  const ops = [rec(1000, 'ana', decide('most')), rec(2000, 'ana', change), rec(2100, 'bo', agree(c))];
  const view = fold(call, read(spans, ops), 2099);
  assert.equal(view.changes.find((x) => x.id === c)?.verdict.verdict, 'ratified');
  assert.deepEqual(view.rule, change.rule, 'the rule bo’s agreement passed, not the one it replaced');
});

test('an answer the page could not read is said as unread, never as a house whose people are unsettled', () => {
  // svrngs says a house whose people are unsettled as `spans: null`
  // (crates/svrngs-node/src/carried.rs `acts`); an answer with no `spans` at
  // all, as index.js made of a body that was not JSON (`{}`), is one the page
  // could not read, and says so (#209).
  assert.deepEqual(fold(call, { ...read(founders('ana'), []), spans: null }, 3000), { unsettled: true });
  assert.throws(() => fold(call, {}, 3000), /^Error: this computer’s answer doesn’t say who is in your house$/);
});
