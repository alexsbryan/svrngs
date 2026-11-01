// SPDX-License-Identifier: AGPL-3.0-or-later
// The index of what the group lends, and opening one: page-index's, a stub
// until then (web/v2/page.js `compose`), its place as web/index.html:82 draws it.
import { html } from '../ui/view.js?v=__VERSION__';

export const unbuilt = ['open'];
export const view = { lent: () => html`<table class="index"><tbody id="lent"></tbody></table>` };
