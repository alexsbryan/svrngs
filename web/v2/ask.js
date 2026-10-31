// SPDX-License-Identifier: AGPL-3.0-or-later
// The person asked whether an app may use a lend: page-acts', a stub until
// then (web/v2/page.js `compose`), its place as web/index.html:77-81 draws it.
import { html } from '../ui/view.js?v=__VERSION__';

export const unbuilt = ['ask'];
export const view = { ask: () => html`<form id="ask" hidden><blockquote id="words"></blockquote><button>Yes</button><button type="button" id="not">Not now</button></form>` };
