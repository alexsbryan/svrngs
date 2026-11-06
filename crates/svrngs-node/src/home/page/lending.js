// SPDX-License-Identifier: AGPL-3.0-or-later
// "What you lend" on the home page (what-you-lend.js), once the person is
// in a group, and what HOME takes of it: the page's foot, its address, is
// held until what you lend is first read, so the page does not look done
// while its list is empty. A driver takes the address as the page loaded
// (web/tests/home.mjs, `#home p.address`), and drawn sooner it read the
// list empty.
import { html } from '../ui/view.js?v=__VERSION__';
import { go } from '../ui/machine.js?v=__VERSION__';
import { Lending } from './what-you-lend.js?v=__VERSION__';
/** @import { Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Choice } from './group-choice.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * The field of HOME's that `take` sets: whether what you lend is read.
 * @typedef {{ lentRead?: boolean }} Fields
 */

/**
 * HOME's message this file takes: what you lend read.
 * @typedef {{ type: 'told', flow: 'lending', why: string }} Taken
 */

/** @typedef {NonNullable<HOME.State['groups']>} Groups */

/**
 * @param {HOME.State} state
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state) => go(state, {}, { lentRead: true });

// Asked only past the groups' read (home-page.js `Home`, `held`, past its first
// `return`), so `state.groups` is read.
/** @param {HOME.State} state */
export const holds = (state) => /** @type {Groups} */ (state.groups).length > 0 && !state.lentRead;

/** @param {HOME.PlaceProps} props */
function LendingPlace({ s: state, home }) {
  // HOME draws its places once the groups are read (home-page.js `Home`).
  const groups = /** @type {Choice[]} */ (state.groups);
  if (groups.length === 0) return false;
  return html`
    <${Lending} round=${state.round} groups=${groups} home=${home} told=${state.lentRead} />
  `;
}

/** @type {{ lent: (props: HOME.PlaceProps) => Drawn }} */
export const view = { lent: LendingPlace };
