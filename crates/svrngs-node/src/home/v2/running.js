// SPDX-License-Identifier: AGPL-3.0-or-later
// "Already running here" (page-index (d); home.js `running`, :225-250):
// what runs on this computer and is not lent yet (/running, v1ui-lend
// (b)), looked for only when the person presses Look, as often as they
// like, never as the page loads: the look knocks at every address here,
// seconds of this computer's work (v1ui-lend-ask, the operator's "On
// request, with a button"). Its machine, RUNNING, is the place's own
// (ui/view.js `useMachine`), each look counted, so an older look's answer
// is refused by name. Each program found is a row with a machine of its
// own, LEND: a kind svrngs knows named with what it needs first, any other
// lent only once the person names it and says what it is; one button
// lends it, and the row then says what happened, or why not, each refusal
// beside the button as v1 appends it, kept there while it lends again. Drawn once the person is in a group.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';

// `body` posted as JSON to `path`, its answer, or its words when it refuses.
export const post = (path, body) => fetch(path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) }).then(async (r) => ((b) => (r.ok ? b : Promise.reject(new Error(b.error || `answered ${r.status}`))))(await r.json().catch(() => ({}))));
// What a lend needs first, as /running says from svrngs's table, in a
// sentence each: its own accounts, a setting, or what svrngs keeps back.
export const needs = (r) => [r.signs?.accounts && `It keeps its own accounts, so make each person one in it: ${r.signs.accounts}.`, r.signs?.header && `Before it knows who is who, set this in it: ${r.signs.header}.`, r.keeps?.length && `svrngs keeps these parts of it from everyone: ${r.keeps.join(', ')}.`].filter(Boolean).join(' ');

// Each look counted, an older look's answer refused by name.
const RUNNING = { states: { running: ['unlooked', 'looking', 'looked', 'unread'] }, io: { look: ({ n }) => get('/running').then(({ running: found, named }) => ({ type: 'looked', n, found, named, why: `${found.length} running` })) },
  step: (s, m, [asked, n] = ask(s, 'running'), old = m.type !== 'look' && late(s, 'running', m.n)) => (old ? no(s, old) : ({ look: () => go(s, { running: 'looking' }, { asked }, { run: 'look', n, failed: 'unread' }),
    looked: () => go(s, { running: 'looked' }, { found: m.found, named: m.named }), unread: () => go(s, { running: 'unread' }, { why: m.why }) })[m.type]?.() ?? no(s, `what runs here takes no ${m.type}`)) };
// A second press while one lends is refused; what it is called is typed.
// Once lent, HOME reads the groups again, so the index lists it and "What
// you lend" with it (page-index (d)'s fix), where v1 redrew only the last.
const LEND = (home) => ({ states: { lend: ['unlent', 'lending', 'lent'] }, io: { lend: ({ port, name, kind }) => post('/running', { port, name, kind }).then((l) => ({ type: 'lent', l, why: l.name })), again: () => (home({ type: 'again' }), { type: 'told', why: 'the groups read again' }) },
  step: (s, m) => ({ typed: () => go(s, {}, { [m.field]: m.value }), lend: () => (s.at.lend === 'lending' ? no(s, 'a lend is under way, and is answered first') : go(s, { lend: 'lending' }, {}, { run: 'lend', port: s.port, name: s.name.trim() || null, kind: s.kind.trim() || null, failed: 'refused' })),
    lent: () => go(s, { lend: 'lent' }, { l: m.l }, { run: 'again', failed: 'told' }), told: () => go(s, {}), refused: () => go(s, { lend: 'unlent' }, { said: `${s.said} ${m.why}` }) })[m.type]?.() ?? no(s, `a program found takes no ${m.type}`) });

function Found({ r, named, home }) {
  const [s, send] = useMachine(() => machine('LEND', LEND(home), { at: { lend: 'unlent' }, port: r.port, name: '', kind: '', said: '' })), at = `http://localhost:${r.port}`;
  if (s.at.lend === 'lent') return html`<li>${`You lent ${s.l.name}, at ${at}: everyone in your groups can open it now. ${needs(s.l)} `}</li>`;
  return html`<li>${r.kind ? `${r.kind}, at ${at}. ${needs(r)} ` : `Something at ${at}. To lend it, give it a name and say what it is, each one word${named.length ? `; if it is ${named.join(' or ')}, say that` : ''}. `}${!r.kind && [['name', 'Its name: '], ['kind', 'What it is: ']].map(([n, w]) => html`<${Field} name=${n} words=${w} value=${s[n]} send=${send} />`)}<${Act} words=${`Lend ${r.kind ?? 'it'} to everyone in your groups`} doing=${{ state: s.at.lend === 'lending' ? 'busy' : 'idle' }} on=${{ type: 'lend' }} send=${send} />${s.said}</li>`;
}
const SAID = { unlooked: () => 'This page lists the programs running on this computer only when you ask: looking takes it a few seconds of work.', looking: () => 'Looking for programs running on this computer…', unread: (s) => `You looked, and what runs on this computer cannot be read: ${s.why}`, looked: (s) => (s.found.length ? 'You looked just now: the programs running on this computer, each at its address. Lend one, and everyone in your groups opens it in their browser, as themselves, while this computer is on. You can stop lending it below at any time.' : 'You looked just now: nothing else on this computer is running at an address. Start a program, then look again.') };
function Running({ home }) {
  const [s, send] = useMachine(() => machine('RUNNING', RUNNING, { at: { running: 'unlooked' } }));
  // A look again empties the list, and each row it finds is a new one (h:237).
  return html`<section id="running"><h2>Already running here</h2><p>${SAID[s.at.running](s)}</p><p><${Act} words="Look for programs running on this computer" doing=${{ state: s.at.running === 'looking' ? 'busy' : 'idle' }} on=${{ type: 'look' }} send=${send} /></p><ul>${s.at.running === 'looked' && s.found.map((r) => html`<${Found} key=${`${s.asked.running}:${r.port}`} r=${r} named=${s.named} home=${home} />`)}</ul></section>`;
}
export const view = { running: ({ s, home }) => s.groups.length > 0 && html`<${Running} home=${home} />` };
