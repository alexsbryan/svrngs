// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's first cell in the group's index: its name, which opens it in a
// tab, with what opening it agrees to beside it (question.js `agreed`). A
// port is named alone, since it does not open in a tab.
import { html } from '../ui/view.js?v=__VERSION__';
import { agreed, label, named } from '../question.js?v=__VERSION__';
import { Reach } from './reach.js?v=__VERSION__';
/** @import { Group, Lends, Lent } from './group.js?v=__VERSION__' */

/**
 * The lend, the group's shelf, which says what an app it carries agrees
 * to, and the group.
 * @typedef {{ l: Lent, shelf: Lends['shelf'], g: Group }} LendNameProps
 */

/** @param {LendNameProps} props */
export function LendName({ l: lend, shelf, g: group }) {
  if (lend.raw) return html`<td>${named(lend)}</td>`;
  // A lend reading friends' lends opens by the form posting its yes; any
  // other through `/to` (home.rs), which gives its origin a code of its
  // own that becomes its session there.
  const to = `/to?${new URLSearchParams({ lend: label(lend) })}`;
  const opener =
    lend.reads && lend.reads.length > 0
      ? html`<${Reach} app=${label(lend)} words=${named(lend)} />`
      : html`<a href=${to} target="_blank">${named(lend)}</a>`;
  return html`<td>${opener}${` ${agreed(lend, shelf, group)}`}</td>`;
}
