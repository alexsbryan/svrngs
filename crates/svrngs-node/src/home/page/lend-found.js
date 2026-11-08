// SPDX-License-Identifier: AGPL-3.0-or-later
// LEND, the machine of a program found running here (found.js `Found`): a
// kind svrngs knows is lent as it is, any other only once the person names
// it and says what it is. It is made to an explicit set of groups, never
// to everyone by default (SCOPE_PARALLEL.md, "Shared API shape"): with no
// group chosen the lend is refused, so joining a group later never widens
// it. A second press while one lends is refused. Once lent, HOME reads the
// groups again, so the index lists it and "What you lend" with it. Each
// refusal is said beside the button, after the ones before it, and kept
// there while it lends again.
import { assertNever, go, no } from '../ui/machine.js?v=__VERSION__';
import { post } from './routes.js?v=__VERSION__';
import { chosen } from './group-choice.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Shown } from './running-look.js?v=__VERSION__' */

/**
 * A program found's state: where its lend is, its port, its name and what
 * it is as typed, the stable ids of the groups chosen to open it, the
 * refusals said beside its button, and the lend made.
 * @typedef {object} State
 * @property {{ lend: 'unlent' | 'lending' | 'lent' }} at
 * @property {Record<string, number>} [asked]
 * @property {number} port
 * @property {string} name
 * @property {string} kind
 * @property {string} said
 * @property {string[]} groups
 * @property {Shown} [l]
 */

/**
 * The messages a program found takes: its name or what it is typed; a
 * group ticked or unticked, by its stable id; Lend pressed; it lent; HOME
 * told; or the lend refused, with why.
 * @typedef {{ type: 'typed', field: 'name' | 'kind', value: string }
 *   | { type: 'choose', id: string, on: boolean }
 *   | { type: 'lend' }
 *   | { type: 'lent', l: Shown, why: string }
 *   | { type: 'told', why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effects its step answers: the port lent, by the name and kind
 * typed, `null` where none is, to the groups chosen, by their stable ids;
 * and HOME told to read the groups again.
 * @typedef {{ run: 'lend', port: number, name: string | null, kind: string | null, groups: string[], failed: 'refused' }
 *   | { run: 'again', failed: 'told' }} Effect
 */

/**
 * @param {Home} home
 * @returns {Machine<State, Message, Effect>}
 */
export const LEND = (home) => ({
  states: { lend: ['unlent', 'lending', 'lent'] },
  io: {
    lend: ({ port, name, kind, groups }) =>
      post('/running', { port, name, kind, groups }).then((lend) => ({
        type: 'lent',
        l: lend,
        why: lend.name,
      })),
    again: () => {
      home({ type: 'again' });
      return { type: 'told', why: 'the groups read again' };
    },
  },
  step: (state, message) => {
    switch (message.type) {
      case 'typed':
        return go(state, {}, { [message.field]: message.value });
      case 'choose':
        return go(state, {}, { groups: chosen(state.groups, message), said: '' });
      case 'lend':
        if (state.at.lend === 'lending')
          return no(state, 'a lend is under way, and is answered first');
        if (state.groups.length === 0)
          return go(state, {}, { said: 'Choose which groups can open this first.' });
        return go(
          state,
          { lend: 'lending' },
          {},
          {
            run: 'lend',
            port: state.port,
            name: state.name.trim() || null,
            kind: state.kind.trim() || null,
            groups: state.groups,
            failed: 'refused',
          },
        );
      case 'lent':
        return go(state, { lend: 'lent' }, { l: message.l }, { run: 'again', failed: 'told' });
      case 'told':
        return go(state, {});
      case 'refused':
        return go(state, { lend: 'unlent' }, { said: `${state.said} ${message.why}` });
      default:
        return assertNever(state, message, 'a program found');
    }
  },
});
