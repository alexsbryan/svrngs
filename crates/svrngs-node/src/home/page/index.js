// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's index on the home page: one table of what is in the group,
// on any port, drawn from GROUP's last read of its people and what they
// lend, so each read draws it again (index-rows.js). Above it, once, what
// two or more of its rows would each say (question.js `shared`).
import { html } from '../ui/view.js?v=__VERSION__';
import { Row } from '../ui/parts.js?v=__VERSION__';
import { lentBy, may, named, once, shared } from '../question.js?v=__VERSION__';
import { IndexRows } from './index-rows.js?v=__VERSION__';
import { PLAYS } from './play.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

const HEADS = ['What', 'Whose'];

/** @param {PlaceProps} props */
function Index({ g: group, s: state, me }) {
  const people = state.people?.people ?? [];
  const lent = once((state.lends?.lent ?? []).map((lend) => lentBy(lend, people)));
  // Each sentence two or more rows share, with the lends it is true of by
  // name: what opening one lets it do (question.js `may`), and what Play
  // does for a port another computer lends; this computer's own has no Play.
  // A sentence only one row says stays on its row.
  const above = shared(
    lent
      .filter((lend) => !lend.raw || lend.key !== me)
      .map((lend) => [lend.raw ? PLAYS : may(lend, group), named(lend)]),
    'or',
  ).filter(([, , rows]) => rows > 1);
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
    ${above.map(([words, names]) => html`<p>${words.replace(' it ', () => ` ${names} `)}</p>`)}
    <table class="index">
      <thead><${Row} tag="th" cells=${HEADS} /></thead>
      <tbody>${rows}</tbody>
    </table>
  `;
}

/** @type {{ index: (props: PlaceProps) => Drawn }} */
export const view = { index: Index };
