// SPDX-License-Identifier: AGPL-3.0-or-later
// #removals: once the group is read, each removal and each ask to remove
// someone, or a line when they cannot be read, which the computer answers
// as `null`.
import { html } from '../ui/view.js?v=__VERSION__';
import { removal } from '../question.js?v=__VERSION__';
/** @import { Props } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function Removals({ s: state }) {
  if (!state.lent) return html`<ul id="removals"></ul>`;
  const unread = `Who was removed from ${state.group.name} cannot be read just now.`;
  const lines = (state.removals ?? [null]).map((gone) =>
    gone ? removal(gone, state.people) : unread,
  );
  return html`<ul id="removals">${lines.map((line) => html`<li>${line}</li>`)}</ul>`;
}
