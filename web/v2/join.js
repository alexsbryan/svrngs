// SPDX-License-Identifier: AGPL-3.0-or-later
// Joining by a link, the person's name asked once: page-join's, a stub
// until then (web/v2/page.js `compose`), its place as web/index.html:70-76
// draws it.
import { html } from '../ui/view.js?v=__VERSION__';

export const unbuilt = ['join'];
export const view = { join: () => html`<form id="join" hidden><p id="keep" hidden></p><label for="name">What’s your name? Everyone in <span id="group"></span> sees it.</label><input id="name" name="name" autocomplete="name" maxlength="64" required /><p id="why" class="bad"></p><button>Join</button></form>` };
