// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the home page (acting.js `Acting`), once the person
// is in a group; each group's read told to HOME (`GroupTold`); and
// HOME's part of it: what each group told (`take`), and the page's foot
// held until the section has read (`holds`).
import { useEffect } from '../ui/view.js?v=__VERSION__';
import { go } from '../ui/machine.js?v=__VERSION__';
import { Acting } from './acting.js?v=__VERSION__';
/** @import { Group, Lends, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */
/** @import { Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * The fields of HOME's that `take` sets: whether "What acts as you" is
 * read, and what each group's GROUP last read, by its id.
 * @typedef {object} Fields
 * @property {boolean} [actsRead]
 * @property {Record<string, Told>} [told]
 */

/**
 * HOME's messages this file takes: "What acts as you" read, or what a
 * group's GROUP read (`Told`).
 * @typedef {{ type: 'told', flow: 'acts', read: true, why: string }
 *   | { type: 'told', flow: 'acts', id: string, people: PeopleRow[] | undefined, lends: Lends | undefined, fresh: boolean, why: string, read?: undefined }} Taken
 */

/**
 * What a group's GROUP last read, as HOME keeps it (`Fields`' `told`), and
 * whether both were read in its last round (`fresh`).
 * @typedef {{ people: PeopleRow[] | undefined, lends: Lends | undefined, fresh: boolean }} Told
 */

// HOME's: what each group's GROUP last read, by its id.
/**
 * @param {HOME.State} state
 * @param {Taken} message
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state, message) => {
  if (message.read) return go(state, {}, { actsRead: true });
  const told = { people: message.people, lends: message.lends, fresh: message.fresh };
  return go(state, {}, { told: { ...state.told, [message.id]: told } });
};

// The page's foot held until the section has read (home-page.js `Home`).
// HOME asks once the groups are read (home-page.js `Home`).
/** @param {HOME.State} state */
export const holds = (state) => /** @type {Group[]} */ (state.groups).length > 0 && !state.actsRead;

// A group's GROUP read, told to HOME (acts.js `take`) by its place in the
// group's section (group-read.js PLACES), once neither its people nor its lends
// are reading, so "What acts as you" (acting.js) draws from every group's
// read. `fresh` says both were read this round: a read that failed keeps
// what the one before answered, which the overview must not count as now's
// (grouped.js `Live`). It draws nothing.
/** @param {PlaceProps} props */
function GroupTold({ g: group, s: state, home }) {
  useEffect(() => {
    if (state.at.people === 'reading' || state.at.lends === 'reading') return;
    home({
      type: 'told',
      flow: 'acts',
      id: group.id,
      people: state.people?.people,
      lends: state.lends,
      fresh: state.at.people === 'read' && state.at.lends === 'read',
      why: `${group.name}: people ${state.at.people}, lends ${state.at.lends}`,
    });
  }, [state.people, state.lends, state.at.people, state.at.lends]);
  return null;
}

// Drawn once the person is in a group (home-page.js `MEMBERS`).
/** @type {{ acts: (props: HOME.PlaceProps) => Drawn, told: (props: PlaceProps) => Drawn }} */
export const view = { acts: Acting, told: GroupTold };
