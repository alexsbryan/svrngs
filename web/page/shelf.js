// SPDX-License-Identifier: AGPL-3.0-or-later
// Bringing things to the group (page-shelf (a); web/app.js@19b63b2b:482-489): the
// shelf (ui/shelf.js), drawn in #shelf once the group is read, and brought
// through the computer the page reaches (web/page/dial.js `through`); then
// the group read again (enter.js `reread`).
import { html } from '../ui/view.js?v=__VERSION__';
import { Shelf } from '../ui/shelf.js?v=__VERSION__';
import { through } from './dial.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Props } from './page.js?v=__VERSION__' */

/**
 * @param {Group} group
 * @param {unknown} apps what the shelf brings, as ui/shelf.js `Shelf` asks it
 */
const bring = (group, apps) => through(group, '/shelf', apps);
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  shelf: ({ s: state, send }) =>
    html`<div id="shelf">${state.lent && html`<${Shelf} g=${state.group} lent=${state.lent} shelf=${state.shelf} joined=${state.joined} bring=${bring} after=${() => send({ type: 'reread' })} />`}</div>`,
};
