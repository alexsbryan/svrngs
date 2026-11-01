// SPDX-License-Identifier: AGPL-3.0-or-later
// The person asked whether an app may use a lend (page-acts (b); web/app.js
// `asking`, :298-325): a lent page sends its person to
// `#ask=<lend>&app=<its label>&at=<path>` (lend.js `asking`), and once the
// group is read the page asks in svrngs's words around the lender's, its
// heading question.js `question`, the lender's in #words; Yes posts the
// person's yes to its lender's door (web/v2/uses.js `choose`), then opens
// the app again at `at`, as Not now does at once (web/v2/index.js `open`).
// An app or a lend not lent here, or not asking, is said, and nothing asked.
// The ask is sent by the page once its list is read (`Asking`), as no flow
// sends it: the read is enter's.
import { html, useEffect } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { question, unlent } from '../question.js?v=__VERSION__';
import { choose } from './uses.js?v=__VERSION__';

export const owns = ['asking', 'answer'];
export const states = { ask: ['none', 'asking', 'sending', 'refused'] };
const lendOf = (s, label) => s.lent?.find((l) => l.label === label), back = (s) => ({ run: 'send', m: { type: 'open', label: s.asking.app, at: s.asking.at } });
export const takes = {
  ask: (s, m, [lend, app] = [m.lend, m.app].map((x) => lendOf(s, x))) => (s.at.ask !== 'none' ? no(s, `the ask is ${s.at.ask} already`)
    : !lend?.asks || !app ? go(s, { ask: 'refused' }, { asking: null, answer: unlent }) : go(s, { ask: 'asking' }, { asking: { lend: m.lend, app: m.app, at: m.at ?? '/' } })),
  say: (s) => (s.at.ask !== 'asking' ? no(s, `nothing is asked: the ask is ${s.at.ask}`) : go(s, { ask: 'sending' }, {}, { run: 'answer', group: s.group, l: lendOf(s, s.asking.lend), app: s.asking.app, failed: 'unsaid' })),
  not: (s) => (s.at.ask !== 'asking' ? no(s, `nothing is asked: the ask is ${s.at.ask}`) : go(s, {}, {}, back(s))),
  said: (s) => (s.at.ask !== 'sending' ? no(s, 'no answer is being sent') : go(s, {}, {}, back(s))),
  unsaid: (s, m) => (s.at.ask !== 'sending' ? no(s, 'no answer is being sent') : go(s, { ask: 'refused' }, { answer: m.said ?? 'Your answer could not be sent.' })),
};
export const heading = { ask: (s) => (['asking', 'sending'].includes(s.at.ask) ? [question(lendOf(s, s.asking.app), lendOf(s, s.asking.lend)), 'good'] : s.at.ask === 'refused' && s.asking && [s.answer, 'bad']) };
export const response = { ask: (s) => s.at.ask === 'refused' && !s.asking && [s.answer] };

// Sent once the list is read and the address asks, and then not again.
const Asking = ({ s, send }) => (useEffect(() => void (s.at.list === 'read' && s.at.ask === 'none' && send({ type: 'ask', lend: s.wants.ask, app: s.wants.app, at: s.wants.at, why: `whether ${s.wants.app} may use ${s.wants.ask}` })), [s.at.list]), null);
export const view = { ask: ({ s, send }) => html`<form id="ask" hidden=${s.at.ask !== 'asking'} onSubmit=${(e) => (e.preventDefault(), send({ type: 'say' }))}><blockquote id="words">${s.asking && lendOf(s, s.asking.lend)?.asks}</blockquote><button>Yes</button><button type="button" id="not" onClick=${() => send({ type: 'not' })}>Not now</button>${s.wants?.ask && html`<${Asking} s=${s} send=${send} />`}</form>` };
export const io = { answer: ({ group, l, app }) => choose(group, l, app, 'yes').then(() => ({ type: 'said', why: `the yes kept at ${l.person}'s computer` }), (e) => ({ type: 'unsaid', said: e.said, why: e.message })) };
