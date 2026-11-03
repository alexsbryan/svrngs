// SPDX-License-Identifier: AGPL-3.0-or-later
// #door: the choice of tab when the group is open in another (enter.js
// `chosen`), then, once the group is read, its name and whose it is, and
// its door (ui/door.js). The door is judged by the answer of the computer
// the page reached, so a computer that did not answer is drawn off.
import { html } from '../ui/view.js?v=__VERSION__';
import { Door } from '../ui/door.js?v=__VERSION__';
import { titled } from '../question.js?v=__VERSION__';
import { chosen } from './enter.js?v=__VERSION__';
/** @import { Props } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function GroupDoor({ s: state, send }) {
  const door =
    state.lent &&
    html`
      <p>${titled(state.group)}</p>
      <${Door} rows=${state.people} lends=${{ unanswered: state.unanswered }} />
    `;
  return html`<div id="door">${chosen(state, send)}${door}</div>`;
}
