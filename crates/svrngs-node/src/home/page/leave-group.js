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

/**
 * What a waiting hand-on says, to this computer's person or from them, and
 * its button.
 * @param {NonNullable<Group['offered']>} offer
 * @param {string} name
 * @returns {[words: string, button: string]}
 */
function waiting(offer, name) {
  const who = offer.person ?? 'someone';
  if (offer.to_you) {
    return [
      `${who} handed you the order of removals in ${name}. It is yours once you accept it: `,
      'Accept it',
    ];
  }
  const words = `You handed the order of removals in ${name} to ${who}. Nobody keeps it until they accept it on their computer, or you take it back: `;
  return [words, 'Take it back'];
}

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
  /**
   * One of the section's parts: its words, what else it asks, and its
   * button, each answer said beside it.
   * @param {string} words
   * @param {unknown} control
   * @param {[On, string, Body, string]} button what it is, where it posts, the body and its words
   * @param {string} [kind]
   */
  const part = (words, control, [on, path, body, label], kind) => html`
    <p class=${kind}>
      ${words}${control}
      <button onClick=${press(on, path, body)}>${label}</button>
      ${says(on)}
    </p>
  `;
  const keeps = `You keep the order of removals in ${name}. Choose who keeps it after you, then leave: `;
  const select = html`<select onChange=${choose}>${options}</select>`;
  const handOn =
    group.keeps &&
    others.length > 0 &&
    part(keeps, select, ['hand', '/v1/handovers', to, 'Hand it on']);
  // A hand-on waiting on its Accept: its recipient's yes, or its giver's taking it back.
  const at = { group: group.id };
  const offer = group.offered && waiting(group.offered, name);
  const taking = offer && part(offer[0], '', ['take', '/v1/accepts', at, offer[1]]);
  const vacates = group.vacates
    ? 'If you leave without handing it on, nobody keeps the order of removals until the others agree on who does. '
    : '';
  const stops = `${vacates}Leaving ${name} stops it reaching you, and stops what you lend reaching its people. What you have of it stays on this computer, its folder too. Rejoining needs a new invitation, and this computer can’t rejoin yet. `;
  /** @type {[On, string, Body, string]} */
  const leaving = ['leave', '/v1/leaves', at, `Leave ${name}`];
  // A group made before sources-v1, moved by whoever keeps it (removal.rs `adopt`).
  const MOVES = {
    move: `${name} keeps who decides removals the way svrngs first did. Move it, and handing that on waits until the other person accepts it. Every computer in ${name} needs the newest svrngs, and has to be on while it moves: `,
    phone: `You keep the order of removals in ${name} on your phone. To move ${name}, hand it to one of your computers first.`,
    waits: `This computer is still getting part of ${name}’s record, so it changes nobody in ${name} until then. Keep svrngs running; it carries on by itself.`,
  };
  const moving =
    group.moves === 'move'
      ? part(MOVES.move, '', ['move', '/v1/adopts', at, `Move ${name}`])
      : group.moves && html`<p>${MOVES[group.moves]}</p>`;
  return html`${moving} ${taking} ${handOn} ${part(stops, '', leaving, 'leave')}`;
}
