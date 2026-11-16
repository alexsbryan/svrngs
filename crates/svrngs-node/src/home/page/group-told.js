// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's GROUP read, told to HOME (acts.js `take`) by its place in the
// group's section (group-read.js PLACES), once neither its people nor its lends
// are reading, so "What acts as you" (acting.js) draws from every group's
// read. `fresh` says both were read this round: a read that failed keeps
// what the one before answered, which the overview must not count as now's
// (grouped.js `Live`). It draws nothing.
import { useEffect } from '../ui/view.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */

/** @param {PlaceProps} props */
export function GroupTold({ g: group, s: state, home }) {
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
