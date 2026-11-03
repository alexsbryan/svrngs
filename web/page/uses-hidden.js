// SPDX-License-Identifier: AGPL-3.0-or-later
// #uses, hidden and empty, in its place in web/index.html's body while no
// lend asks, as frame.test.mjs reads the first state against it. Once a
// lend asks, acts.js draws the fold itself (what-apps-use.js), and this
// draws nothing.
import { html } from '../ui/view.js?v=__VERSION__';
/** @import { Props } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function UsesHidden({ s: state }) {
  if (state.lent?.some((lend) => lend.asks)) return false;
  return html`
    <details id="uses" hidden>
      <summary>What apps use</summary>
      <ul id="used"></ul>
    </details>
  `;
}
