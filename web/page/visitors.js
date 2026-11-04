// SPDX-License-Identifier: AGPL-3.0-or-later
// #visitors: who is visiting now, once the group is read, and the wait for
// the first visit to end, so the group is read again as it ends and the
// visit drops off the page then.
import { html } from '../ui/view.js?v=__VERSION__';
import { Ends } from './ends.js?v=__VERSION__';
import { VisitingNow } from './visiting-now.js?v=__VERSION__';
/** @import { Props } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function Visitors({ s: state, send }) {
  // No one visiting, or who cannot be read: `undefined > 0` is false.
  const anyone = /** @type {number} */ (state.visiting?.length) > 0;
  return html`
    <section id="visitors">
      ${state.lent && html`<${VisitingNow} state=${state} />`}
      ${anyone && html`<${Ends} vs=${state.visiting} send=${send} />`}
    </section>
  `;
}
