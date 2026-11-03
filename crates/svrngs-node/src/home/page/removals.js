// SPDX-License-Identifier: AGPL-3.0-or-later
// Who was removed from a group, or asked to be, in its section once its
// people are read, each said by question.js `removal`; nothing when no one
// was.
import { html } from '../ui/view.js?v=__VERSION__';
import { removal } from '../question.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Removal } from '../question.js?v=__VERSION__' */

/** @param {PlaceProps} props */
export function Removals({ g: group, s: state }) {
  if (!state.people) return null;
  // `removals` is `null` only beside a `removed` of `null`, and a group
  // whose `removed` is `null` draws no place (group.js `below`).
  const removals = /** @type {Removal[]} */ (group.removals);
  if (removals.length === 0) return null;
  const people = state.people.people;
  const said = removals.map((gone) => html`<li>${removal(gone, people)}</li>`);
  return html`<ul class="removals">${said}</ul>`;
}
