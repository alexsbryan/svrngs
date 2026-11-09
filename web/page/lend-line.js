// SPDX-License-Identifier: AGPL-3.0-or-later
// One line of the index of what the group lends (lent-index.js): the lend's
// open, named; what opening it agrees to (question.js `agreed`); " Full now."
// when it is; and, on a fine pointer, which is a computer's, the address
// to keep.
import { html } from '../ui/view.js?v=__VERSION__';
import { Keep } from '../ui/parts.js?v=__VERSION__';
import { agreed, alias, aliased, named, opens, readLine } from '../question.js?v=__VERSION__';
import { here, readingOf } from './hand.js?v=__VERSION__';
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Props, State } from './page.js?v=__VERSION__' */

/**
 * The address to keep for `lend` on this page, `here` (svrn-docs
 * rings/current/IN_THE_BROWSER.md §4): its alias, or, where two share one,
 * its label, which never collides. The alias is written as the page reads
 * it back, a space as `+` and `@` as itself.
 * @param {State} state the page's, once the group is read
 * @param {Lent} lend
 */
const kept = (state, lend) => {
  const lent = /** @type {Lent[]} */ (state.lent);
  const name = alias(lend, lent, state.people);
  const alone = aliased(name, lent, state.people).length === 1;
  const address = alone
    ? encodeURIComponent(name).replaceAll('%20', '+').replaceAll('%40', '@')
    : lend.label;
  return `${here()}#open=${address}`;
};

/**
 * @typedef {object} LendLineProps
 * @property {State} state the page's, once the group is read
 * @property {Lent} lend the lend this line opens
 * @property {Props['send']} send
 */

/** @param {LendLineProps} props */
export function LendLine({ state, lend, send }) {
  const full = opens(lend)[1] === 'full' ? ' Full now.' : '';
  const reads = lend.reads ?? [];
  const lent = /** @type {Lent[]} */ (state.lent);
  const reading = reads.length ? readingOf(lend, lent, state.group.reads?.[lend.label]) : undefined;
  const line = reading ? ` ${readLine(reading, reading.length, reads)}` : '';
  const agrees = `${line} ${agreed({ ...lend, reading }, state.shelf, state.group)}${full}`;
  const keep = state.fine && html`${' '}<${Keep} at=${kept(state, lend)} />`;
  return html`
    <span>
      <button onClick=${() => send({ type: 'open', label: lend.label })}>${named(lend)}</button>
      ${agrees}
      ${keep}
    </span>
  `;
}
