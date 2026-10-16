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
// checked by expenses.js's one validator before it is sent.
import { EXPENSE, SETTLE, SHARE, actsOf, init, money, parseCents, reduce, transfers, validate } from './expenses.js';

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
  const { you, people } = read;
  const state = actsOf(read).reduce(reduce, init());
  const said = Object.assign(el('p'), { id: 'said' });
  // An entry checked as expenses.js would read it, then sent; the page
  // drawn again once it is saved, or why not said.
  const send = (op, body = { op }) => {
    const gaps = validate(state, { id: 'new', ts: 0, by: you, op });
    if (gaps.length) return said.replaceChildren(`That isn’t saved: ${gaps.map((g) => g.message).join('; ')}.`);
    const sent = fetch(ACTS, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
    return sent.then(async (w) => (w.ok ? show() : Promise.reject(new Error((await w.json().catch(() => ({}))).error ?? `it answered ${w.status}`)))).catch((e) => said.replaceChildren(`That isn’t saved: ${e.message}.`));
  };
  const mine = state.balances[you] ?? 0;
  const owing = transfers(state.balances);
  const pays = owing.map((t) => {
    const li = el('li', `${t.from === you ? 'You pay' : `${t.from} pays`} ${t.to === you ? 'you' : t.to} ${money(t.cents)}. `);
    if (t.from === you) li.append(button(`Say you paid ${t.to} ${money(t.cents)}`, null, () => send({ kind: SETTLE, to: t.to, amount_cents: t.cents })));
    return li;
  });
  // A person's name, and whose computer wrote their part when it was someone else's.
  const through = (name, via) => (via ? `${name}, through ${via === you ? 'your' : `${via}’s`} computer` : name);
  const spent = Object.entries(state.expenses).map(([id, e]) => {
    const payer = through(e.payer === you ? 'You' : e.payer, e.vias[e.payer]);
    const split = e.participants.map((p) => (p === e.payer ? p : through(p, e.vias[p])));
    const li = el('li', `${payer}${e.vias[e.payer] ? ',' : ''} paid ${money(e.amount_cents)} for ${e.description}, split between ${split.join(split.some((p) => p.includes(',')) ? '; ' : ', ')}. `);
    if (!e.participants.includes(you)) li.append(button('Say you were in on it', null, () => send({ kind: SHARE, expense: id })));
    return li;
  });
  const add = el('p', '', field('What it was for: ', 'what'), ' ', field('How much: ', 'amount'), ' ', button('Add it, paid by you', 'add', () => {
    const cents = parseCents(document.getElementById('amount').value);
    if (cents === null) return said.replaceChildren('Say how much, in dollars and cents, such as 12.50.');
    return send({ kind: EXPENSE, amount_cents: cents, description: document.getElementById('what').value });
  }));
  app.replaceChildren(
    el('h1', 'Money'),
    el('p', mine > 0 ? `You’re owed ${money(mine)}.` : mine < 0 ? `You owe ${money(-mine)}.` : 'You’re square with everyone.'),
    el('h2', 'Who pays whom'),
    pays.length ? el('ul', '', ...pays) : el('p', 'Nobody owes anybody.'),
    el('h2', 'What people paid for'),
    spent.length ? el('ul', '', ...spent) : el('p', 'Nobody has added anything yet.'),
    el('p', `When someone adds what they paid, it is split between them and whoever says they were in on it. Everyone in your group — ${people.join(', ')} — sees it.`),
    el('h2', 'Add what you paid'),
    add,
    said,
  );
  if (state.gaps.length) app.append(el('p', `${state.gaps.length} ${state.gaps.length === 1 ? 'entry' : 'entries'} can’t be read by this version of Money, so ${state.gaps.length === 1 ? 'it isn’t' : 'they aren’t'} counted.`));
}

show();
