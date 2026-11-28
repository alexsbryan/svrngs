// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's cell in the group's index: its name, which opens it in a tab,
// under the caption that says what opening one does (index.js), so it reads
// as the app's way in and never its files (docs/APP_ACCESS_AND_STOP.md A1),
// with what only it says beside it (question.js `agreed`), and for an app
// the group carries, who brought it and when (`brought`): few of a group's
// things are its own apps, and a friend's name is how one is found. A lend
// that opens friends' lends as you is a button, "Open" and its name:
// pressing it is the yes. A port does not open in a tab, nor does a lend
// its lender says is for programs, such as a model a chat app uses
// (`for_programs`, a choice of what it is for, never read from its kind):
// it is named, then said to be this computer's own, or given the Play that
// opens another's here and copies its address (play.js `Turn`), and a
// lend's own Address after it. Last in a cell that opens in a tab, "Address": what is for
// other devices and programs, in a popover (wifi.js `Address`), with, for
// another computer's lend no folder carries, its Play, which lets a program
// here in at its address. So a cell has one way in, its name, drawn
// heaviest (look.css), and the rarer ways wait behind one press.
import { html } from '../ui/view.js?v=__VERSION__';
import {
  agreed,
  brought,
  label,
  may,
  named,
  notRunning,
  opens,
} from '../question.js?v=__VERSION__';
import { Turn } from './play.js?v=__VERSION__';
import { Address } from './wifi.js?v=__VERSION__';
import { Opening, Reach } from './reach.js?v=__VERSION__';
/** @import { Lent, PlaceProps } from './group.js?v=__VERSION__' */

/**
 * The lend, the id of its grid's caption, none for this computer's own
 * port, and what its cell is drawn with.
 * @typedef {Pick<PlaceProps, 'g' | 's' | 'me'> & { l: Lent, by?: string }} LendNameProps
 */

// A port, or a lend for programs: its name, then the Play that opens it
// for a program here and copies its address, else, this computer's own,
// said so; one for programs keeps its Address too.
/** @param {{ l: Lent, by?: string, id: string, round: number, address: unknown }} props */
function Played({ l: lend, by, id, round, address }) {
  const how = by
    ? html`<${Turn} l=${lend} round=${round} by=${by} id=${`at-${id}`} />`
    : 'A port this computer lends.';
  return html`${named(lend)} ${how}${!lend.raw && html` ${address}`}`;
}

/** @param {LendNameProps} props */
export function LendName({ l: lend, by, g: group, s: state, me }) {
  const round = state.asked?.lends ?? 1;
  // Its way in has an id, `at-`, the place a person's popover links to
  // (people.js `Person`): the link goes to the way in, under its caption.
  const id = `${group.id}-${label(lend)}`;
  const address = html`<${Address} l=${lend} id=${`address-${id}`} round=${round} me=${me} />`;
  if (lend.raw || lend.for_programs)
    return html`<${Played} l=${lend} by=${by} id=${id} round=${round} address=${address} />`;
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
      ? html`<${Reach} app=${label(lend)} words=${words} aria-describedby=${said} id=${`at-${id}`} />`
      : html`<a id=${`at-${id}`} href=${to} target="_blank" aria-describedby=${said}>${named(lend)}</a>`;
  const opener = lend.reading
    ? html`<${Opening} app=${label(lend)} words=${words} reading=${lend.reading} kinds=${lend.reads ?? []} by=${said} id=${`at-${id}`} />`
    : plain;
  // What opening it lets it do is the grid's caption, never said again
  // here; that it is on is the usual, so only that it is full, or that its
  // program is not running on its lender's computer, is said.
  const full = `${opens(lend) === 'full' ? ' Full now.' : ''}${notRunning(lend)}`;
  const bringer = brought(lend, state.people, me);
  return html`${opener}${' '}<span id=${`said-${id}`}>${agreed(lend, group, [may(lend, group)])}${bringer}${full}</span>${' '}${address}`;
}
