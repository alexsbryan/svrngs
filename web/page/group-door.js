// SPDX-License-Identifier: AGPL-3.0-or-later
// #door: the groups to open `#open=` in, when more than one held here lends
// it (offered-groups.js), or the group open in another tab (one-tab.js);
// then, once the group is read, its name and whose it is, and its door
// (ui/door.js). The door is judged by the answer of the computer the page
// reached, so a computer that did not answer is drawn off. Each choice is
// drawn only while it is asked, so the first state draws the empty #door
// web/index.html has, as frame.test.mjs reads it.
import { html } from '../ui/view.js?v=__VERSION__';
import { Door } from '../ui/door.js?v=__VERSION__';
import { titled } from '../question.js?v=__VERSION__';
import { choosing } from './among.js?v=__VERSION__';
import { OfferedGroups } from './offered-groups.js?v=__VERSION__';
import { OneTab } from './one-tab.js?v=__VERSION__';
/** @import { Props } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function GroupDoor({ s: state, send }) {
  // `choosing` holds only while no tab here holds the group, so at most one
  // of the two is drawn.
  const offered =
    choosing(state) &&
    html`
      <${OfferedGroups}
        groups=${state.offered.groups}
        wants=${state.offered.wants}
        send=${send}
      />
    `;
  const elsewhere =
    state.at.tab === 'elsewhere' && html`<${OneTab} group=${state.group} send=${send} />`;
  const door =
    state.lent &&
    html`
      <p>${titled(state.group)}</p>
      <${Door} rows=${state.people} lends=${{ unanswered: state.unanswered }} />
    `;
  return html`<div id="door">${offered}${elsewhere}${door}</div>`;
}
