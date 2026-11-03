// SPDX-License-Identifier: AGPL-3.0-or-later
// Who is visiting the group now (ui/visitors.js), with a line for you when
// a guest is yours, since you stop their visit on the home page of the
// computer that made their link, not here; or why it cannot be read.
import { html } from '../ui/view.js?v=__VERSION__';
import { Visiting } from '../ui/visitors.js?v=__VERSION__';
/** @import { Visitor } from '../ui/visitors.js?v=__VERSION__' */
/** @import { State } from './page.js?v=__VERSION__' */

/**
 * Whether a guest visiting is yours: by your first key, or by your name
 * when the computer that made their link names its host only so.
 * @param {State} state
 * @param {Visitor[]} visiting
 */
const yours = (state, visiting) => {
  const you = state.people.find((row) => row.key === state.me)?.person_key;
  return visiting.some((visit) =>
    visit.host_key ? visit.host_key === you : visit.host === state.group.person,
  );
};

/**
 * @typedef {object} VisitingNowProps
 * @property {State} state the page's, once the group is read
 */

/** @param {VisitingNowProps} props */
export function VisitingNow({ state }) {
  const visiting = state.visiting;
  if (!visiting) return html`<p>Who is visiting ${state.group.name} cannot be read just now.</p>`;
  if (visiting.length === 0) return null;
  const mine =
    yours(state, visiting) &&
    html`<p>You stop your guest’s visit on the home page of the computer you made their link on.</p>`;
  return html`
    <h2>Visiting now</h2>
    <${Visiting} vs=${visiting} rows=${state.people} />
    ${mine}
  `;
}
