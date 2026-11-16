// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's first cell in the group's index: Open and its name, which opens
// it in a tab, with what opening it agrees to beside it (question.js
// `agreed`), so it reads as the app's way in and never its files
// (docs/APP_ACCESS_AND_STOP.md A1); what the index says once above its
// table (index.js `shared`) is not said again here. A port does not open in
// a tab: it is named, then said to be this computer's own, or given the Play
// that opens another's here (play-cell.js).
import { html } from '../ui/view.js?v=__VERSION__';
import { agreed, label, named } from '../question.js?v=__VERSION__';
import { PLAYS } from './play.js?v=__VERSION__';
import { PlayCell } from './play-cell.js?v=__VERSION__';
import { Opening, Reach } from './reach.js?v=__VERSION__';
/** @import { LendRowProps } from './lend-row.js?v=__VERSION__' */

/** @param {LendRowProps} props */
export function LendName({ l: lend, shelf, above, g: group, s: state, me }) {
  if (lend.raw) {
    const how =
      lend.key === me
        ? 'A port this computer lends.'
        : html`<${PlayCell} l=${lend} s=${state} quiet=${above.includes(PLAYS)} />`;
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
      : html`<a href=${to} target="_blank">${words}</a>`;
  const opener = lend.reading
    ? html`<${Opening} app=${label(lend)} words=${words} reading=${lend.reading} kinds=${lend.reads ?? []} />`
    : plain;
  return html`${opener}${` ${agreed(lend, shelf, group, above)}`}`;
}
