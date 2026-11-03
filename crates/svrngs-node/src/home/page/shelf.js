// SPDX-License-Identifier: AGPL-3.0-or-later
// Bringing things to a group, in its section (page-shelf (a); home.js@19b63b2b:203):
// the shelf (ui/shelf.js), once the group's lends are read, brought by
// /shelf, then the groups read again (HOME's `again`). Folded for the group
// HOME says was just joined here, its `q.joined` given through home.js and
// group.js, as v1 reads `?joined=` (home.js@19b63b2b:203).
import { html, Shelf } from '../ui/kit.js?v=__VERSION__';
import { post } from './group.js?v=__VERSION__';
/** @import { Group, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

// The apps ticked, by kind, brought to group `g`; /shelf answers each.
/**
 * @param {Group} g
 * @param {string[]} apps
 */
const bring = (g, apps) => post('/shelf', [g.id, apps]);
/** @type {{ shelf: (props: PlaceProps) => Drawn }} */
export const view = {
  shelf: ({ g, s, home, joined }) =>
    s.lends &&
    html`<${Shelf} g=${g} lent=${s.lends.lent} shelf=${s.lends.shelf} joined=${joined === g.name} bring=${bring} after=${() => home({ type: 'again' })} />`,
};
