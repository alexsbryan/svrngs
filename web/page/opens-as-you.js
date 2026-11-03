// SPDX-License-Identifier: AGPL-3.0-or-later
// One lend that opens friends' things as you, under "What acts as you"
// (what-acts-as-you.js): the yes or stop kept with the group, the yes with
// its Stop. Once stopped here, it says when the stop holds: the pass the
// lend holds already lasts out its life.
import { html } from '../ui/view.js?v=__VERSION__';
import { Act } from '../ui/parts.js?v=__VERSION__';
import { named, reading, spoken } from '../question.js?v=__VERSION__';
import { PASS_LIFE } from './hand.js?v=__VERSION__';
/** @import { Said } from '../store.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Acted } from './acts.js?v=__VERSION__' */
/** @import { Props } from './page.js?v=__VERSION__' */

/**
 * @typedef {object} OpensAsYouProps
 * @property {Lent & { reads: string[] }} lend a lend that reads
 * @property {Said} said the yes or stop the group keeps for it
 * @property {Acted} act its Stop's answer
 * @property {Props['send']} send
 */

/** @param {OpensAsYouProps} props */
export function OpensAsYou({ lend, said, act, send }) {
  if (act.state === 'done') {
    const reads = lend.reads.map(spoken).join(' and ');
    const hours = PASS_LIFE / 3600;
    return html`<p>${named(lend)} stops opening your friends’ ${reads} as you within ${hours} hours, when the pass it holds ends.</p>`;
  }
  const stop = { type: 'stop reads', label: lend.label };
  const button =
    said.said === 'yes' && html`<${Act} words="Stop" doing=${act} on=${stop} send=${send} />`;
  return html`<p>${reading(lend, said)}${button}</p>`;
}
