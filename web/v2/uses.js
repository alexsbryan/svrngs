// SPDX-License-Identifier: AGPL-3.0-or-later
// What apps use: page-acts', a stub until then (web/v2/page.js `compose`), its
// place as web/index.html:97 draws it.
import { html } from '../ui/view.js?v=__VERSION__';

export const view = { uses: () => html`<details id="uses" hidden><summary>What apps use</summary><ul id="used"></ul></details>` };
