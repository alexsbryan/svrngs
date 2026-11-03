// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's index on the home page (page-index (b); home.js@19b63b2b:118-119,
// :172-191; V1_UI_BUILD §3.1): one table of what is in the group, on any
// port, drawn from GROUP's last read of its people and what they lend, so
// each read draws it again. Each thing by its open, named (question.js
// `named`), which opens it in a tab, with what opening it agrees to beside
// it (`agreed`); whose; how it opens, a port by Play's cell (play.js); and
// whether it is full now. Each computer that did not say is a row of its
// own, said by how: what it lends never stale, and never nobody's.
import { agreed, called, html, named, once, opens, owner, Row } from '../ui/kit.js?v=__VERSION__';
import { cell } from './play.js?v=__VERSION__';
/** @import { Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PeopleRow as Person } from '../question.js?v=__VERSION__' */

// The lend's label, the first name of its origin here (home.js@19b63b2b:45).
// A split answers one part at least, so its first is there.
/**
 * @param {{ origin: string }} l
 * @returns {string}
 */
export const label = (l) => /** @type {string} */ (new URL(l.origin).hostname.split('.')[0]);
// A form posting what the person says of the app labelled `app` reaching
// friends' lends as them to /reaching (home.rs): its yes, which goes on to
// the app in a tab of its own (capability-open), or its Stop, which comes
// back here (capability-what-acts; home.js@19b63b2b:51).
/**
 * The form's props: the app's label, the button's words, and what it posts.
 * @typedef {{ app: string, words: Drawn, said?: 'yes' | 'stop' }} ReachProps
 */

/** @param {ReachProps} props */
export const Reach = ({ app, words, said = 'yes' }) =>
  html`<form method="post" action="/reaching" target=${said === 'yes' ? '_blank' : undefined}><input type="hidden" name="app" value=${app} /><input type="hidden" name="said" value=${said} /><button>${words}</button></form>`;
// A lend reading friends' lends opens by the form posting its yes, any
// other through `/to` (home.rs), which gives its origin a code of its own
// that becomes its session there (give-session; home.js@19b63b2b:58-61).
/** @param {Lent} l */
const opener = (l) =>
  l.reads?.length
    ? html`<${Reach} app=${label(l)} words=${named(l)} />`
    : html`<a href=${`/to?${new URLSearchParams({ lend: label(l) })}`} target="_blank">${named(l)}</a>`;
const UNLISTED = {
  unanswered: 'did not answer, so what they lend is not listed',
  unheard: 'has not heard of yours yet, so what they lend is not listed yet',
  refused: 'answered without saying what they lend, so it is not listed',
  older: 'runs an older svrngs, so what they lend is not listed',
};

// Each lend, whose by the key its computer joined with (question.js
// `apart`, never written into the read), then each computer unlisted.
// Drawn only once GROUP has read the lends (`view.index`), so `s.lends` is
// read.
/**
 * @param {PlaceProps} props
 * @param {Lends} [L]
 * @param {Person[]} [people]
 * @returns {Drawn[]}
 */
const rows = (
  { g, s, send, home, me },
  L = /** @type {Lends} */ (s.lends),
  people = s.people?.people ?? [],
) => {
  const lent = L.lent.map((l) => ({
    ...l,
    called: called(people, people.find((r) => r.key === l.key)?.person_key),
  }));
  const each = once(lent).map((l) =>
    ((opened, now) =>
      html`<tr key=${label(l)}>${l.raw ? html`<td>${named(l)}</td>` : html`<td>${opener(l)}${` ${agreed(l, L.shelf, g)}`}</td>`}<td>${owner(l, g)}</td><td>${l.raw ? (l.key === me ? 'A port this computer lends.' : html`<${cell} l=${l} g=${g} s=${s} send=${send} home=${home} />`) : `${opened}.`}</td><td>${now}</td></tr>`)(
      ...opens(l, 'In a tab'),
    ),
  );
  const said = Object.entries(UNLISTED).flatMap(([k, why]) =>
    // Object.entries names each key a string; each is UNLISTED's, a list of
    // the lends' read.
    L[/** @type {keyof typeof UNLISTED} */ (k)].map(
      (p) => `${called(people, p.person_key) ?? p.person}'s computer ${why}.`,
    ),
  );
  return [
    ...each,
    ...[...said, ...(each.length || said.length ? [] : ['Nobody lends anything here yet.'])].map(
      (w) => html`<${Row} cells=${[w]} span=${4} />`,
    ),
  ];
};
/** @type {{ index: (props: PlaceProps) => Drawn }} */
export const view = {
  index: (props) =>
    html`<table class="index"><thead><${Row} tag="th" cells=${['What', 'Whose', 'How it opens', 'Now']} /></thead><tbody>${props.s.lends ? rows(props) : html`<${Row} cells=${['Asking each computer in the group what it lends…']} />`}</tbody></table>`,
};
