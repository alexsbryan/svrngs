// SPDX-License-Identifier: AGPL-3.0-or-later
// CHANGE, the machine of a lend's row in "What you lend" (lent-row.js
// `Lent`): how many it lets in at once, its person's choice, a number or
// everyone in their groups, set or cleared; signing people in with svrngs
// turned on; its audience, the groups that can open it, by their stable
// ids; or its lending stopped. The door takes a change at its next
// request. A second press while one change is under way is refused. A
// change is answered by HOME's `again`, the groups and the list read
// again, and the typed number and what was said are cleared with it. Why a
// change or the number is refused is said once, beside the buttons, the
// last in place of the one before.
import { assertNever, go, no } from '../ui/machine.js?v=__VERSION__';
import { post } from './routes.js?v=__VERSION__';
import { chosen } from './group-choice.js?v=__VERSION__';
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * The app form's fill, the one time signing people in is turned on
 * (home.rs `change`).
 * @typedef {{ issuer: string, client_id: string, client_secret: string | null, deviations: string }} Signed
 */

/**
 * A change of a lend: how many it lets in at once, a number or everyone
 * (`null`); its audience, the group ids that can open it; or its lending
 * stopped. An audience-only change preserves every other setting
 * (SCOPE_PARALLEL.md, `POST /lent`).
 * @typedef {{ limit: number | null } | { groups: string[] } | 'stop'} To
 */

/**
 * A lend's row's state: where its change is, the number typed, the group
 * ids chosen, what was last said, and the app form's fill, once.
 * @typedef {object} State
 * @property {{ change: 'idle' | 'changing' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} said
 * @property {string} [many]
 * @property {string[]} groups
 * @property {Signed} [shown]
 */

/**
 * The messages a row takes: a number typed, in the Field, which names no
 * field here; a group ticked or unticked, by its stable id; a change
 * pressed; the audience saved; Sign people in pressed; Let in this many
 * pressed, with the number; the change made, signing on, or refused, with
 * why.
 * @typedef {{ type: 'typed', value: string }
 *   | { type: 'choose', id: string, on: boolean }
 *   | { type: 'change', to: To }
 *   | { type: 'audience' }
 *   | { type: 'sign' }
 *   | { type: 'many', value: string }
 *   | { type: 'changed', why: string }
 *   | { type: 'signed', c: Signed, why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effects a row's step answers: the change posted, and signing on.
 * @typedef {{ run: 'change', to: To, failed: 'refused' }
 *   | { run: 'sign', failed: 'refused' }} Effect
 */

// The change asked, `effect` run, unless one is under way.
/**
 * @param {State} state
 * @param {Effect} effect
 * @returns {Step<State, Effect>}
 */
const changing = (state, effect) => {
  if (state.at.change === 'changing')
    return no(state, 'a change is under way, and is answered first');
  return go(state, { change: 'changing' }, {}, effect);
};

/**
 * @param {string} lend
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<State, Message, Effect>}
 */
export const CHANGE = (lend, home) => ({
  states: { change: ['idle', 'changing', 'refused'] },
  io: {
    change: ({ to }) =>
      post('/lent', { lend, to }).then(() => {
        home({ type: 'again' });
        return { type: 'changed', why: 'the groups read again' };
      }),
    sign: () =>
      post('/lent', { lend, to: 'sign-in' }).then((signIn) => ({
        type: 'signed',
        c: signIn,
        why: 'its secret shown here once',
      })),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'typed':
        return go(state, {}, { many: message.value });
      case 'choose':
        return go(state, {}, { groups: chosen(state.groups, message) });
      case 'audience':
        return changing(state, {
          run: 'change',
          to: { groups: state.groups },
          failed: 'refused',
        });
      case 'change':
        return changing(state, { run: 'change', to: message.to, failed: 'refused' });
      case 'sign':
        return changing(state, { run: 'sign', failed: 'refused' });
      case 'many':
        if (!/^[1-9][0-9]*$/.test(message.value))
          return go(state, { change: 'refused' }, { said: ' Say how many: 1 or more.' });
        return changing(state, {
          run: 'change',
          to: { limit: Number(message.value) },
          failed: 'refused',
        });
      case 'changed':
        return go(state, { change: 'idle' }, { many: undefined, said: '' });
      case 'signed':
        return go(state, { change: 'idle' }, { shown: message.c });
      case 'refused':
        return go(state, { change: 'refused' }, { said: ` ${message.why}` });
      default:
        return assertNever(state, message, 'a lend here');
    }
  },
});
