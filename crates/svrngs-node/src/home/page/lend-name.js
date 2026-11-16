// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's cell in the group's index: its name, which opens it in a tab,
// under the caption that says what opening one does (index.js), so it reads
// as the app's way in and never its files (docs/APP_ACCESS_AND_STOP.md A1),
// with what only it says beside it (question.js `agreed`). A lend that
// opens friends' lends as you is a button, "Open" and its name: pressing it
// is the yes. A port does not open in a tab: it is named, then said to be
// this computer's own, or given the Play that opens another's here
// (play-cell.js).
import { html } from '../ui/view.js?v=__VERSION__';
import { agreed, label, may, named } from '../question.js?v=__VERSION__';
import { PlayCell } from './play-cell.js?v=__VERSION__';
import { Opening, Reach } from './reach.js?v=__VERSION__';
/** @import { Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */

/**
 * The lend, the group's shelf, and what its cell is drawn with.
 * @typedef {Pick<PlaceProps, 'g' | 's' | 'me'> & { l: Lent, shelf: Lends['shelf'] }} LendNameProps
 */

/** @param {LendNameProps} props */
export function LendName({ l: lend, shelf, g: group, s: state, me }) {
  if (lend.raw) {
    const how =
      lend.key === me ? 'A port this computer lends.' : html`<${PlayCell} l=${lend} s=${state} />`;
    return html`${named(lend)} ${how}`;
  }
  // A lend reading friends' lends opens by the form posting its yes; any
  // other through `/to` (home.rs), which gives its origin a code of its
  // own that becomes its session there.
  const to = `/to?${new URLSearchParams({ lend: label(lend) })}`;
  const words = `Open ${named(lend)}`;
  const plain =
    lend.reads && lend.reads.length > 0
      ? html`<${Reach} app=${label(lend)} words=${words} />`
      : html`<a href=${to} target="_blank">${named(lend)}</a>`;
  const opener = lend.reading
    ? html`<${Opening} app=${label(lend)} words=${words} reading=${lend.reading} kinds=${lend.reads ?? []} />`
    : plain;
  // What opening it lets it do is the grid's caption, never said again here.
  return html`${opener}${` ${agreed(lend, shelf, group, [may(lend, group)])}`}`;
}
