// SPDX-License-Identifier: AGPL-3.0-or-later
// The ask, at /ask (page-acts (c); home.js `lentIn` and `ask`, :63-66,
// :238-246): what the people of each group you are in lend, read, then
// svrngs's question in its one wording (question.js `question`) around the
// lender's words, or why it is not asked (`unlent`). Yes posts the yes to
// /choices, then leaves for the app at `back` through /to, as Not now
// leaves at once, by location.assign at the edge; a refusal is said after
// the buttons, in place of any before it, where v1 appended each (h:343;
// page-acts (d)). Its machine, ASK, is the place's own (ui/view.js
// `useMachine`).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { go, machine, no } from '../ui/machine.js?v=__VERSION__';
import { question, unlent } from '../question.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';
import { label } from './index.js?v=__VERSION__';
import { post } from './running.js?v=__VERSION__';

// A group's lends, none when they cannot be read, or who is in it cannot (h:63-66).
const lentIn = (groups) => Promise.all(groups.map((g) => (g.removed === false ? get(`/v1/lends?namespace=${encodeURIComponent(g.id)}`).then((l) => l.lent, () => []) : []))).then((a) => a.flat());
// The app at `back`, kept only on its own origin, as open.js judges its way back (h:233-236).
const leave = (app, back) => (location.assign(`/to?${new URLSearchParams({ lend: label(app), at: ((u) => (u.origin === new URL(app.origin).origin ? u.pathname + u.search + u.hash : '/'))(new URL(back ?? '/', app.origin)) })}`), { type: 'gone', why: `to ${label(app)}` });
const ASK = (q) => ({ states: { ask: ['reading', 'asking', 'sending', 'unlent'] },
  io: { read: ({ groups }) => lentIn(groups).then((lent) => ({ type: 'lent', lent, why: `${lent.length} lent` })), yes: ({ lend, app }) => post('/choices', { lend: label(lend), app: label(app), said: 'yes' }).then(() => leave(app, q.back)), leave: ({ app }) => leave(app, q.back) },
  step: (s, m) => ({
    lent: () => ((lend, app) => (!lend || !app || !lend.asks ? go(s, { ask: 'unlent' }) : go(s, { ask: 'asking' }, { lend, app })))(...[q.lend, q.app].map((x) => (m.lent ?? []).find((l) => label(l) === x))),
    yes: () => (s.at.ask === 'asking' ? go(s, { ask: 'sending' }, {}, { run: 'yes', lend: s.lend, app: s.app, failed: 'unsaid' }) : no(s, `the ask is ${s.at.ask}`)),
    not: () => (s.lend ? go(s, {}, {}, { run: 'leave', app: s.app, failed: 'unsaid' }) : no(s, 'nothing is asked')),
    unsaid: () => go(s, { ask: 'asking' }, { said: m.why }), gone: () => go(s, {}),
  })[m.type]?.() ?? no(s, `the ask takes no ${m.type}`) });

function Asked({ s: h }) {
  const [s, send] = useMachine(() => machine('ASK', ASK(h.q), { at: { ask: 'reading' }, said: '' }, [{ run: 'read', groups: h.groups, failed: 'lent' }]));
  if (s.at.ask === 'reading') return null;
  return s.at.ask === 'unlent' ? html`<p>${unlent}</p>` : html`<section><h2>${question(s.app, s.lend)}</h2><blockquote>${s.lend.asks}</blockquote><button onClick=${() => send({ type: 'yes' })}>Yes</button><button onClick=${() => send({ type: 'not' })}>Not now</button>${s.said}</section>`;
}
export const view = { ask: (props) => html`<${Asked} ...${props} />` };
