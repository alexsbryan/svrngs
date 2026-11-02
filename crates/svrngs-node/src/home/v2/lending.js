// SPDX-License-Identifier: AGPL-3.0-or-later
// "What you lend" (page-index (e); home.js `lending` and `signed`,
// :252-287): what this computer lends, each with how many it lets in at
// once — its person's choice, a number or everyone in their groups — to
// set, clear, sign people in to or stop here, taken by the door at its next
// request. An app a group carries is the group's, not this computer's to
// change, so not listed. Its machine, LENDING, is the place's own
// (ui/view.js `useMachine`), each read counted, so an older answer is
// refused by name; it reads as it is drawn and again on each of HOME's
// rounds (`again`, `visible`), and the last read stays drawn while the next
// is asked. Each lend is a row with a machine of its own, CHANGE, its
// region `change`: a second press while one change is under way is refused.
// A change is answered by HOME's `again`, the groups and this read again,
// where v1 loaded the page again (h:283; page-index (e)'s fix), and the
// typed number and what was said go with it, as v1's load drops them. Why
// a change or the number is refused is said once, beside the buttons, the
// last in place of the one before, where v1 appended each again (h:283,
// :285; page-index (e)'s second fix). The
// secret from "Sign people in with svrngs" is shown in its row's `shown`,
// once, and nowhere else: never in a request, a link or a trace line
// (SIGN_IN_WITH_SVRNGS §5 "Turning it on"); a read again keeps the row, so
// keeps it shown, and a load of the page, which reads none, shows it no more.
import { Act, ask, Field, go, html, late, machine, no, useEffect, useMachine } from '../ui/kit.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';
import { post } from './running.js?v=__VERSION__';

const LENDING = { states: { lent: ['reading', 'read', 'unread'] }, io: { read: ({ n }) => get('/lent').then(({ lent, signing }) => ({ type: 'answered', n, lent: lent.filter((l) => !l.group), signing, why: `${lent.length} lent` })) },
  step: (s, m, [asked, n] = ask(s, 'lent'), old = m.type !== 'read' && late(s, 'lent', m.n)) => (old ? no(s, old) : ({ read: () => go(s, { lent: 'reading' }, { asked }, { run: 'read', n, failed: 'unread' }),
    answered: () => go(s, { lent: 'read' }, { lent: m.lent, signing: m.signing ?? [] }), unread: () => go(s, { lent: 'unread' }, { why: m.why }) })[m.type]?.() ?? no(s, `what you lend takes no ${m.type}`)) };
// The change asked, `run` with what it carries, unless one is under way.
const changing = (s, effect) => (s.at.change === 'changing' ? no(s, 'a change is under way, and is answered first') : go(s, { change: 'changing' }, {}, effect));
const CHANGE = (lend, home) => ({ states: { change: ['idle', 'changing', 'refused'] },
  io: { change: ({ to }) => post('/lent', { lend, to }).then(() => (home({ type: 'again' }), { type: 'changed', why: 'the groups read again' })), sign: () => post('/lent', { lend, to: 'sign-in' }).then((c) => ({ type: 'signed', c, why: 'its secret shown here once' })) },
  step: (s, m) => ({ typed: () => go(s, {}, { many: m.value }), change: () => changing(s, { run: 'change', to: m.to, failed: 'refused' }), sign: () => changing(s, { run: 'sign', failed: 'refused' }),
    many: () => (/^[1-9][0-9]*$/.test(m.value) ? changing(s, { run: 'change', to: { limit: Number(m.value) }, failed: 'refused' }) : go(s, { change: 'refused' }, { said: ' Say how many: 1 or more.' })),
    changed: () => go(s, { change: 'idle' }, { many: undefined, said: '' }), signed: () => go(s, { change: 'idle' }, { shown: m.c }), refused: () => go(s, { change: 'refused' }, { said: ` ${m.why}` }) })[m.type]?.() ?? no(s, `a lend here takes no ${m.type}`) });

// What a lend's app form is filled with, the one time its lender turns
// signing people in with svrngs on: the issuer, the client and its secret,
// and the deviations in their words (h:256).
const signed = (c) => html`<p>Signing people in with svrngs. In the app’s own settings, choose OpenID Connect, and fill in: issuer <code>${c.issuer}</code>, client id <code>${c.client_id}</code>, client secret <code>${c.client_secret}</code>${`. ${c.deviations} Copy the secret now: it is not shown again. `}</p>`;
function Lent({ l, signs, home }) {
  const [s, send] = useMachine(() => machine('CHANGE', CHANGE(l.name, home), { at: { change: 'idle' }, said: '' })), n = l.limit, value = s.many ?? String(n ?? '');
  const act = (words, on) => html`<${Act} words=${words} doing=${{ state: s.at.change === 'changing' ? 'busy' : 'idle' }} on=${on} send=${send} />`;
  return html`<li>${`${l.name}: ${n ? `up to ${n} ${n === 1 ? 'person' : 'people'}` : 'everyone in your groups'} at once, ${l.using} using it now. `}<${Field} words="How many people at once: " value=${value} attrs=${{ type: 'number', min: 1 }} send=${send} />${act('Let in this many', { type: 'many', value })}${n && act('Let everyone in', { type: 'change', to: { limit: null } })}${signs ? 'Signs people in with svrngs. ' : act('Sign people in with svrngs', { type: 'sign' })}${act('Stop lending', { type: 'change', to: 'stop' })}${s.said}${s.shown && signed(s.shown)}</li>`;
}
// HOME's: the page's foot held until what you lend is first read, as v1
// drew its address after it (h:326-328): drawn at once, a driver read the
// list empty (main.rs:2524 on v2).
export const take = (s) => go(s, {}, { lentRead: true });
export const holds = (s) => s.groups.length > 0 && !s.lentRead;
function Lending({ round, home, told }) {
  const [s, send] = useMachine(() => machine('LENDING', LENDING, { at: { lent: 'reading' } }));
  useEffect(() => send({ type: 'read' }), [round]);
  useEffect(() => void (!told && s.at.lent !== 'reading' && home({ type: 'told', flow: 'lending', why: `what you lend is ${s.at.lent}` })), [s.at.lent]);
  return html`<section id="lent"><h2>What you lend</h2>${s.at.lent === 'unread' ? html`<p>This computer cannot read what you lend: ${s.why}</p>` : s.lent && html`${!s.lent.length && html`<p>You lend nothing yet.</p>`}<ul>${s.lent.map((l) => html`<${Lent} key=${l.name} l=${l} signs=${s.signing.includes(l.name)} home=${home} />`)}</ul>`}</section>`;
}
export const view = { lent: ({ s, home }) => s.groups.length > 0 && html`<${Lending} round=${s.round} home=${home} told=${s.lentRead} />` };
