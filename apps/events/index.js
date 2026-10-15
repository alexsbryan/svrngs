// SPDX-License-Identifier: AGPL-3.0-or-later
// Events' page, carried in a group (v1ui-shelf (b); svrngs DESIGN §3, "Or a
// folder the group carries"): each event posted in the group, who is coming
// and who came, your own answer to say or take back, and a form to post one.
// It reads the group at its own origin, `svrngs/acts` — you, the people in
// the group, and Events' own entries in the shape the group's record gives
// them, changes of mind included, which events.js folds unchanged (`actsOf`,
// `reduce`, `answers`) — and writes there; svrngs writes each entry as you.
// Each entry is checked by events.js's one validator before it is sent. An
// event's id is made here, as the host's computer made one (server.mjs).
import { CAME, COMING, EVENT, actsOf, answers, guestName, init, ownActs, reduce, validate } from './events.js';

const ACTS = 'svrngs/acts';
const app = document.getElementById('app');
const el = (tag, text, ...kids) => {
  const e = document.createElement(tag);
  e.append(text ?? '', ...kids);
  return e;
};
const button = (text, id, act) => Object.assign(el('button', text), { id, onclick: act });
const field = (words, id) => el('label', words, Object.assign(el('input'), { id }));
const value = (id) => document.getElementById(id).value.trim();
const named = (who, you) => (who === you ? 'you' : who);

async function show() {
  const r = await fetch(ACTS);
  const read = await r.json().catch(() => ({}));
  if (!r.ok) return app.replaceChildren(el('h1', 'Events'), el('p', `Your group’s events can’t be read just now: ${read.error ?? `it answered ${r.status}`}.`));
  const { you, people } = read;
  const state = actsOf(read).reduce(reduce, init());
  const { events, gaps } = answers(state);
  const said = Object.assign(el('p'), { id: 'said' });
  // Each entry checked as events.js would read it, then sent in turn; the
  // page drawn again once they are saved, or why not said.
  const send = async (...bodies) => {
    for (const body of bodies) {
      const wrong = validate({ id: 'new', ts: 0, by: you, op: body.op, corrects: body.corrects ?? null });
      if (wrong.length) return said.replaceChildren(`That isn’t saved: ${wrong.map((g) => g.message).join('; ')}.`);
      const w = await fetch(ACTS, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
      if (!w.ok) return said.replaceChildren(`That isn’t saved: ${(await w.json().catch(() => ({}))).error ?? `it answered ${w.status}`}.`);
    }
    return show();
  };
  const listed = Object.entries(events).map(([event, e]) => {
    const coming = Object.entries(e.coming).map(([p, guests]) => [named(p, you), ...guests].join(' + '));
    const came = [...e.came.map((p) => named(p, you)), ...Object.entries(e.guests_came).flatMap(([p, gs]) => gs.map((g) => `${g}, ${named(p, you)}’s guest`))];
    const s = el('section', '', el('h2', e.what), el('p', `${e.when || 'Some time'}, ${e.where ? `at ${e.where}` : 'somewhere'}. ${e.host === you ? 'You host it.' : `${e.host} hosts it.`}`));
    s.append(el('p', coming.length ? `Coming: ${coming.join(', ')}.` : 'Nobody has said they’re coming yet.'), el('p', came.length ? `Came: ${came.join(', ')}.` : 'Nobody has said they came yet.'));
    const yours = ownActs(state, you, event, COMING);
    if (yours.length) {
      s.append(button('Say you’re not coming after all', null, () => send(...yours.map((id) => ({ corrects: id, op: null })))));
    } else {
      const guest = Object.assign(el('input'), { placeholder: 'their name' });
      s.append(el('label', 'Bringing someone? ', guest), ' ', button('Say you’re coming', null, () => send({ op: { kind: COMING, event, ...(guestName(guest.value) ? { guests: [guestName(guest.value)] } : {}) } })));
    }
    if (!e.came.includes(you)) s.append(' ', button('Say you came', null, () => send({ op: { kind: CAME, event } })));
    return s;
  });
  const id = () => [...crypto.getRandomValues(new Uint8Array(8))].map((b) => b.toString(16).padStart(2, '0')).join('');
  const post = el('p', '', field('What: ', 'what'), ' ', field('When: ', 'when'), ' ', field('Where: ', 'where'), ' ', button('Post it to your group', 'post', () => send({ op: { kind: EVENT, event: id(), what: value('what'), when: value('when'), where: value('where') } })));
  app.replaceChildren(
    el('h1', 'Events'),
    ...(listed.length ? listed : [el('p', 'Nobody has posted an event yet.')]),
    el('h2', 'Post an event'),
    el('p', `Everyone in your group — ${people.join(', ')} — sees it and can say they’re coming. Nobody is shown saying no.`),
    post,
    said,
  );
  if (gaps.length) app.append(el('p', `${gaps.length} ${gaps.length === 1 ? 'entry' : 'entries'} can’t be counted: ${gaps.map((g) => g.message).join('; ')}.`));
}

show();
