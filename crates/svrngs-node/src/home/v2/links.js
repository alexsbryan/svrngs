// SPDX-License-Identifier: AGPL-3.0-or-later
// A link made in a group's section (page-join (e); home.js:133-140), as
// `svrngs link` makes it (start-home): each button says what it makes, to
// invite someone, to add a phone or another computer, or a guest's, whose
// length is asked first, 48 hours chosen to start and nothing longer
// offered (guests-link; GUESTS §5). Its machine, LINK, is the place's own
// (ui/view.js `useMachine`), one per group, and a second press while one
// is made is refused. The link is then said in p.link, in its words, with
// Copy (ui/copy.js) beside it, its last answer alone, where h:137 appends
// each (page-join (e)'s fix); a refusal said there in its place.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { go, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { COPY, drawn } from '../ui/copy.js?v=__VERSION__';
import { post } from './running.js?v=__VERSION__';

const LINK = (group) => ({ states: { link: ['none', 'making', 'made', 'refused'] },
  io: { make: ({ asked, visit }) => post('/link', { group, device: asked.device, ...(visit && { visit: Number(visit) }) }).then((a) => ({ type: 'made', a, asked, why: a.https ?? a.client })) },
  step: (s, m) => ({ guest: () => go(s, {}, { guest: true }), typed: () => go(s, {}, { visit: m.value }),
    make: () => (s.at.link === 'making' ? no(s, 'a link is being made, and is answered first') : go(s, { link: 'making' }, {}, { run: 'make', asked: m, visit: m.guest && s.visit, failed: 'refused' })),
    made: () => go(s, { link: 'made' }, { shown: m }), refused: () => go(s, { link: 'refused' }, { why: m.why }) })[m.type]?.() ?? no(s, `a link takes no ${m.type}`) });
const Copy = (props) => ((s, send) => drawn({ ...s, said: s.said.slice(-1) }, send, props))(...useMachine(() => machine('COPY', COPY, { at: { copy: 'idle' }, said: [] })));
const WORDS = { copy: 'Copy the link', copied: ' Copied.', unwritten: ' Select it above to copy it.' }, LAST = ' It works for a day, while this computer is on. A new link stops this one letting anyone new in; everyone it let in stays. ';
// The link made, in the words of whom it is for (h:134, :139), kept while
// the next is made, as v1 keeps it until the answer (h:135).
const said = ({ a, asked }, name, at = a.https ?? a.client) => [
  ...(asked.guest ? ['Send this link to your guest, in any chat: ', html`<code>${at}</code>`, ` They open it in a browser and give their name. For ${asked.hours} from then they can open what you lend, and everyone in ${name} sees they are your guest. It lets one guest in, once, within a day, while this computer is on. `]
    : [asked.device ? 'Open this link on your other computer or phone; it joins as you, so send it to nobody else: ' : 'Send this link to the person you are inviting: ', html`<code>${at}</code>`, a.https ? ' It opens in a browser, which says how to join from there.' : ' They join with svrngs join and this link.', LAST]),
  html`<${Copy} key=${at} text=${at} words=${WORDS} beside=${true} />`];

function Links({ g }) {
  const [s, send] = useMachine(() => machine('LINK', LINK(g.id), { at: { link: 'none' }, visit: '172800' })), name = g.name ?? 'this group';
  const act = (words, on) => html`<${Act} words=${words} doing=${{ state: s.at.link === 'making' ? 'busy' : 'idle' }} on=${{ type: 'make', ...on }} send=${send} />`;
  return html`<p>${act(`Invite someone to ${name}`, { device: false })}${act('Add your phone or another computer', { device: true })}${s.guest ? html`<${Field} name="visit" words="For how long, from when they open the link: " value=${s.visit} options=${[48, 24, 6].map((h) => [h * 3600, `${h} hours`])} send=${send} />${act('Make a link for your guest', { device: false, guest: true, hours: `${s.visit / 3600} hours` })}` : html`<${Act} words="Have a guest" on=${{ type: 'guest' }} send=${send} />`}</p><p class="link">${s.at.link === 'refused' ? s.why : s.shown && said(s.shown, name)}</p>`;
}
// Drawn once the group's people are read, as v1 draws it (h:121-139).
export const view = { links: ({ g, s }) => s.people && html`<${Links} g=${g} />` };
