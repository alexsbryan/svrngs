// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the home page (acting.js `Acting`), once the person
// is in a group; each group's read told to HOME (group-told.js `GroupTold`); and
// HOME's part of it: what each group told (`take`), and the page's foot
// held until the section has read (`holds`).
import { html } from '../ui/view.js?v=__VERSION__';
import { go } from '../ui/machine.js?v=__VERSION__';
import { Acting } from './acting.js?v=__VERSION__';
import { GroupTold } from './group-told.js?v=__VERSION__';
/** @import { Group, Lends, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */
/** @import { Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * The fields of HOME's that `take` sets: whether "What acts as you" is
 * read, and what each group's GROUP last read, by its id.
 * @typedef {object} Fields
 * @property {boolean} [actsRead]
 * @property {Record<string, { people: PeopleRow[] | undefined, lends: Lends | undefined }>} [told]
 */

/**
 * HOME's messages this file takes: "What acts as you" read, or what a
 * group's GROUP read (`Told`).
 * @typedef {{ type: 'told', flow: 'acts', read: true, why: string }
 *   | { type: 'told', flow: 'acts', id: string, people: PeopleRow[] | undefined, lends: Lends | undefined, why: string, read?: undefined }} Taken
 */

/**
 * What a group's GROUP last read, as HOME keeps it (`Fields`' `told`).
 * @typedef {{ people: PeopleRow[] | undefined, lends: Lends | undefined }} Told
 */

// HOME's: what each group's GROUP last read, by its id.
/**
 * @param {HOME.State} state
 * @param {Taken} message
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state, message) => {
  if (message.read) return go(state, {}, { actsRead: true });
  const told = { people: message.people, lends: message.lends };
  return go(state, {}, { told: { ...state.told, [message.id]: told } });
};

// The page's foot held until the section has read (home.js `Home`).
// HOME asks once the groups are read (home.js `Home`).
/** @param {HOME.State} state */
export const holds = (state) => /** @type {Group[]} */ (state.groups).length > 0 && !state.actsRead;

/** @param {HOME.PlaceProps} props */
function ActsPlace({ s: state, home }) {
  // HOME draws its places once the groups are read (home.js `Home`).
  if (/** @type {Group[]} */ (state.groups).length === 0) return false;
  return html`<${Acting} s=${state} home=${home} />`;
}

/** @type {{ acts: (props: HOME.PlaceProps) => Drawn, told: (props: PlaceProps) => Drawn }} */
export const view = { acts: ActsPlace, told: GroupTold };
