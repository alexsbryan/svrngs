// SPDX-License-Identifier: AGPL-3.0-or-later
// Bringing things to a group, in its section: the shelf (ui/shelf.js), once
// the group's lends are read, brings by /shelf, then has HOME read the
// groups again (`again`). It is folded for the group HOME says was just
// joined here: a link someone joins from opens the page at `?joined=`, given
// through home-page.js and group.js as `joined`.
import { html } from '../ui/view.js?v=__VERSION__';
import { Shelf } from '../ui/shelf.js?v=__VERSION__';
import { post } from './routes.js?v=__VERSION__';
/** @import { Group, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

// The apps ticked, by kind, brought to `group`; /shelf answers each.
/**
 * @param {Group} group
 * @param {string[]} apps
 */
const bring = (group, apps) => post('/shelf', [group.id, apps]);

/** @param {PlaceProps} props */
function GroupShelf({ g: group, s: state, home, joined }) {
  if (!state.lends) return null;
  return html`
    <${Shelf}
      g=${group}
      lent=${state.lends.lent}
      shelf=${state.lends.shelf}
      joined=${joined === group.name}
      bring=${bring}
      after=${() => home({ type: 'again' })}
    />
  `;
}

/** @type {{ shelf: (props: PlaceProps) => Drawn }} */
export const view = { shelf: GroupShelf };
