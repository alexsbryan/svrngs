// ported from commonwealth-ai sovereign/apps/ring-doc/expenses.test.mjs@96261aa71
// The money rules, pinned. Each name says which mistake loses money. What is
// not tested here is order: every copy folding the same acts in the same order
// is the record's property (svrngs-core `admit`), not the reducer's. The laws
// every reducer holds are `laws.test.mjs`.

import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  EXPENSE,
  REMIND_AFTER_SECS,
  SETTLE,
  SHARE,
  due,
  init,
  money,
  parseCents,
  propose,
  reduce,
  say,
  splitShares,
  transfers,
  validate,
} from './expenses.js';

const [SAM, AMA, BO, CY] = ['discord:1', 'discord:2', 'discord:3', 'discord:4'];
const PEOPLE = { [SAM]: 'Sam', [AMA]: 'Ama', [BO]: 'Bo', [CY]: 'Cy' };
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

test('propose writes only what reduce would apply', () => {
  const state = fold([]);
  const p = propose(state, { command: 'paid', options: { amount: '60', what: 'pizza' }, by: SAM });
  assert.deepEqual(p, { op: { kind: EXPENSE, amount_cents: 6000, description: 'pizza' } });
  assert.equal(propose(state, { command: 'paid', options: { amount: 'lots', what: 'pizza' }, by: SAM }).private, true);
  assert.equal(propose(state, { button: 'confirm:act-nowhere', by: AMA }).private, true);
  assert.equal(propose(state, { command: 'settle', options: { amount: '5', to: SAM }, by: SAM }).private, true);
  assert.equal(propose(state, { command: 'what' }).private, true);
});

test('the pizza, as the chat sees it', () => {
  let state = init();
  const write = (by, input) => {
    const { op } = propose(state, { ...input, by });
    const a = act(by, op);
    state = reduce(state, a);
    return say(state, { people: PEOPLE, act: a.id });
  };
  const posted = write(SAM, { command: 'paid', options: { amount: '60', what: 'pizza' } });
  assert.equal(posted.text, "Sam paid $60.00 for pizza. If you had some, tap Confirm and I'll split it.");
  const [button] = posted.buttons;
  assert.equal(button.label, 'Confirm');
  write(AMA, { button: button.id });
  write(BO, { button: button.id });
  assert.equal(write(CY, { button: button.id }).text, 'Who owes whom:\nAma owes Sam $15.00\nBo owes Sam $15.00\nCy owes Sam $15.00');
  assert.equal(propose(state, { button: button.id, by: CY }).text, "You're already in on pizza.");
  assert.equal(propose(state, { command: 'owe', by: AMA, people: PEOPLE }).private, false);
});

test('a debt left alone is due a reminder once, until it changes', () => {
  const pizza = paid(SAM, 6000, 'pizza', 1000);
  let state = fold([pizza, share(AMA, pizza.id, 1000)]);
  assert.deepEqual(due(state, 1000 + REMIND_AFTER_SECS - 1), []);
  const [item] = due(state, 1000 + REMIND_AFTER_SECS);
  assert.equal(item.person, AMA);
  assert.deepEqual(item.fulfil, { kind: 'reminded', due: item.id });
  state = reduce(state, act(null, item.fulfil, 1000 + REMIND_AFTER_SECS));
  assert.deepEqual(due(state, 1000 + 10 * REMIND_AFTER_SECS), []);
  state = reduce(state, settle(AMA, SAM, 1000, 2000 + REMIND_AFTER_SECS));
  assert.equal(due(state, 2000 + 2 * REMIND_AFTER_SECS).length, 1, 'a changed debt is due again');
});
