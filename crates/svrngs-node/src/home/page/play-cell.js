// SPDX-License-Identifier: AGPL-3.0-or-later
// The index's Play or Stop for a port another computer lends
// (lend-name.js), asking PLAY again on each round of its group's reads,
// so PLAY reads once a round however many rows ask.
import { html, useEffect } from '../ui/view.js?v=__VERSION__';
import { Turn, play } from './play.js?v=__VERSION__';
/** @import * as GROUP from './group-reading.js?v=__VERSION__' */
/** @import { Port } from './play.js?v=__VERSION__' */

/**
 * The port, its group's GROUP state, whose lends' read's number is the
 * round, and `quiet` where what Play does is said above the index.
 * @typedef {{ l: Port, s: GROUP.State, quiet: boolean }} PlayCellProps
 */

/** @param {PlayCellProps} props */
export function PlayCell({ l: lend, s: state, quiet }) {
  useEffect(
    () => play.dispatch({ type: 'read', round: state.asked?.lends ?? 1 }),
    [state.asked?.lends],
  );
  return html`<${Turn} l=${lend} quiet=${quiet} />`;
}
