// SPDX-License-Identifier: AGPL-3.0-or-later
// One lend in the group's index: what it is (lend-name.js), whose, how it
// opens (how-it-opens.js), and whether it is full now.
import { html } from '../ui/view.js?v=__VERSION__';
import { opens, owner } from '../question.js?v=__VERSION__';
import { HowItOpens } from './how-it-opens.js?v=__VERSION__';
import { LendName } from './lend-name.js?v=__VERSION__';
/** @import { Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */

/**
 * The lend, the group's shelf, and what its cells are drawn with.
 * @typedef {Omit<PlaceProps, 'joined'> & { l: Lent, shelf: Lends['shelf'] }} LendRowProps
 */

/** @param {LendRowProps} props */
export function LendRow({ l: lend, shelf, g: group, s: state, send, home, me }) {
  const now = opens(lend);
  return html`
    <tr>
      <${LendName} l=${lend} shelf=${shelf} g=${group} />
      <td>${owner(lend, group)}</td>
      <td>
        <${HowItOpens}
          l=${lend}
          opened=${'In a tab'}
          g=${group}
          s=${state}
          send=${send}
          home=${home}
          me=${me}
        />
      </td>
      <td>${now}</td>
    </tr>
  `;
}
