// SPDX-License-Identifier: AGPL-3.0-or-later
// Each group held here that lends what `#open=` names, drawn in #door
// (enter.js `chosen`) when more than one does: an open each, in the group
// the person picks (among.js `pick`).
import { html } from '../ui/view.js?v=__VERSION__';
import { titled } from '../question.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Wants } from './enter.js?v=__VERSION__' */
/** @import { Props } from './page.js?v=__VERSION__' */

/**
 * @typedef {object} OfferedGroupsProps
 * @property {Group[]} groups each group that lends it
 * @property {Wants} wants what the address asks, opened in the one picked
 * @property {Props['send']} send
 */

/** @param {OfferedGroupsProps} props */
export function OfferedGroups({ groups, wants, send }) {
  return groups.map((group) => {
    const pick = () => send({ type: 'pick', group, wants });
    return html`
      <p>
        <button onClick=${pick}>Open it in ${titled(group)}</button>
      </p>
    `;
  });
}
