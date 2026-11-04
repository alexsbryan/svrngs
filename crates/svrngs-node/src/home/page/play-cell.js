// SPDX-License-Identifier: AGPL-3.0-or-later
// The index's cell for a port another computer lends (how-it-opens.js): its
// Play or Stop, asking PLAY again on each round of its group's reads, so
// PLAY reads once a round however many cells ask.
import { html, useEffect } from '../ui/view.js?v=__VERSION__';
import { Turn, play } from './play.js?v=__VERSION__';
/** @import * as GROUP from './group-reading.js?v=__VERSION__' */
/** @import { Port } from './play.js?v=__VERSION__' */

/**
 * The index's cell: the port, and its group's GROUP state, whose lends'
 * read's number is the round.
 * @typedef {{ l: Port, s: GROUP.State }} PlayCellProps
 */

/** @param {PlayCellProps} props */
export function PlayCell({ l: lend, s: state }) {
  useEffect(
    () => play.dispatch({ type: 'read', round: state.asked?.lends ?? 1 }),
    [state.asked?.lends],
  );
  return html`<${Turn} l=${lend} />`;
}
