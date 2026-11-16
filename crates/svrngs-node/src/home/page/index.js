// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's index on the home page: what is in the group, on any port,
// drawn from GROUP's last read of its people and what they lend, so each
// read draws it again. What is shared grows without bound, so it is drawn
// as grids of cells, one grid under each sentence true of everything in it:
// what opening one lets it do (question.js `may`), and what Play does for a
// port another computer lends. The sentence names nothing, so the words
// outside the cells are the same for eight things and for eighty; a cell is
// a thing's name as its way in (lend-name.js), and a remark only where it
// differs. Then each computer whose lends are not listed, said by why, so
// what a computer lends is never stale and never nobody's.
import { html } from '../ui/view.js?v=__VERSION__';
import { called, label, lentBy, may, named, once, opens } from '../question.js?v=__VERSION__';
import { LendName } from './lend-name.js?v=__VERSION__';
import { PLAYS } from './play.js?v=__VERSION__';
/** @import { Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

const UNLISTED = {
  unanswered: 'did not answer, so what they lend is not listed',
  unheard: 'has not heard of yours yet, so what they lend is not listed yet',
  refused: 'answered without saying what they lend, so it is not listed',
  older: 'runs an older svrngs, so what they lend is not listed',
};

// Each computer whose lends are not listed, by why.
/** @param {Lends} lends @param {PeopleRow[]} people */
const unsaid = (lends, people) =>
  Object.entries(UNLISTED).flatMap(([how, why]) =>
    // Object.entries names each key a string; each is UNLISTED's, a list of
    // the lends' read.
    lends[/** @type {keyof typeof UNLISTED} */ (how)].map(
      (person) => `${called(people, person.person_key) ?? person.person}'s computer ${why}.`,
    ),
  );

// Where a lend's grid comes: the group's own apps, then what people's
// computers lend that opens in a tab, then their ports, this computer's
// own last, which has no Play.
/** @param {Lent} lend @param {string} me */
const place = (lend, me) => {
  if (!lend.raw) return lend.group ? 0 : 1;
  return lend.key === me ? 3 : 2;
};

// The sentence true of everything in a lend's grid, its caption: what
// opening one lets it do, or what Play does; none for this computer's port.
/** @param {Lent} lend @param {PlaceProps['g']} group @param {string} me */
const says = (lend, group, me) => {
  if (place(lend, me) === 3) return '';
  return lend.raw ? PLAYS : may(lend, group);
};

// A grid under its caption. Over several the caption says "one of these"
// where a lend alone is "it": "Opening one of these lets it …". A function,
// so a `$` is only a `$`.
/** @param {[caption: string, cells: Drawn[]]} grid */
const grid = ([caption, cells]) => html`
  ${caption && html`<p>${cells.length > 1 ? caption.replace(' it ', () => ' one of these ') : caption}</p>`}
  <ul>${cells}</ul>
`;

/** @param {PlaceProps} props */
function Index({ g: group, s: state, me }) {
  const { lends } = state;
  if (!lends)
    return html`<div class="index"><p>Asking each computer in the group what it lends…</p></div>`;
  const people = state.people?.people ?? [];
  // One person's things sit together: a name says whose, "Kim’s navidrome".
  const lent = once(lends.lent.map((lend) => lentBy(lend, people))).sort(
    (one, other) => place(one, me) - place(other, me) || named(one).localeCompare(named(other)),
  );
  /** @type {Map<string, Drawn[]>} */
  const grids = new Map();
  for (const lend of lent) {
    const caption = says(lend, group, me);
    // That a lend is on is the usual, so its cell says only that it is full.
    const cell = html`
      <li key=${label(lend)}>
        <${LendName} l=${lend} shelf=${lends.shelf} g=${group} s=${state} me=${me} />
        ${opens(lend) === 'full' && ' Full now.'}
      </li>
    `;
    grids.set(caption, [...(grids.get(caption) ?? []), cell]);
  }
  const unlisted = unsaid(lends, people);
  const none = lent.length || unlisted.length ? [] : ['Nobody lends anything here yet.'];
  return html`
    <div class="index">
      ${[...grids].map(grid)}
      ${[...unlisted, ...none].map((words) => html`<p>${words}</p>`)}
    </div>
  `;
}

/** @type {{ index: (props: PlaceProps) => Drawn }} */
export const view = { index: Index };
