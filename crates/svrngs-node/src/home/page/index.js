// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's index on the home page: one table of what is in the group,
// on any port, drawn from GROUP's last read of its people and what they
// lend, so each read draws it again (index-rows.js). Above it, once, what
// two or more of its rows would each say (`shared`).
import { html } from '../ui/view.js?v=__VERSION__';
import { Row } from '../ui/parts.js?v=__VERSION__';
import { lentBy, list, may, named, once } from '../question.js?v=__VERSION__';
import { IndexRows } from './index-rows.js?v=__VERSION__';
import { PLAYS } from './play.js?v=__VERSION__';
/** @import { Group, Lent, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

const HEADS = ['What', 'Whose'];

// Each sentence two or more of `lent` share, and the lends it is true of
// by name: what opening one lets it do (question.js `may`), and what Play
// does for a port another computer lends. Said per row, eight rows read as
// eight sentences of one shape, and the row that differs was the hardest to
// find. A sentence only one row says stays on its row.
/**
 * @param {Lent[]} lent
 * @param {Group} group
 * @param {string} me
 */
function shared(lent, group, me) {
  /** @type {Map<string, string[]>} */
  const said = new Map();
  // A port this computer lends has no Play here.
  for (const lend of lent.filter((one) => !one.raw || one.key !== me)) {
    const words = lend.raw ? PLAYS : may(lend, group);
    said.set(words, [...(said.get(words) ?? []), named(lend)]);
  }
  return [...said].filter(([, names]) => names.length > 1);
}

/** @param {PlaceProps} props */
function Index({ g: group, s: state, me }) {
  const people = state.people?.people ?? [];
  const lent = once((state.lends?.lent ?? []).map((lend) => lentBy(lend, people)));
  const above = shared(lent, group, me);
  const rows = state.lends
    ? html`
        <${IndexRows}
          lends=${state.lends}
          lent=${lent}
          above=${above.map(([words]) => words)}
          g=${group}
          s=${state}
          me=${me}
        />
      `
    : html`<${Row} cells=${['Asking each computer in the group what it lends…']} />`;
  // Each sentence names its lends where it said "it": "Opening Money or
  // Events lets it …". A function, so a `$` in a name is only a `$`.
  return html`
    ${above.map(([words, names]) => html`<p>${words.replace(' it ', () => ` ${list(names, 'or')} `)}</p>`)}
    <table class="index">
      <thead><${Row} tag="th" cells=${HEADS} /></thead>
      <tbody>${rows}</tbody>
    </table>
  `;
}

/** @type {{ index: (props: PlaceProps) => Drawn }} */
export const view = { index: Index };
