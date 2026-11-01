// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's door, its removals and who is visiting: page-members', a stub
// until then (web/v2/page.js `compose`), its places as web/index.html:83, :85-86
// draw them.
import { html } from '../ui/view.js?v=__VERSION__';

export const view = { door: () => html`<div id="door"></div>`, removals: () => html`<ul id="removals"></ul>`, visitors: () => html`<section id="visitors"></section>` };
