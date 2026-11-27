// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's index on the home page: what is in the group, on any port,
// drawn from GROUP's last read of its people and what they lend, so each
// read draws it again. What is shared grows without bound, so it is drawn
// as grids of cells, one grid under each sentence true of everything in it:
// where it runs and what opening one lets it do (question.js `may`, said
// short for many), and what Play does for a port another computer lends.
// The sentence names nothing, so the words
// outside the cells are the same for eight things and for eighty; a cell is
// a thing's name as its way in (lend-name.js), and a remark only where it
// differs. Then, in one line, the computers whose lends are not listed,
// said by why, so what a computer lends is never stale and never nobody's.
import { html } from '../ui/view.js?v=__VERSION__';
import { called, computers, lentBy, named, once } from '../question.js?v=__VERSION__';
import { LendName } from './lend-name.js?v=__VERSION__';
import { PLAYS } from './play.js?v=__VERSION__';
/** @import { Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

// Why a computer's lends are not listed, by how its read names it; `a|b`
// is the word for one computer, then for several.
const UNLISTED = {
  unanswered: 'did not answer, so what they lend is not listed',
  unheard: 'has|have not heard of yours yet, so what they lend is not listed yet',
  refused: 'answered without saying what they lend, so it is not listed',
  older: 'runs|run an older svrngs, so what they lend is not listed',
};

// The computers whose lends are not listed, in one line for them all: a
// sentence for each why, naming whose, every one (question.js `computers`),
// so a group with many asleep is not a line a computer.
/** @param {Lends} lends @param {PeopleRow[]} people */
const unsaid = (lends, people) =>
  Object.entries(UNLISTED)
    .flatMap(([how, why]) => {
      // Object.entries names each key a string; each is UNLISTED's, a list
      // of the lends' read.
      const whose = lends[/** @type {keyof typeof UNLISTED} */ (how)].map(
        (person) => called(people, person.person_key) ?? person.person,
      );
      const said = why.replace(/(\w+)\|(\w+)/, whose.length > 1 ? '$2' : '$1');
      return whose.length ? [`${computers(whose)} ${said}.`] : [];
    })
    .join(' ');

// Where a lend's grid comes: the group's own apps, then what people's
// computers lend that opens in a tab, the one that asks a yes last, since
// its cell takes a row of its own and a cell is read and tabbed in the
// order it is drawn; then their ports, this computer's own last, which has
// no Play.
/** @param {Lent} lend @param {string} me */
const place = (lend, me) => {
  if (lend.raw) return lend.key === me ? 3 : 2;
  if (lend.group) return 0;
  return (lend.reads ?? []).length > 0 ? 1.5 : 1;
};

// The sentences true of everything in a lend's grid, its caption: where it
// runs, so when it opens, an app the group carries in its folder on every
// computer (DESIGN §3, "Or a folder the group carries") and a lend on its
// lender's, so a person never learns the difference by one not opening;
// then what opening one lets it do, as you, the consent `may` says beside
// one open, said for many in a line read at a glance: a newcomer reads 20
// to 40 words a step, and the two captions were 71. "Each" says it of one
// and of many alike.
// Or what Play does; none for this computer's port.
const CARRIED =
  'On every computer in the group. Each opens as you, reading and writing only its own entries.';
const READING = 'On every computer in the group. Each opens as you, reading but never writing.';
const LENT =
  'On its lender’s computer, while it’s on. Each opens as you, and may write its own part of the record.';
/** @param {Lent} lend @param {string} me */
const says = (lend, me) => {
  if (place(lend, me) === 3) return '';
  if (lend.raw) return PLAYS;
  if (!lend.group) return LENT;
  return (lend.reads ?? []).length > 0 ? READING : CARRIED;
};

/**
 * A grid's caption, its lends, the caption's id, and what a cell is drawn with.
 * @typedef {Pick<PlaceProps, 'g' | 's' | 'me'> & { caption: string, lent: Lent[], by: string, shelf: Lends['shelf'] }} GridProps
 */

// A grid under its caption, a list named by it. `role="list"`: a list
// drawn with no markers is no list to VoiceOver without it. A lend's
// origin is its own, so it keys its cell.
/** @param {GridProps} props */
function Grid({ caption, lent, by, ...props }) {
  const id = caption ? by : undefined;
  const cells = lent.map(
    (lend) => html`<li key=${lend.origin}><${LendName} l=${lend} by=${id} ...${props} /></li>`,
  );
  return html`${caption && html`<p id=${by}>${caption}</p>`}<ul role="list" aria-labelledby=${id}>${cells}</ul>`;
}

/** @param {PlaceProps} props */
function Index({ g: group, s: state, me }) {
  const { lends } = state;
  if (!lends)
    return html`<div class="index"><p>Asking each computer in the group what it lends…</p></div>`;
  const people = state.people?.people ?? [];
  // One person's things sit together: a name says whose, "Kim’s navidrome".
  const lent = once(lends.lent.map((lend) => lentBy(lend, people, me))).sort(
    (one, other) => place(one, me) - place(other, me) || named(one).localeCompare(named(other)),
  );
  /** @type {Map<string, Lent[]>} */
  const grids = new Map();
  for (const lend of lent) {
    const caption = says(lend, me);
    grids.set(caption, [...(grids.get(caption) ?? []), lend]);
  }
  const under = unsaid(lends, people) || (lent.length ? '' : 'Nobody lends anything here yet.');
  return html`
    <div class="index">
      ${[...grids].map(
        ([caption, each], at) =>
          html`<${Grid} caption=${caption} lent=${each} by=${`says-${group.id}-${at}`} shelf=${lends.shelf} g=${group} s=${state} me=${me} />`,
      )}
      ${under && html`<p>${under}</p>`}
    </div>
  `;
}

/** @type {{ index: (props: PlaceProps) => Drawn }} */
export const view = { index: Index };
