// SPDX-License-Identifier: AGPL-3.0-or-later
// Money's page, carried in a group (v1ui-shelf (b); svrngs DESIGN §3, "Or a
// folder the group carries"): what you owe or are owed, who pays whom to
// settle, each expense, and a form to add one. It reads the group at its own
// origin, `svrngs/acts` — you, the people in the group, and Money's own
// entries in the shape the group's record gives them, which expenses.js
// folds unchanged (`actsOf`, `reduce`) — and writes there; svrngs writes each
// entry as you, so nothing here names who wrote it. An entry another
// person's computer wrote in someone's name says so: "bo, through cy’s
// computer" (gv-via; the phone's own words, web/app.js). Each entry is
// checked by expenses.js's one validator before it is sent. Each person is
// kept by their first key, you by `you_key`, and said by name, two of one
// name told apart by when each joined (`called`; keys-names).
import { EXPENSE, NOT_IN, SETTLE, SHARE, actsOf, called, init, money, parseCents, reduce, splitShares, transfers, validate } from './expenses.js';

const ACTS = 'svrngs/acts';
const app = document.getElementById('app');
const el = (tag, text, ...kids) => {
  const e = document.createElement(tag);
  e.append(text ?? '', ...kids);
  return e;
};
const button = (text, id, act) => Object.assign(el('button', text), { id, onclick: act });
const field = (words, id) => el('label', words, Object.assign(el('input'), { id }));

async function show() {
  const r = await fetch(ACTS);
  const read = await r.json().catch(() => ({}));
  if (!r.ok) return app.replaceChildren(el('h1', 'Money'), el('p', `Your group’s money can’t be read just now: ${read.error ?? `it answered ${r.status}`}.`));
  const { you, you_key, people, spans } = read;
  const me = you_key ?? you;
  const state = actsOf(read).reduce(reduce, init());
  const nameOf = (p) => (Object.hasOwn(state.names, p) ? state.names[p] : p);
  const name = (p) => called(spans, p, nameOf(p));
  const said = Object.assign(el('p'), { id: 'said' });
  // An entry checked as expenses.js would read it, then sent; the page
  // drawn again once it is saved, or why not said.
  const send = (op, body = { op }) => {
    const gaps = validate(state, { id: 'new', ts: 0, by: you, key: you_key ?? null, op });
    if (gaps.length) return said.replaceChildren(`That isn’t saved: ${gaps.map((g) => g.message).join('; ')}.`);
    const sent = fetch(ACTS, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
    return sent.then(async (w) => (w.ok ? show() : Promise.reject(new Error((await w.json().catch(() => ({}))).error ?? `it answered ${w.status}`)))).catch((e) => said.replaceChildren(`That isn’t saved: ${e.message}.`));
  };
  const mine = state.balances[me] ?? 0;
  const owing = transfers(state.balances, state.names);
  // A payment says whom it pays by name, for an older Money, and by key.
  const paying = (t) => ({ kind: SETTLE, to: nameOf(t.to), ...(t.to !== nameOf(t.to) && { to_key: t.to }), amount_cents: t.cents });
  const pays = owing.map((t) => {
    const li = el('li', `${t.from === me ? 'You pay' : `${name(t.from)} pays`} ${t.to === me ? 'you' : name(t.to)} ${money(t.cents)}. `);
    if (t.from === me) li.append(button(`Say you paid ${name(t.to)} ${money(t.cents)}`, null, () => send(paying(t))));
    return li;
  });
  // A person's name, and whose computer wrote their part when it was someone else's.
  const through = (name, via) => (via ? `${name}, through ${via === you ? 'your' : `${via}’s`} computer` : name);
  // Who the payer named and hasn't said yet, and who said not them, by name
  // (expenses.js `named`): said on the line, never counted.
  const saying = (names, words) => (names.length ? `; ${words(names.join(', '))}` : '');
  const spent = Object.entries(state.expenses).map(([id, e]) => {
    const payer = through(e.payer === me ? 'You' : name(e.payer), e.vias[e.payer]);
    const split = e.participants.map((p) => (p === e.payer ? name(p) : through(name(p), e.vias[p])));
    const others = saying(Object.keys(e.waiting ?? {}), (n) => `waiting on ${n}`) + saying(Object.keys(e.not ?? {}), (n) => `${n} wasn’t in on it`);
    const li = el('li', `${payer}${e.vias[e.payer] ? ',' : ''} paid ${money(e.amount_cents)} for ${e.description}, split between ${split.join(split.some((p) => p.includes(',')) ? '; ' : ', ')}${others}. `);
    if (!e.participants.includes(me)) li.append(button('Say you were in on it', null, () => send({ kind: SHARE, expense: id })));
    return li;
  });
  const everyone = spans?.filter((s) => s.left == null).map((s) => called(spans, s.person_key, s.person)) ?? people;
  // Who was in on it: each other person in the group, unticked. A name ticked
  // is asked, and counts only once they say so themselves (expenses.js).
  const ticks = (spans?.filter((s) => s.left == null).map((s) => s.person) ?? people).filter((p) => p !== you).map((p) => [p, Object.assign(el('input'), { type: 'checkbox' })]);
  const who = ticks.length ? ['Who was in on it? ', ...ticks.flatMap(([p, t]) => [el('label', '', t, ` ${p}`), ' '])] : [];
  const add = el('p', '', field('What it was for: ', 'what'), ' ', field('How much: ', 'amount'), ' ', ...who, button('Add it, paid by you', 'add', () => {
    const cents = parseCents(document.getElementById('amount').value);
    if (cents === null) return said.replaceChildren('Say how much, in dollars and cents, such as 12.50.');
    const named = ticks.filter(([, t]) => t.checked).map(([p]) => p);
    return send({ kind: EXPENSE, amount_cents: cents, description: document.getElementById('what').value, ...(named.length && { named }) });
  }));
  // What someone named you in, asked of you, with what your share would be.
  const asked = Object.entries(state.expenses).filter(([, e]) => Object.hasOwn(e.waiting ?? {}, you)).map(([id, e]) => {
    const yours = splitShares(e.amount_cents, [...e.participants, me], state.names).find(([p]) => p === me)[1];
    return el('p', `${name(e.payer)} says you were in on ${e.description}: ${money(yours)}. `, button('That’s right', null, () => send({ kind: SHARE, expense: id })), ' ', button('Not me', null, () => send({ kind: NOT_IN, expense: id })));
  });
  app.replaceChildren(
    el('h1', 'Money'),
    el('p', mine > 0 ? `You’re owed ${money(mine)}.` : mine < 0 ? `You owe ${money(-mine)}.` : 'You’re square with everyone.'),
    ...asked,
    el('h2', 'Who pays whom'),
    pays.length ? el('ul', '', ...pays) : el('p', 'Nobody owes anybody.'),
    el('h2', 'What people paid for'),
    spent.length ? el('ul', '', ...spent) : el('p', 'Nobody has added anything yet.'),
    el('p', `When someone adds what they paid, it is split between them and whoever says they were in on it. Everyone in your group — ${everyone.join(', ')} — sees it.`),
    el('h2', 'Add what you paid'),
    add,
    said,
  );
  // Each entry not counted, and why: one this version can't read, or
  // someone's change to an entry that only its author can change.
  if (state.gaps.length) app.append(el('p', `${state.gaps.length} ${state.gaps.length === 1 ? 'entry' : 'entries'} can’t be counted: ${state.gaps.map((g) => g.message).join('; ')}.`));
}

show();
