// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's links in its section (group-links.js `Links`), once the
// group's people are read.
import { html } from '../ui/view.js?v=__VERSION__';
import { Links } from './group-links.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/** @param {PlaceProps} props */
function LinksPlace({ g: group, s: state }) {
  // Its people read, as the group's section reads them (group.js `Read`).
  if (!state.people) return false;
  return html`<${Links} g=${group} />`;
}

/** @type {{ links: (props: PlaceProps) => Drawn }} */
export const view = { links: LinksPlace };
