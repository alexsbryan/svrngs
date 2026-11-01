// SPDX-License-Identifier: AGPL-3.0-or-later
// "Join a group from a link" (page-join (c); home.js:404-414): a group
// joined from a link someone sent, by the svrngs running here as `svrngs
// join` joins (arch-command (c)). Its machine, JOIN, is the place's own
// (ui/view.js `useMachine`). Each link typed is asked of the computer that
// sent it, what it is for, each ask counted, so an older answer is refused
// by name (`late`), where v1 compared the typed value (h:411); the answer
// is said on the button before it is pressed, and the name box hidden when
// the link names whom it joins as. Pressed, a second press is refused while
// it joins; joined, HOME says so (`take`) and reads the groups again, where
// v1 opened the page again (h:409; page-join (c)'s fix), and the form is
// empty again; a refusal is said below, in place of the last.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';
import { post } from './running.js?v=__VERSION__';

const FIRST = { at: { join: 'typing' }, link: '', person: '', for: undefined, why: '' }, who = (s) => s.for?.[1] ?? s.person.trim();
export const JOIN = (home) => ({ states: { join: ['typing', 'checking', 'checked', 'refused', 'joining'] },
  io: { check: ({ link, n }) => get(`/join?${new URLSearchParams({ link })}`).then((a) => ({ type: 'checked', n, for: a, why: a[0] })),
    join: ({ link, person }) => post('/join', { link, person }).then(([joined, person, unplaced]) => (home({ type: 'joined', flow: 'join', q: { joined, person, unplaced } }), home({ type: 'again' }), { type: 'joined', why: 'said, the groups read again' })) },
  step: (s, m, old = ['checked', 'refused'].includes(m.type) && late(s, 'link', m.n)) => (old ? no(s, old) : ({
    typed: () => (m.field === 'person' ? go(s, {}, { person: m.value }) : ((asked, n) => go(s, { join: 'checking' }, { link: m.value, asked, why: 'Asking the computer that sent it what it is for…' }, { run: 'check', link: m.value, n, failed: 'refused' }))(...ask(s, 'link'))),
    checked: () => go(s, { join: 'checked' }, { for: m.for, why: '' }), refused: () => go(s, { join: 'refused' }, { for: undefined, why: m.why }),
    join: () => (s.at.join === 'joining' ? no(s, 'a join is under way, and is answered first') : s.for ? go(s, { join: 'joining' }, {}, { run: 'join', link: s.link, person: who(s), failed: 'unjoined' }) : no(s, 'no link is asked of yet')),
    joined: () => go(s, FIRST.at, FIRST), unjoined: () => go(s, { join: 'refused' }, { why: m.why }) })[m.type]?.() ?? no(s, `joining takes no ${m.type}`)) });
export const take = (s, m) => go(s, { said: 'joined' }, { q: m.q });

function Join({ home }) {
  const [s, send] = useMachine(() => machine('JOIN', JOIN(home), FIRST));
  const typed = (e) => send({ type: 'typed', field: 'person', value: e.currentTarget.value });
  return html`<section id="join"><h2>Join a group from a link</h2><p><${Field} name="link" words="The link someone sent you: " value=${s.link} send=${send} /><label hidden=${Boolean(s.for?.[1])}>${'Your name in it: '}<input name="person" value=${s.person} onInput=${typed} /></label>${s.for && html`<${Act} words=${`Join ${s.for[0]}${who(s) ? ` as ${who(s)}` : ''}`} doing=${{ state: s.at.join === 'joining' ? 'busy' : 'idle' }} on=${{ type: 'join' }} send=${send} />`}</p><p>${s.why}</p></section>`;
}
export const view = { join: ({ home }) => html`<${Join} home=${home} />` };
