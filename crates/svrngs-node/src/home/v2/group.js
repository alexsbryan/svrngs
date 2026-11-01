// SPDX-License-Identifier: AGPL-3.0-or-later
// One group's section on the home page, and GROUP, its machine, one per
// group (page-open (d); home.js `section` and `show`, :100-114 and :116-239).
// GROUP reads the group's people and what they lend, each a region of its
// own, counted, so an older answer is refused by name; both are asked at
// once, and the people, answered first, are drawn without waiting for the
// lends (start-home: a group shown before its slowest computer). It reads
// again on HOME's `again` and `visible`, each a new `round` from HOME.
// What each read draws is each feature's, in its place (PLACES), from
// home/v2/<file>.js: a file that draws nothing draws nothing here.
import { html, useEffect, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { whose } from '../question.js?v=__VERSION__';

// A route's answer, or its words when it refuses, never read as empty
// (home.js:25-30); refused as the page's session lapsed (home.rs:198,
// session.rs `refused`), said as how to open it again.
// A message to a feature (`m.flow`, its file's name) is taken by its
// `take`, or refused by the file not built yet, as its `unbuilt` names it.
export const get = (path) => fetch(path).then(async (r) => (r.ok ? r.json() : Promise.reject(new Error(r.status === 401 ? 'this page’s session has ended. Open svrngs from your Applications, or on Linux your applications menu, and it opens here again' : (await r.json().catch(() => ({}))).error || `answered ${r.status}`)))),
  offered = (flows, s, m, f = flows[m.flow]) => (f?.unbuilt ? no(s, `not built yet: ${f.unbuilt}`) : f?.take ? f.take(s, m) : no(s, `nothing here takes ${m.type}`));

// Each place of a group's section, in v1's order (home.js:116-232): the
// index, its people, its folder, the links made here, leaving, removals,
// who is visiting, the shelf, what is open to this WiFi, and its link to a screen (`show`).
const PLACES = ['index', 'people', 'folder', 'links', 'leave', 'removals', 'visits', 'shelf', 'wifi', 'show'], ROUTES = { people: '/v1/rail/people', lends: '/v1/lends' };
// Both read, each counted (ui/machine.js `ask`), the round's why traced.
const reading = (s, m, [a, p] = ask(s, 'people'), [asked, l] = ask({ asked: a }, 'lends')) =>
  go(s, { people: 'reading', lends: 'reading' }, { asked }, ...[['people', p], ['lends', l]].map(([kind, n]) => ({ run: 'get', kind, n, id: s.id, failed: 'unread' })));
const GROUP = (flows) => ({
  states: { people: ['reading', 'read', 'unread'], lends: ['reading', 'read', 'unread'] },
  step: (s, m) => ({ read: reading, again: reading, visible: reading,
    answered: () => (late(s, m.kind, m.n) ? no(s, late(s, m.kind, m.n)) : go(s, { [m.kind]: 'read' }, { [m.kind]: m.body })),
    unread: () => (late(s, m.kind, m.n) ? no(s, late(s, m.kind, m.n)) : go(s, { [m.kind]: 'unread' }, { why: m.why })) })[m.type]?.(s, m) ?? offered(flows, s, m),
  io: { get: ({ kind, n, id }) => get(`${ROUTES[kind]}?namespace=${encodeURIComponent(id)}`).then((body) => ({ type: 'answered', kind, n, body, why: `${kind} ${n}` }), (e) => ({ type: 'unread', kind, n, why: e.message })) },
});

// The section, in its group's colour, derived from its id (look.css
// `data-colour`), headed by its name and whose it is (#138); who is in it
// unread, or the person removed, said and nothing read (home.js:113-114).
export const Group = ({ g, ...props }) => ((name) =>
  html`<section data-colour=${[...g.id].reduce((h, c) => (h * 31 + c.charCodeAt(0)) % 12, 0)}><h2>${g.name ?? 'A group whose name this computer cannot read'}${g.made && html`<small> (${whose(g.made)})</small>`}</h2>${g.removed === null ? html`<p>This computer could not read who is in ${name}.</p>` : g.removed ? html`<p>You are no longer in ${name}. What you already have stays on this computer; nothing new reaches it.</p>` : html`<${Read} g=${g} ...${props} />`}</section>`)(g.name ?? 'this group');
function Read({ g, round, places, flows, home, me, joined }) {
  const [s, send] = useMachine(() => machine('GROUP', GROUP(flows), { at: { people: 'reading', lends: 'reading' }, id: g.id }));
  useEffect(() => send({ type: round.type }), [round]);
  return html`${PLACES.map((k) => places[k] && html`<${places[k]} g=${g} s=${s} send=${send} home=${home} me=${me} joined=${joined} />`)}${(s.at.people === 'unread' || s.at.lends === 'unread') && html`<p>This computer cannot read what ${g.name ?? 'this group'} lends: ${s.why}</p>`}`;
}
