// SPDX-License-Identifier: AGPL-3.0-or-later
// The fold "What apps use" (#uses over #used), for acts.js's "Opens
// friends’ things as you", which draws it only where a lend asks, so the
// group's list is read. Its lines are ui/uses.js `Uses`, as `li`, each yes
// with Stop and a refusal said after it (uses.js).
import { html } from '../ui/view.js?v=__VERSION__';
import { Fold } from '../ui/parts.js?v=__VERSION__';
import { Uses } from '../ui/uses.js?v=__VERSION__';
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Props } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function WhatAppsUse({ s: state, send }) {
  const lines = Uses({
    lent: /** @type {Lent[]} */ (state.lent),
    labelOf: (/** @type {Lent} */ lend) => lend.label,
    choices: state.choices ?? {},
    tag: 'li',
    said: state.unstopped,
    send,
  });
  return html`
    <${Fold} id="uses" summary="What apps use" fold="uses" open=${state.using} send=${send}>
      <ul id="used">${lines}</ul>
    <//>
  `;
}
