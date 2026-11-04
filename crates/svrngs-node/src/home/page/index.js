// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's index on the home page: one table of what is in the group,
// on any port, drawn from GROUP's last read of its people and what they
// lend, so each read draws it again (index-rows.js).
import { html } from '../ui/view.js?v=__VERSION__';
import { Row } from '../ui/parts.js?v=__VERSION__';
import { IndexRows } from './index-rows.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

const HEADS = ['What', 'Whose', 'How it opens', 'Now'];

/** @param {PlaceProps} props */
function Index({ g: group, s: state, send, home, me }) {
  const rows = state.lends
    ? html`
        <${IndexRows}
          lends=${state.lends}
          g=${group}
          s=${state}
          send=${send}
          home=${home}
          me=${me}
        />
      `
    : html`<${Row} cells=${['Asking each computer in the group what it lends…']} />`;
  return html`
    <table class="index">
      <thead><${Row} tag="th" cells=${HEADS} /></thead>
      <tbody>${rows}</tbody>
    </table>
  `;
}

/** @type {{ index: (props: PlaceProps) => Drawn }} */
export const view = { index: Index };
