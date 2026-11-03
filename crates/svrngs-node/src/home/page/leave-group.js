// SPDX-License-Identifier: AGPL-3.0-or-later
// Leaving a group, in its section on the home page (leaving.js LEAVE):
// what stays and what stops, said before Leave is pressed; and, when this
// computer's person keeps the group's order of removals, who of its other
// people keeps it next, chosen beside it. The machine is the section's own
// (ui/view.js `useMachine`), so HOME's step never sees it.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { called } from '../question.js?v=__VERSION__';
import { LEAVE } from './leaving.js?v=__VERSION__';
/** @import { Group, Home } from './group.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */
/** @import { Body, On } from './leaving.js?v=__VERSION__' */

/**
 * A group's Leave, and its hand-on when this computer's person keeps its
 * order of removals: the group, its people, this computer's key and HOME's
 * `dispatch`.
 * @typedef {object} LeaveProps
 * @property {Group} g
 * @property {PeopleRow[]} rows
 * @property {string} me
 * @property {Home} home
 */

/** @param {LeaveProps} props */
export function Leave({ g: group, rows, me, home }) {
  const name = group.name ?? 'this group';
  const [state, send] = useMachine(() =>
    machine('LEAVE', LEAVE(home), { at: { leave: 'idle' }, said: [], to: 0, name }),
  );
  const own = rows.find((row) => row.key === me)?.person_key;
  // Each other person once, by a computer of theirs: a phone cannot keep
  // the order.
  const theirs = rows.filter((row) => row.device === 'computer' && row.person_key !== own);
  const others = [...new Map(theirs.map((row) => [row.person_key, row])).values()];
  /**
   * @param {On} on
   * @param {string} path
   * @param {Body} body
   */
  const press = (on, path, body) => () => send({ type: 'ask', on, path, body });
  /** @param {On} on */
  const says = (on) => state.said.filter((said) => said[0] === on).map((said) => said[1]);
  /** @param {{ target: HTMLSelectElement }} chosen */
  const choose = (chosen) => send({ type: 'to', i: chosen.target.value });
  const options = others.map(
    (row, i) => html`<option value=${i}>${called(rows, row.person_key)}</option>`,
  );
  // `state.to` is the chosen option's value, a string after the first
  // choice, by which an array is indexed as by its number.
  const to = { group: group.id, ...others[Number(state.to)] };
  const keeps = `You keep the order of removals in ${name}. Choose who keeps it after you, then leave: `;
  const handOn =
    group.keeps &&
    others.length > 0 &&
    html`
      <p>
        ${keeps}
        <select onChange=${choose}>
          ${options}
        </select>
        <button onClick=${press('hand', '/v1/handovers', to)}>Hand it on</button>
        ${says('hand')}
      </p>
    `;
  const stops = `Leaving ${name} stops it reaching you, and stops what you lend reaching its people. What you have of it stays on this computer, its folder too. Rejoining needs a new invitation, and this computer can’t rejoin yet. `;
  return html`
    ${handOn}
    <p class="leave">
      ${stops}
      <button onClick=${press('leave', '/v1/leaves', { group: group.id })}>Leave ${name}</button>
      ${says('leave')}
    </p>
  `;
}
