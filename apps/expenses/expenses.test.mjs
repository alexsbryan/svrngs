// rewritten in E0 from commonwealth-ai sovereign/apps/ring-doc/expenses.test.mjs@96261aa71;
// not a fork, so scripts/ports.py does not pair it.
// The money rules, pinned. Each name says which mistake loses money. What is
// not tested here is order: every copy folding the same acts in the same order
// is the record's property (svrngs-core `admit`), not the reducer's. The laws
// every reducer holds are `laws.test.mjs`.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  EXPENSE,
  actsOf,
  SETTLE,
  SHARE,
  init,
  money,
  parseCents,
  reduce,
  splitShares,
  transfers,
  validate,
} from './expenses.js';

const [SAM, AMA, BO, CY] = ['discord:1', 'discord:2', 'discord:3', 'discord:4'];
const DAY = 86400;

let n = 0;
const act = (by, op, ts = 1000) => ({ id: `act-${++n}`, ts, by, op });
const paid = (by, cents, what = 'pizza', ts) => act(by, { kind: EXPENSE, amount_cents: cents, description: what }, ts);
const share = (by, expense, ts) => act(by, { kind: SHARE, expense }, ts);
const settle = (by, to, cents, ts) => act(by, { kind: SETTLE, to, amount_cents: cents }, ts);
const fold = (acts) => acts.reduce(reduce, init());
const sum = (state) => Object.values(state.balances).reduce((a, b) => a + b, 0);

test('ten dollars three ways keeps the penny and gives it to the first name', () => {
  assert.deepEqual(splitShares(1000, ['a', 'b', 'c']), [
    ['a', 334],
    ['b', 333],
    ['c', 333],
  ]);
});

test('shares always sum to the amount, whatever order the people came in', () => {
  const people = ['g', 'a', 'f', 'b', 'e', 'c', 'd'];
  for (let k = 1; k <= people.length; k++) {
    for (const amount of [1, 2, 3, 99, 100, 1000, 1001, 123457]) {
      const shares = splitShares(amount, people.slice(0, k));
      assert.equal(
        shares.reduce((a, [, c]) => a + c, 0),
        amount,
      );
      assert.deepEqual(shares, splitShares(amount, people.slice(0, k).reverse()));
    }
  }
});

test('the pizza: whoever confirms is in, and every fold sums to zero', () => {
  const pizza = paid(SAM, 6000);
  const acts = [pizza, share(AMA, pizza.id), share(BO, pizza.id), share(CY, pizza.id)];
  const state = fold(acts);
  assert.deepEqual(state.balances, { [SAM]: 4500, [AMA]: -1500, [BO]: -1500, [CY]: -1500 });
  for (let k = 0; k <= acts.length; k++) assert.equal(sum(fold(acts.slice(0, k))), 0);
});

test('confirming twice is confirming once', () => {
  const pizza = paid(SAM, 6000);
  const once = fold([pizza, share(AMA, pizza.id)]);
  const twice = fold([pizza, share(AMA, pizza.id), share(AMA, pizza.id)]);
  assert.deepEqual(twice.balances, once.balances);
  assert.equal(twice.counted, once.counted);
});

test('the payer is in on their own expense and cannot be added twice', () => {
  const pizza = paid(SAM, 6000);
  assert.deepEqual(fold([pizza, share(SAM, pizza.id)]).balances, { [SAM]: 0 });
});

test('a settlement recorded twice on one day is counted once, and on another day twice', () => {
  const pizza = paid(SAM, 6000);
  const base = [pizza, share(AMA, pizza.id)];
  const once = fold([...base, settle(AMA, SAM, 3000, 5000)]);
  assert.deepEqual(once.balances, { [SAM]: 0, [AMA]: 0 });
  assert.deepEqual(fold([...base, settle(AMA, SAM, 3000, 5000), settle(AMA, SAM, 3000, 6000)]).balances, once.balances);
  const later = fold([...base, settle(AMA, SAM, 1000, 5000), settle(AMA, SAM, 1000, 5000 + DAY)]);
  assert.deepEqual(later.balances, { [SAM]: 1000, [AMA]: -1000 });
});

test('a share of an expense that is not in the book is refused, not guessed', () => {
  const state = fold([share(AMA, 'act-nowhere')]);
  assert.deepEqual(state.balances, {});
  assert.equal(state.gaps[0].code, 'no_expense');
});

test('a non-positive or fractional amount is refused whole', () => {
  for (const cents of [0, -100, 12.5, '600', null, 1e12]) {
    const state = fold([paid(SAM, cents)]);
    assert.deepEqual(state.balances, {}, `${cents}`);
    assert.equal(state.gaps[0].code, 'bad_amount');
  }
});

test('paying yourself back is refused', () => {
  assert.equal(validate(init(), settle(SAM, SAM, 100))[0].code, 'bad_settle');
});

test('an act from a newer version is reported, not misread', () => {
  const state = fold([act(SAM, { kind: 'loan', amount_cents: 500 })]);
  assert.deepEqual(state.balances, {});
  assert.equal(state.gaps[0].code, 'unknown_kind');
});

test('an act no one in the chat wrote moves no money', () => {
  const state = fold([paid(null, 6000)]);
  assert.deepEqual(state.balances, {});
  assert.equal(state.gaps[0].code, 'no_one');
});

test('a person named __proto__ cannot make money vanish', () => {
  const pizza = paid(SAM, 6000);
  const state = fold([pizza, share('__proto__', pizza.id), settle(SAM, '__proto__', 100)]);
  assert.deepEqual(state.balances, { [SAM]: 0 });
});

test('amounts are read the way people type them', () => {
  assert.equal(parseCents('60'), 6000);
  assert.equal(parseCents('$12.50'), 1250);
  assert.equal(parseCents('7,5'), 750);
  assert.equal(parseCents(' 0.01 '), 1);
  for (const bad of ['', '0', 'sixty', '1.234', '-5', '1e3', undefined]) assert.equal(parseCents(bad), null, bad);
});

test('money renders the way a person reads it', () => {
  assert.equal(money(0), '$0.00');
  assert.equal(money(5), '$0.05');
  assert.equal(money(-1234), '-$12.34');
});

test('who owes whom settles every balance in a few payments', () => {
  const balances = { a: 4500, b: -1500, c: -1500, d: -1500 };
  const out = transfers(balances);
  assert.equal(out.length, 3);
  for (const t of out) {
    balances[t.from] += t.cents;
    balances[t.to] -= t.cents;
  }
  assert.deepEqual(Object.values(balances), [0, 0, 0, 0]);
});

test('the record\'s log: `by` is whom an app wrote for, else the computer\'s person; voided and empty ops are not acts', () => {
  const op = (id, extra) => ({ id, actor: 'k1', person: 'sam', seq: 0, ts_unix: 7, voided: false, ...extra });
  const pizza = { kind: EXPENSE, amount_cents: 6000, description: 'pizza' };
  const log = {
    ops: [
      op('o1', {}), // the group's own act: no payload
      op('o2', { payload: pizza }),
      op('o3', { payload: { kind: SHARE, expense: 'o2' }, on_behalf_of: 'bo' }),
      op('o4', { payload: pizza, voided: true }),
      op('o5', { corrects: 'o4' }), // voids, states nothing
    ],
  };
  assert.deepEqual(actsOf(log), [
    { id: 'o2', ts: 7, by: 'sam', via: null, op: pizza },
    { id: 'o3', ts: 7, by: 'bo', via: 'sam', op: { kind: SHARE, expense: 'o2' } },
  ]);
  assert.throws(() => actsOf({ error: 'not held' }), /no list of acts/);
});

test('which computer wrote it is kept: `via` is its person when it wrote for another, else null; the sums unchanged', () => {
  // ralph/FINDING_ON_BEHALF_OF.md's scene: Cy's computer writes "Bo paid $400" in Bo's name.
  const op = (id, extra) => ({ id, actor: 'k1', seq: 0, ts_unix: 7, voided: false, ...extra });
  const rent = { kind: EXPENSE, amount_cents: 40000, description: 'the rent' };
  const log = {
    ops: [
      op('o1', { person: 'cy', on_behalf_of: 'bo', payload: rent }),
      op('o2', { person: 'ama', on_behalf_of: 'ama', payload: { kind: SHARE, expense: 'o1' } }),
      op('o3', { person: 'cy', payload: { kind: SHARE, expense: 'o1' } }),
      // A correction's replacement, through another's computer: kept with its own `via`.
      op('o4', { person: 'cy', on_behalf_of: 'ama', corrects: 'o9', payload: { kind: SETTLE, to: 'bo', amount_cents: 100 } }),
    ],
  };
  const acts = actsOf(log);
  assert.deepEqual(acts.map((a) => [a.id, a.by, a.via]), [['o1', 'bo', 'cy'], ['o2', 'ama', null], ['o3', 'cy', null], ['o4', 'ama', 'cy']]);
  const state = fold(acts);
  const untold = fold(acts.map(({ via, ...a }) => a));
  assert.deepEqual([state.balances, state.counted], [untold.balances, untold.counted], 'via moves no money');
  assert.deepEqual(state.expenses.o1.vias, { bo: 'cy' }, "the expense keeps whose computer wrote each person's part");
});
