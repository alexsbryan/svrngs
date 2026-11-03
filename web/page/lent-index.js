// SPDX-License-Identifier: AGPL-3.0-or-later
// #lent: the index of what the group lends, a line each (lend-line.js),
// drawn from the page's `lent`, so each read of the group draws it again.
// A guest's lends are drawn first, in the same table (visit.js `listed`).
import { html } from '../ui/view.js?v=__VERSION__';
import { Row } from '../ui/parts.js?v=__VERSION__';
import { once } from '../question.js?v=__VERSION__';
import { LendLine } from './lend-line.js?v=__VERSION__';
import { listed } from './visit.js?v=__VERSION__';
/** @import { Props } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function LentIndex({ s: state, send }) {
  const lines = once(state.lent ?? []).map((lend) => {
    const line = html`<${LendLine} state=${state} lend=${lend} send=${send} />`;
    return html`<${Row} key=${lend.label} cells=${[line]} />`;
  });
  return html`
    <table class="index">
      <tbody id="lent">
        ${listed(state, send)}
        ${lines}
      </tbody>
    </table>
  `;
}
