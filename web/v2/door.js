// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's door, its removals and who is visiting: page-members', a stub
// until then (web/v2/page.js `compose`), its places as web/index.html:83, :85-86
// draw them; #door holds enter's choice of tab too (web/v2/enter.js `chosen`).
import { html } from '../ui/view.js?v=__VERSION__';
import { chosen } from './enter.js?v=__VERSION__';

export const view = { door: ({ s, send }) => html`<div id="door">${chosen(s, send)}</div>`, removals: () => html`<ul id="removals"></ul>`, visitors: () => html`<section id="visitors"></section>` };
