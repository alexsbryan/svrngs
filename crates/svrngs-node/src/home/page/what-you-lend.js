// SPDX-License-Identifier: AGPL-3.0-or-later
// "What you lend": what this computer lends (lent-read.js LENDING), each a
// row of its own (lent-row.js). It reads as it is drawn and again on each
// of HOME's rounds (`again`, `visible`), and tells HOME once it is first
// read, which holds the page's foot until then (lending.js `holds`).
import { html, useEffect, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { LENDING } from './lent-read.js?v=__VERSION__';
import { Lent } from './lent-row.js?v=__VERSION__';
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * The list, read again on each of HOME's rounds: the round, HOME's
 * `dispatch`, and whether HOME is told it was read.
 * @typedef {object} LendingProps
 * @property {HOME.State['round']} round
 * @property {HOME.PlaceProps['home']} home
 * @property {boolean | undefined} told
 */

/** @param {LendingProps} props */
export function Lending({ round, home, told }) {
  const [state, send] = useMachine(() => machine('LENDING', LENDING, { at: { lent: 'reading' } }));
  useEffect(() => {
    send({ type: 'read' });
  }, [round]);
  useEffect(() => {
    if (told || state.at.lent === 'reading') return;
    home({ type: 'told', flow: 'lending', why: `what you lend is ${state.at.lent}` });
  }, [state.at.lent]);
  // `signing` is set with `lent`, by `answered`, so it is there with it.
  const signing = /** @type {string[]} */ (state.signing);
  const rows = state.lent?.map(
    (lend) => html`
      <${Lent}
        key=${lend.name}
        l=${lend}
        signs=${signing.includes(lend.name)}
        home=${home}
      />
    `,
  );
  const list =
    state.lent &&
    html`
      ${!state.lent.length && html`<p>You lend nothing yet.</p>`}
      <ul>${rows}</ul>
    `;
  const body =
    state.at.lent === 'unread'
      ? html`<p>This computer cannot read what you lend: ${state.why}</p>`
      : list;
  return html`
    <section id="lent">
      <h2>What you lend</h2>
      ${body}
    </section>
  `;
}
