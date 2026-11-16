// SPDX-License-Identifier: AGPL-3.0-or-later
// One lend in the group's index: what it is (lend-name.js) and whose. That
// it is on and opens in a tab is the usual, so a row says only what differs:
// a port's Play (lend-name.js), and that it is full now.
import { html } from '../ui/view.js?v=__VERSION__';
import { opens, owner } from '../question.js?v=__VERSION__';
import { LendName } from './lend-name.js?v=__VERSION__';
/** @import { Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */

/**
 * The lend, the group's shelf, what the index says once above its table
 * (index.js `shared`), and what its cells are drawn with.
 * @typedef {Pick<PlaceProps, 'g' | 's' | 'me'> & { l: Lent, shelf: Lends['shelf'], above: string[] }} LendRowProps
 */

/** @param {LendRowProps} props */
export function LendRow({ l: lend, shelf, above, g: group, s: state, me }) {
  return html`
    <tr>
      <td>
        <${LendName} l=${lend} shelf=${shelf} above=${above} g=${group} s=${state} me=${me} />
        ${opens(lend) === 'full' && ' Full now.'}
      </td>
      <td>${owner(lend, group)}</td>
    </tr>
  `;
}
