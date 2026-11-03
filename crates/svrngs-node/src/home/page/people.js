// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's people in its section, once GROUP has read them: the door
// (ui/door.js), each person's lends judged by GROUP's read of them. While
// that read is under way the door says "asking…", so it is given no
// lends; when they could not be read, `null`, which it says as not known
// now.
import { html } from '../ui/view.js?v=__VERSION__';
import { Door } from '../ui/door.js?v=__VERSION__';
/** @import { Lends, PlaceProps } from './group.js?v=__VERSION__' */

/**
 * The lends as the door is given them, by GROUP's read of them.
 * @param {PlaceProps['s']} state
 * @returns {Lends | null | undefined}
 */
const judged = (state) => {
  if (state.at.lends === 'read') return state.lends;
  if (state.at.lends === 'unread') return null;
  return undefined;
};

/** @param {PlaceProps} props */
export function People({ s: state }) {
  if (!state.people) return null;
  return html`<${Door} rows=${state.people.people} lends=${judged(state)} />`;
}
