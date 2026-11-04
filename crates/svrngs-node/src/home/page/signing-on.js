// SPDX-License-Identifier: AGPL-3.0-or-later
// What a lend's app form is filled with, the one time its lender turns
// signing people in with svrngs on: the issuer, the client and its secret,
// and the deviations in their words. The secret is shown here once and
// nowhere else, never in a request, a link or a trace line
// (SIGN_IN_WITH_SVRNGS §5 "Turning it on"): a read again keeps its row, so
// keeps it shown, and a load of the page, which reads none, shows it no
// more.
import { html } from '../ui/view.js?v=__VERSION__';
/** @import { Signed } from './lend-change.js?v=__VERSION__' */

/** @param {{ c: Signed }} props */
export function SigningOn({ c: fill }) {
  const after = `. ${fill.deviations} Copy the secret now: it is not shown again. `;
  return html`<p>Signing people in with svrngs. In the app’s own settings, choose OpenID Connect, and fill in: issuer <code>${fill.issuer}</code>, client id <code>${fill.client_id}</code>, client secret <code>${fill.client_secret}</code>${after}</p>`;
}
