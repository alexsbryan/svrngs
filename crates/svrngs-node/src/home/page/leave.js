// SPDX-License-Identifier: AGPL-3.0-or-later
// Leaving a group, in its section on the home page (leave-group.js
// `Leave`), once the group's people are read; and HOME's `left`, the
// leave said and the groups read again (`take`).
import { html } from '../ui/view.js?v=__VERSION__';
import { ask, go, no } from '../ui/machine.js?v=__VERSION__';
import { Leave } from './leave-group.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Taken as Offered } from './offered.js?v=__VERSION__' */
/** @import { Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * HOME's message this file takes: a group left, by its name.
 * @typedef {{ type: 'left', flow: 'leave', name: string }} Taken
 */

// HOME's `left`: the leave said (home.js SAYS `left`), the groups read again.
// Any message to this flow is offered here (offered.js), so one of
// another type is refused.
/**
 * @param {HOME.State} state
 * @param {Offered} message
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state, message) => {
  if (message.type !== 'left') return no(state, `leaving takes no ${String(message.type)}`);
  const [asked, serial] = ask(state, 'groups');
  const left = /** @type {Taken} */ (message).name;
  return go(
    state,
    { groups: 'reading', said: 'left' },
    { asked, q: { ...state.q, left } },
    { run: 'groups', n: serial, round: 'again', failed: 'unread' },
  );
};

/** @param {PlaceProps} props */
function LeavePlace({ g: group, s: state, home, me }) {
  // Its people read, as the group's section reads them (group-read.js `GroupRead`).
  if (!state.people) return false;
  return html`<${Leave} g=${group} rows=${state.people.people} me=${me} home=${home} />`;
}

/** @type {{ leave: (props: PlaceProps) => Drawn }} */
export const view = { leave: LeavePlace };
