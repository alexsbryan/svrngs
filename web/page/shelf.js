// SPDX-License-Identifier: AGPL-3.0-or-later
// Bringing things to the group (page-shelf (a); web/app.js@19b63b2b:482-489): the
// shelf (ui/shelf.js), drawn in #shelf once the group is read, and brought
// through the computer the page reaches (web/page/enter.js `through`); then
// the group read again (enter.js `reread`).
import { html, Shelf } from '../ui/kit.js?v=__VERSION__';
import { through } from './enter.js?v=__VERSION__';

const bring = (g, apps) => through(g, '/shelf', apps);
export const view = {
  shelf: ({ s, send }) =>
    html`<div id="shelf">${s.lent && s.shelf && html`<${Shelf} g=${s.group} lent=${s.lent} shelf=${s.shelf} joined=${s.joined} bring=${bring} after=${() => send({ type: 'reread' })} />`}</div>`,
};
