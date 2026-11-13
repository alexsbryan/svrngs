// SPDX-License-Identifier: AGPL-3.0-or-later
// The foot of a group's section: each lend of the group's that is a page,
// with its line (wifi.js `Line`) and its address for a program. It is
// drawn once ROOM has answered, so no line says a lend is closed before
// ROOM has said what is open. This file, not wifi.js, names the place: it
// imports wifi.js, so wifi.js cannot import it back (`import/no-cycle`),
// and the law reads ROOM from wifi.js.
import { html, useEffect, useStore } from '../ui/view.js?v=__VERSION__';
import { lentBy, named, once } from '../question.js?v=__VERSION__';
import { keyOf } from './lend-key.js?v=__VERSION__';
import { Line, room } from './wifi.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/**
 * @param {PlaceProps} props
 * @returns {Drawn}
 */
function WifiFoot({ s: state }) {
  useEffect(
    () => room.dispatch({ type: 'read', round: state.asked?.lends ?? 1 }),
    [state.asked?.lends],
  );
  const roomState = useStore(room);
  if (!state.lends) return null;
  if (roomState.room === undefined && roomState.at.room !== 'unread') return null;
  const people = state.people?.people ?? [];
  const lines = once(state.lends.lent)
    .filter((lend) => !lend.raw)
    .map((lend) => {
      const whose = lentBy(lend, people);
      return html`<${Line} key=${keyOf(lend)} l=${whose} words=${`${named(whose)}:`} foot />`;
    });
  return html`<ul class="wifi">${lines}</ul>`;
}

/** @type {{ wifi: (props: PlaceProps) => Drawn }} */
export const view = { wifi: WifiFoot };
