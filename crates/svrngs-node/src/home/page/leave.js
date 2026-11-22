// SPDX-License-Identifier: AGPL-3.0-or-later
// Leaving a group, in its section on the home page (leave-group.js
// `Leave`), once the group's people are read. Left, HOME says so (its
// `again`, with `?left=`; leaving.js TELLS).
import { html } from '../ui/view.js?v=__VERSION__';
import { Leave } from './leave-group.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/** @param {PlaceProps} props */
function LeavePlace({ g: group, s: state, home, me }) {
  // Its people read, as the group's section reads them (group-read.js `GroupRead`).
  if (!state.people) return false;
  return html`<${Leave} g=${group} rows=${state.people.people} me=${me} home=${home} />`;
}

/** @type {{ leave: (props: PlaceProps) => Drawn }} */
export const view = { leave: LeavePlace };
