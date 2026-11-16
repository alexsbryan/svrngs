// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's cell in the group's index: its name, which opens it in a tab,
// under the caption that says what opening one does (index.js), so it reads
// as the app's way in and never its files (docs/APP_ACCESS_AND_STOP.md A1),
// with what only it says beside it (question.js `agreed`). A lend that
// opens friends' lends as you is a button, "Open" and its name: pressing it
// is the yes. A port does not open in a tab: it is named, then said to be
// this computer's own, or given the Play that opens another's here
// (play.js `Turn`). Last in a cell that opens in a tab, "Address": what is for
// other devices and programs, in a popover (wifi.js `Address`).
import { html } from '../ui/view.js?v=__VERSION__';
import { agreed, label, may, named, opens } from '../question.js?v=__VERSION__';
import { Turn } from './play.js?v=__VERSION__';
import { Address } from './wifi.js?v=__VERSION__';
import { Opening, Reach } from './reach.js?v=__VERSION__';
/** @import { Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */

/**
 * The lend, the group's shelf, the id of its grid's caption, none for this
 * computer's own port, and what its cell is drawn with.
 * @typedef {Pick<PlaceProps, 'g' | 's'> & { l: Lent, shelf: Lends['shelf'], by?: string }} LendNameProps
 */

/** @param {LendNameProps} props */
export function LendName({ l: lend, shelf, by, g: group, s: state }) {
  const round = state.asked?.lends ?? 1;
  // Its way in has an id, `at-`, the place a person's popover links to
  // (people.js `Person`): the link goes to the way in, under its caption.
  const id = `${group.id}-${label(lend)}`;
  if (lend.raw) {
    const how = by
      ? html`<${Turn} l=${lend} round=${round} by=${by} id=${`at-${id}`} />`
      : 'A port this computer lends.';
    return html`${named(lend)} ${how}`;
  }
  // A lend reading friends' lends opens by the form posting its yes; any
  // other through `/to` (home.rs), which gives its origin a code of its
  // own that becomes its session there. Each way in is described by its
  // grid's caption and by what its own cell says (`aria-describedby`), so
  // someone who reaches it from a list of links, never having seen what is
  // above it, still hears what opening it lets it do.
  const said = `${by} said-${id}`;
  const to = `/to?${new URLSearchParams({ lend: label(lend) })}`;
  const words = `Open ${named(lend)}`;
  const plain =
    lend.reads && lend.reads.length > 0
      ? html`<${Reach} app=${label(lend)} words=${words} by=${said} id=${`at-${id}`} />`
      : html`<a id=${`at-${id}`} href=${to} target="_blank" aria-describedby=${said}>${named(lend)}</a>`;
  const opener = lend.reading
    ? html`<${Opening} app=${label(lend)} words=${words} reading=${lend.reading} kinds=${lend.reads ?? []} by=${said} id=${`at-${id}`} />`
    : plain;
  // What opening it lets it do is the grid's caption, never said again
  // here; that it is on is the usual, so only that it is full is said.
  const full = opens(lend) === 'full' ? ' Full now.' : '';
  return html`${opener}${' '}<span id=${`said-${id}`}>${agreed(lend, shelf, group, [may(lend, group)])}${full}</span>${' '}<${Address} l=${lend} id=${`address-${id}`} round=${round} />`;
}
