// SPDX-License-Identifier: AGPL-3.0-or-later
// "Make a group" (page-join (d); home.js:415-422): a group made here as
// `svrngs new` makes one, by the svrngs running here (start-group), from
// its name and the person's. Its machine, MAKE, is the place's own
// (ui/view.js `useMachine`). A name a group here has already is said below
// as it is typed (#138), each name asked of `GET /group` counted, so an
// older answer is refused by name (`late`), where v1 compared the typed
// value (h:420). Pressed, a second press is refused while it makes; made,
// the page opens again saying so, and again whose namesakes there are, as
// v1 does (h:421); a refusal is said beside the button once, the last in
// place of the one before, where v1 appended each (h:421; page-join (d)'s fix).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { namesakes } from '../question.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';
import { post } from './running.js?v=__VERSION__';

const MAKE = { states: { make: ['typing', 'checking', 'checked', 'making', 'refused'] },
  io: { check: ({ name, n }) => get(`/group?${new URLSearchParams({ name })}`).then((b) => namesakes(b, name, 'This makes'), (e) => (name ? e.message : '')).then((words) => ({ type: 'checked', n, words })),
    make: ({ name, person }) => post('/group', [name, person]).then(([made, unplaced, beside]) => (location.assign(`/?${new URLSearchParams({ made, also: namesakes(beside, made, 'This is'), ...(unplaced && { unplaced }) })}`), { type: 'made', why: 'the page opened again' })) },
  step: (s, m) => ({
    typed: () => (m.field === 'person' ? go(s, {}, { person: m.value }) : ((asked, n) => go(s, { make: 'checking' }, { name: m.value, asked }, { run: 'check', name: m.value, n, failed: 'checked' }))(...ask(s, 'name'))),
    checked: () => (late(s, 'name', m.n) ? no(s, late(s, 'name', m.n)) : go(s, { make: s.at.make === 'checking' ? 'checked' : s.at.make }, { before: m.words ?? m.why })),
    make: () => (s.at.make === 'making' ? no(s, 'a group is being made, and is answered first') : go(s, { make: 'making' }, {}, { run: 'make', name: s.name, person: s.person, failed: 'refused' })),
    made: () => go(s, {}), refused: () => go(s, { make: 'refused' }, { said: ` ${m.why}` }) })[m.type]?.() ?? no(s, `making a group takes no ${m.type}`) };

function Make() {
  const [s, send] = useMachine(() => machine('MAKE', MAKE, { at: { make: 'typing' }, name: '', person: '', before: '', said: '' }));
  return html`<section id="make"><h2>Make a group</h2><p>${[['name', 'A name for the group: '], ['person', 'Your name in it: ']].map(([n, w]) => html`<${Field} name=${n} words=${w} value=${s[n]} send=${send} />`)}<${Act} words="Make a group" doing=${{ state: s.at.make === 'making' ? 'busy' : 'idle', said: s.said }} on=${{ type: 'make' }} send=${send} /></p><p>${s.before}</p></section>`;
}
export const view = { make: () => html`<${Make} />` };
