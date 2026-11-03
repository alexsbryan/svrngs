// SPDX-License-Identifier: AGPL-3.0-or-later
// A guest's lends, first in #lent (lent-index.js) while the visit is on:
// an open each, named. Opening one asks visit.js for a pass (`call`), so
// the guest opens it with the visit's key, not a member's.
import { html } from '../ui/view.js?v=__VERSION__';
import { Row } from '../ui/parts.js?v=__VERSION__';
import { named } from '../question.js?v=__VERSION__';
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Props } from './page.js?v=__VERSION__' */

/**
 * @typedef {object} GuestLendsProps
 * @property {Lent[]} lent what the visit opens
 * @property {Props['send']} send
 */

/** @param {GuestLendsProps} props */
export function GuestLends({ lent, send }) {
  return lent.map((lend) => {
    const call = () => send({ type: 'call', label: lend.label });
    const open = html`<button onClick=${call}>${named(lend)}</button>`;
    return html`<${Row} key=${lend.label} cells=${[open]} />`;
  });
}
