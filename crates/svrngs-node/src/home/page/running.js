// SPDX-License-Identifier: AGPL-3.0-or-later
// "Already running here" on the home page (running-here.js), once the
// person is in a group.
import { html } from '../ui/view.js?v=__VERSION__';
import { Running } from './running-here.js?v=__VERSION__';
/** @import { Group } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/** @param {HOME.PlaceProps} props */
function RunningPlace({ s: state, home }) {
  // HOME draws its places once the groups are read (home.js `Home`).
  if (/** @type {Group[]} */ (state.groups).length === 0) return false;
  return html`<${Running} home=${home} />`;
}

/** @type {{ running: (props: HOME.PlaceProps) => Drawn }} */
export const view = { running: RunningPlace };
