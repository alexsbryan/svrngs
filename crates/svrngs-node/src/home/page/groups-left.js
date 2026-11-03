// SPDX-License-Identifier: AGPL-3.0-or-later
// The groups you left, with what stays of each on this computer: HOME's
// place `left`, drawn once the groups are read (home-page.js `Home`), so
// `state.groups` is read then.
import { html } from '../ui/view.js?v=__VERSION__';
import { whose } from '../question.js?v=__VERSION__';
/** @import { Group as Held } from './group.js?v=__VERSION__' */
/** @import { PlaceProps } from './home-reading.js?v=__VERSION__' */

/** @param {PlaceProps} props */
export function GroupsLeft({ s: state }) {
  const gone = /** @type {Held[]} */ (state.groups).filter((group) => group.left);
  if (gone.length === 0) return false;
  const said = gone.map((group) => {
    const name = group.name ?? 'A group';
    const made = group.made ? ` (${whose(group.made)})` : '';
    const folders =
      group.folders && group.folders.length > 0
        ? `, its folder at ${group.folders.join(' and ')}`
        : '';
    return html`<p>${name}${made}: what you have of it stays on this computer${folders}, and nothing new of it reaches you.</p>`;
  });
  return html`
    <section id="left">
      <h2>Groups you left</h2>
      ${said}
    </section>
  `;
}
