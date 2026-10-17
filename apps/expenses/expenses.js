// rewritten in E0 from commonwealth-ai sovereign/apps/ring-doc/expenses.js@96261aa71;
// not a fork, so scripts/ports.py does not pair it.
// Expenses: who paid for what, who was in on it, and who owes whom. This file
// is the app; the runtime folds its acts and has never heard of an expense.
//
// `reduce` folds this app's acts as the record orders them on every copy,
// with voided ones gone: `actsOf` reads them from the record a member's node
// serves over loopback (`GET /v1/rail/log`, DESIGN §3). Each act is
// `{id, ts, by, via, op}`: `by` is the person the act speaks for — the person an
// app named when it wrote for someone the door verified, else the person
// whose computer wrote it (in E0, tag e0-bot, the chat speaker) — or null
// when no one did; `via` is the person whose computer wrote it when that is
// someone else, else null, so a page can say whose computer it came through
// (ralph/FINDING_ON_BEHALF_OF.md). It moves no money. So a reducer cannot disagree with another copy about which
// acts it saw or in what order. What it CAN get wrong is the arithmetic, which
// is why the arithmetic is here, in one file, with tests (`expenses.test.mjs`)
// and the laws (`laws.test.mjs`); `canary.mjs` folds two computers' copies.
//
// ── the rules worth reading before you change anything ──
//
// **The person is the act's speaker, never a name in the payload.** Whoever
// wrote the expense paid; whoever wrote a share of it was in on it. No one can
// write an expense in someone else's name, because the op has nowhere to put one.
//
// **The remainder goes to whoever sorts first.** $10.00 three ways is
// 334/333/333, never 333/333/333 with a penny evaporating. The rule is
// arbitrary; what matters is that it is computable from the acts alone.
//
// **Who shares an expense is who confirmed it, never who is in the group.**
// The moment a split reads "everyone here", the same acts divide differently
// once someone joins, and a newcomer silently re-divides every old expense.
//
// **The state is JSON and nothing else** (tag e0-bot, DESIGN C4): no Set, no Map, no
// Date, and nothing kept between calls in a module variable.

export const EXPENSE = 'expense';
export const SHARE = 'share';
export const SETTLE = 'settle';
export const REMINDED = 'reminded';
const MAX_CENTS = 100_000_000;
const MAX_WORDS = 100;

/**
 * The one derivation of a settlement's idempotency key: the same payment
 * recorded twice on one day collapses into one. `day` counts whole days since
 * 1970 from the act's own timestamp, so no clock is read. The accepted cost:
 * two genuinely separate identical payments between one pair on one day
 * collapse; record the second with a different amount if that ever happens.
 */
export function settleKey(from, to, cents, ts) {
  return JSON.stringify({ from, to, cents, day: Math.floor(ts / 86400) });
}

/** Divide `cents` between `people` (non-empty, no repeats), to the cent. */
export function splitShares(cents, people) {
  const sorted = [...people].sort();
  const base = Math.trunc(cents / sorted.length);
  const remainder = cents - base * sorted.length;
  return sorted.map((person, i) => [person, base + (i < remainder ? 1 : 0)]);
}

export function money(cents) {
  const sign = cents < 0 ? '-' : '';
  const a = Math.abs(cents);
  return `${sign}$${Math.trunc(a / 100)}.${String(a % 100).padStart(2, '0')}`;
}

/** "60", "$12.50", "7,5" as cents; null for anything else. */
export function parseCents(text) {
  const m = /^\s*\$?\s*(\d{1,7})(?:[.,](\d{1,2}))?\s*$/.exec(String(text ?? ''));
  if (!m) return null;
  const cents = Number(m[1]) * 100 + Number((m[2] ?? '').padEnd(2, '0'));
  return cents > 0 ? cents : null;
}

const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);
const isCents = (c) => Number.isSafeInteger(c) && c > 0 && c <= MAX_CENTS;
// A key of the state's maps. `__proto__` as a key would set a prototype
// rather than store a value, and money would vanish from a sum.
const isKey = (k) => typeof k === 'string' && k.length > 0 && k.length <= 200 && k !== '__proto__';
const has = (map, k) => typeof k === 'string' && Object.prototype.hasOwnProperty.call(map, k);

/**
 * Everything wrong with one act, as gaps. **The one validator**, for a writer
 * before anything is written and for `reduce` on what was: two copies would be
 * two answers to "is this act writable". A fatal gap means the act
 * is not applied at all — half an expense is not a cautious reading of it, it
 * is a wrong number.
 */
export function validate(state, act) {
  const gap = (code, message) => ({ code, fatal: true, message });
  if (!isObject(act) || !isObject(act.op) || !isKey(act.id) || !Number.isSafeInteger(act.ts)) {
    return [gap('not_an_act', 'an entry with nothing in it')];
  }
  const { op, by } = act;
  if (op.kind === REMINDED) {
    return isKey(op.due) ? [] : [gap('no_due', 'a reminder for nothing')];
  }
  if (!isKey(by)) return [gap('no_one', 'an entry that no one in the chat wrote')];
  if (op.kind === SHARE) {
    return has(state.expenses, op.expense) ? [] : [gap('no_expense', 'a share of an expense that is not in the book')];
  }
  if (op.kind !== EXPENSE && op.kind !== SETTLE) {
    return [gap('unknown_kind', 'an entry a newer version of Expenses may know how to read')];
  }
  const gaps = [];
  if (!isCents(op.amount_cents)) {
    gaps.push(gap('bad_amount', 'an amount that is not a whole number of cents between one cent and $1,000,000'));
  }
  if (op.kind === EXPENSE && (typeof op.description !== 'string' || !op.description.trim())) {
    gaps.push(gap('no_description', 'an expense for nothing in particular'));
  }
  if (op.kind === EXPENSE && typeof op.description === 'string' && op.description.length > MAX_WORDS) {
    gaps.push(gap('long_description', `a description over ${MAX_WORDS} characters`));
  }
  if (op.kind === SETTLE && (!isKey(op.to) || op.to === by)) {
    gaps.push(gap('bad_settle', 'paying back no one, or yourself'));
  }
  return gaps;
}

/**
 * The acts `reduce` folds, from the record's answer to `GET /v1/rail/log`: its
 * ops in the record's order, less every op a correction voided and every op
 * that carries nothing for an app (the group's own acts, and a correction
 * that only voids). A correction's replacement is an act of its own, under
 * the correction's id. `by` is `on_behalf_of` when the op names one, else
 * its `person`; `via` is the `person` when `on_behalf_of` names another,
 * else null. A log with no list of ops is an error, never an empty book.
 */
export function actsOf(log) {
  if (!Array.isArray(log?.ops)) throw new Error(`the record answered no list of acts: ${JSON.stringify(log)}`);
  return log.ops
    .filter((o) => !o.voided && o.payload != null)
    .map((o) => ({ id: o.id, ts: o.ts_unix, by: o.on_behalf_of ?? o.person, via: o.on_behalf_of != null && o.on_behalf_of !== o.person ? o.person : null, op: o.payload }));
}

/** The empty book. */
export function init() {
  return { balances: {}, expenses: {}, settled: {}, reminded: {}, since: {}, gaps: [], counted: 0 };
}

/**
 * Apply one act. Every arm is zero-sum: what one person is owed, others owe.
 * An act that cannot be applied is a gap, never a throw (the totality law).
 */
export function reduce(state, act) {
  const gaps = validate(state, act);
  for (const g of gaps) state.gaps.push({ ...g, id: isObject(act) && typeof act.id === 'string' ? act.id : null });
  if (gaps.length) return state;
  const { op, by, ts, id, via } = act;
  const move = (person, cents) => {
    state.balances[person] = (state.balances[person] || 0) + cents;
    state.since[person] = ts;
  };
  const split = (e, sign) => {
    move(e.payer, sign * e.amount_cents);
    for (const [person, share] of splitShares(e.amount_cents, e.participants)) move(person, -sign * share);
  };

  if (op.kind === EXPENSE) {
    // `vias`: whose computer wrote each person's part, where it was someone else's.
    const e = { payer: by, amount_cents: op.amount_cents, description: op.description.trim(), participants: [by], vias: via ? { [by]: via } : {} };
    state.expenses[id] = e;
    split(e, 1);
  } else if (op.kind === SHARE) {
    const e = state.expenses[op.expense];
    // Confirming twice is confirming once.
    if (e.participants.includes(by)) return state;
    split(e, -1);
    e.participants = [...e.participants, by].sort();
    if (via) e.vias[by] = via;
    split(e, 1);
  } else if (op.kind === SETTLE) {
    const key = settleKey(by, op.to, op.amount_cents, ts);
    if (has(state.settled, key)) return state;
    state.settled[key] = true;
    // `by` hands money over, so their debt shrinks; `to` receives it.
    move(by, op.amount_cents);
    move(op.to, -op.amount_cents);
  } else {
    if (has(state.reminded, op.due)) return state;
    state.reminded[op.due] = true;
  }
  state.counted += 1;
  return state;
}

/** Who pays whom to settle everything, fewest payments first by size. */
export function transfers(balances) {
  const side = (sign) =>
    Object.entries(balances)
      .filter(([, c]) => sign * c > 0)
      .map(([p, c]) => [p, Math.abs(c)])
      .sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1));
  const owers = side(-1);
  const owed = side(1);
  const out = [];
  for (let i = 0, j = 0; i < owers.length && j < owed.length; ) {
    const cents = Math.min(owers[i][1], owed[j][1]);
    out.push({ from: owers[i][0], to: owed[j][0], cents });
    owers[i][1] -= cents;
    owed[j][1] -= cents;
    if (owers[i][1] === 0) i++;
    if (owed[j][1] === 0) j++;
  }
  return out;
}
