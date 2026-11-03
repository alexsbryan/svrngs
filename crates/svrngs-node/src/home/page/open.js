// SPDX-License-Identifier: AGPL-3.0-or-later
// "Open a link from your group" on the home page (link-box.js), once the
// person is in a group.
import { html } from '../ui/view.js?v=__VERSION__';
import { Open } from './link-box.js?v=__VERSION__';
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PlaceProps } from '../home.js?v=__VERSION__' */

/** @param {PlaceProps} props */
function OpenPlace({ s: state }) {
  // HOME draws its places once the groups are read (home.js `Home`).
  const groups = /** @type {NonNullable<PlaceProps['s']['groups']>} */ (state.groups);
  if (groups.length === 0) return false;
  return html`<${Open} origin=${state.origin} />`;
}

/** @type {{ open: (props: PlaceProps) => Drawn }} */
export const view = { open: OpenPlace };
