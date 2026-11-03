// SPDX-License-Identifier: AGPL-3.0-or-later
// The group open in another tab of this browser, drawn in #door
// (group-door.js): one tab at a time holds the group's lock (enter.js
// `tab`), so taking it here (enter.js `take`) stops the other tab reaching
// the group.
import { html } from '../ui/view.js?v=__VERSION__';
import { Act } from '../ui/parts.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Props } from './page.js?v=__VERSION__' */

/**
 * @typedef {object} OneTabProps
 * @property {Group} group the group the other tab reaches
 * @property {Props['send']} send
 */

/** @param {OneTabProps} props */
export function OneTab({ group, send }) {
  return html`
    <p>
      One tab at a time reaches ${group.name}. Open it here, and the other tab stops reaching it.
    </p>
    <${Act} words="Open it here" on=${{ type: 'take' }} send=${send} />
  `;
}
