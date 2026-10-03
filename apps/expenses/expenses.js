// ported from commonwealth-ai sovereign/apps/ring-doc/expenses.js@96261aa71
// Expenses: who paid for what, who was in on it, and who owes whom. This file
// is the app; the runtime folds its acts and has never heard of an expense.
//
// The runtime hands `reduce` this app's acts, in one order every copy computes,
// with voided ones gone (DESIGN C1, C4). Each act is `{id, ts, by, op}`: `by`
// is the chat speaker the act speaks for, or null when no one in the chat
// wrote it. So a reducer cannot disagree with another copy about which acts it
// saw or in what order. What it CAN get wrong is the arithmetic, which is why
// the arithmetic is here, in one file, with tests (`expenses.test.mjs`) and
// the laws (`laws.test.mjs`).
//
// ── the rules worth reading before you change anything ──
//
// **The person is the act's speaker, never a name in the payload.** Whoever
// typed /paid paid; whoever tapped Confirm was in on it. No one can write an
// expense in someone else's name, because the op has nowhere to put one.
//
// **The remainder goes to whoever sorts first.** $10.00 three ways is
// 334/333/333, never 333/333/333 with a penny evaporating. The rule is
// arbitrary; what matters is that it is computable from the acts alone.
//
// **Who shares an expense is who confirmed it, never who is in the group.**
// The moment a split reads "everyone here", the same acts divide differently
// once someone joins, and a newcomer silently re-divides every old expense.
//
// **The state is JSON and nothing else** (DESIGN C4): no Set, no Map, no
// Date, and nothing kept between calls in a module variable.

export const EXPENSE = 'expense';
export const SHARE = 'share';
export const SETTLE = 'settle';
export const REMINDED = 'reminded';
/** A debt left unchanged this long is due a reminder. */
export const REMIND_AFTER_SECS = 3 * 24 * 60 * 60;
const CONFIRM = 'confirm:';
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
 * Everything wrong with one act, as gaps. **The one validator**, called by
 * `propose` before anything is written and by `reduce` on what was: two copies
 * would be two answers to "is this act writable". A fatal gap means the act
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
  const { op, by, ts, id } = act;
  const move = (person, cents) => {
    state.balances[person] = (state.balances[person] || 0) + cents;
    state.since[person] = ts;
  };
  const split = (e, sign) => {
    move(e.payer, sign * e.amount_cents);
    for (const [person, share] of splitShares(e.amount_cents, e.participants)) move(person, -sign * share);
  };

  if (op.kind === EXPENSE) {
    const e = { payer: by, amount_cents: op.amount_cents, description: op.description.trim(), participants: [by] };
    state.expenses[id] = e;
    split(e, 1);
  } else if (op.kind === SHARE) {
    const e = state.expenses[op.expense];
    // Confirming twice is confirming once.
    if (e.participants.includes(by)) return state;
    split(e, -1);
    e.participants = [...e.participants, by].sort();
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

const nameOf = (people, p) => (isObject(people) && has(people, p) && typeof people[p] === 'string' ? people[p] : 'someone who left');

function standing(state, people) {
  const lines = transfers(state.balances).map(
    (t) => `${nameOf(people, t.from)} owes ${nameOf(people, t.to)} ${money(t.cents)}`,
  );
  return lines.length ? `Who owes whom:\n${lines.join('\n')}` : "Everyone's even.";
}

/**
 * A chat input as the act to write, `{op}`, or an answer that writes nothing,
 * `{text, private}`. `input` is `{command, options, button, by}`.
 */
export function propose(state, input) {
  const answer = (text, priv = true) => ({ text, private: priv });
  if (!isObject(input)) return answer("I didn't understand that.");
  const options = isObject(input.options) ? input.options : {};
  const by = input.by;
  let op;
  if (input.command === 'paid') {
    op = { kind: EXPENSE, amount_cents: parseCents(options.amount), description: String(options.what ?? '').trim() };
  } else if (input.command === 'settle') {
    op = { kind: SETTLE, amount_cents: parseCents(options.amount), to: options.to };
  } else if (input.command === 'owe') {
    return answer(standing(state, input.people), false);
  } else if (typeof input.button === 'string' && input.button.startsWith(CONFIRM)) {
    const expense = input.button.slice(CONFIRM.length);
    const e = has(state.expenses, expense) ? state.expenses[expense] : null;
    if (e && e.participants.includes(by)) return answer(`You're already in on ${e.description}.`);
    op = { kind: SHARE, expense };
  } else {
    return answer("I don't know that one.");
  }
  const gaps = validate(state, { id: 'proposed', ts: 0, by, op });
  if (!gaps.length) return { op };
  if (gaps.some((g) => g.code === 'bad_amount')) return answer('How much? Try a number like 60 or 12.50.');
  if (gaps.some((g) => g.code === 'bad_settle')) return answer('Who did you pay back? Pick someone other than yourself.');
  if (gaps.some((g) => g.code === 'no_expense')) return answer("That expense isn't in the book any more.");
  return answer(`I can't write that down: ${gaps.map((g) => g.message).join('; ')}.`);
}

/**
 * A text view, `{text, buttons}`. `view` is `{people, act}`: `people` names
 * each speaker, and `act`, when given, is the id of the act just written.
 */
export function say(state, view) {
  const v = isObject(view) ? view : {};
  const people = v.people;
  const e = has(state.expenses, v.act) ? state.expenses[v.act] : null;
  if (e) {
    return {
      text: `${nameOf(people, e.payer)} paid ${money(e.amount_cents)} for ${e.description}. If you had some, tap Confirm and I'll split it.`,
      buttons: [{ id: `${CONFIRM}${v.act}`, label: 'Confirm' }],
    };
  }
  return { text: standing(state, people), buttons: [] };
}

/**
 * What is due at `now`: a reminder for each person whose debt has not changed
 * in `REMIND_AFTER_SECS`. Each item's `id` changes when their debt does, and a
 * `reminded` act naming it is what stops it being due.
 */
export function due(state, now) {
  const items = [];
  for (const [person, cents] of Object.entries(state.balances)) {
    const id = `remind:${person}:${state.since[person]}`;
    if (cents < 0 && now - state.since[person] >= REMIND_AFTER_SECS && !has(state.reminded, id)) {
      items.push({ id, person, text: `a reminder: you owe ${money(-cents)}. Type /owe to see whom to pay.` });
    }
  }
  return items;
}
