// SPDX-License-Identifier: AGPL-3.0-or-later
// "Address" in a lend's cell of the index (lend-name.js): a button that
// opens the browser's own popover, holding what is for other devices and
// programs and is no part of deciding to open the lend: its line (wifi.js
// `Line`), with its WiFi button, and its address for a program with its
// Copy. What opening it to this WiFi does is said in the popover, over the
// button it is about, never out of sight of it. It was a fold at the
// group's foot, three lines a lend. It is drawn once ROOM has answered, so
// no line says a lend is closed before ROOM has said what is open, and it
// asks ROOM again on each round of its group's reads, once a round however
// many cells ask. This file, not wifi.js, draws it: lend-name.js's cell is
// the index's, and the law reads ROOM from wifi.js.
import { html, useEffect, useStore } from '../ui/view.js?v=__VERSION__';
import { named } from '../question.js?v=__VERSION__';
import { Line, room } from './wifi.js?v=__VERSION__';
/** @import { Lent } from './group.js?v=__VERSION__' */

const OPEN_IT = 'Open it to this WiFi, and anyone on it can open it as you while svrngs runs.';

/**
 * The lend, its popover's id, one of its own in the page, and the round of
 * its group's reads.
 * @typedef {{ l: Lent, id: string, round: number }} AddressProps
 */

/** @param {AddressProps} props */
export function Address({ l: lend, id, round }) {
  useEffect(() => room.dispatch({ type: 'read', round }), [round]);
  const roomState = useStore(room);
  if (roomState.room === undefined && roomState.at.room !== 'unread') return null;
  // `popover="auto"`: one open at a time, closed by Escape or a press
  // outside it, all the browser's doing.
  return html`
    <button type="button" popovertarget=${id}>Address</button>
    <div popover="auto" id=${id}>
      <p>${OPEN_IT}</p>
      <ul class="wifi"><${Line} l=${lend} words=${`${named(lend)}:`} foot /></ul>
    </div>
  `;
}
