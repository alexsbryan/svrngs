// SPDX-License-Identifier: AGPL-3.0-or-later
// A visit you host, with its Stop (visit.js VISITS). The line says what
// Stop does before the person presses it.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { until } from '../question.js?v=__VERSION__';
import { VISITS } from './visit.js?v=__VERSION__';
/** @import { Group, Home, Visit } from './group.js?v=__VERSION__' */

/**
 * A visit you host: the group, the visit, the group's name as said, and
 * HOME's `dispatch`, told to read again once it ends.
 * @typedef {object} StopProps
 * @property {Group} g
 * @property {Visit} v
 * @property {string} name
 * @property {Home} home
 */

/** @param {StopProps} props */
export function Stop({ g: group, v: visit, name, home }) {
  const [state, send] = useMachine(() =>
    machine('VISITS', VISITS(home), { at: { visit: 'on' }, said: [] }),
  );
  const guest = visit.guest;
  const words = `${guest}, until ${until(visit)}. Stop ends the visit now: what you lend stops opening to ${guest}, and everyone in ${name} sees it ended. `;
  const stop = () => send({ type: 'stop', g: group.id, id: visit.id });
  return html`
    <li>
      ${words}
      <button onClick=${stop}>${`Stop ${guest}’s visit`}</button>
      ${state.said}
    </li>
  `;
}
