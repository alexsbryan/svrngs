// SPDX-License-Identifier: AGPL-3.0-or-later
// MAKE, the machine of "Make a group" (make-form.js `MakeForm`): a group
// made here as `svrngs new` makes one, by the svrngs running here, from
// its name and the person's. A name a group here has already is said below
// as it is typed (#138), each name asked of `GET /group` counted, so an
// older answer is refused by name (`late`) and a slow answer never speaks
// of a name typed over. Pressed, a second press is refused while it makes;
// made, HOME says so, and again whose namesakes there are (`take`), and
// reads the groups again, and the form is empty again; a refusal is said
// beside the button once, the last in place of the one before.
import { ask, assertNever, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { namesakes } from '../question.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * HOME's message this file takes: a group made, by its name, the groups
 * of its name said, and why its folder could not be made, `null` when it
 * was.
 * @typedef {{ type: 'made', flow: 'make', q: { made: string, also: string, unplaced: string | null } }} Taken
 */

/**
 * The form's state: where its make is, the names typed, what is said below
 * of a group of that name here already, and the last refusal.
 * @typedef {object} State
 * @property {{ make: 'typing' | 'checking' | 'checked' | 'making' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} name
 * @property {string} person
 * @property {string} before
 * @property {string} said
 */

/**
 * The messages the form takes: a name typed; read `n` of the group's name
 * checked, its namesakes' `words`, or, failed, why; Make pressed; the group
 * made; or refused, with why.
 * @typedef {{ type: 'typed', field: 'name' | 'person', value: string }
 *   | { type: 'checked', n: number, words?: string, why?: string }
 *   | { type: 'make' }
 *   | { type: 'made', why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effects its step answers: read `n` of who holds a group named
 * `name`, and the group made.
 * @typedef {{ run: 'check', name: string, n: number, failed: 'checked' }
 *   | { run: 'make', name: string, person: string, failed: 'refused' }} Effect
 */

// The form as the page draws it first, and again once a group is made.
/** @type {State} */
export const FIRST = { at: { make: 'typing' }, name: '', person: '', before: '', said: '' };

/**
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<State, Message, Effect>}
 */
export const MAKE = (home) => ({
  states: { make: ['typing', 'checking', 'checked', 'making', 'refused'] },
  io: {
    check: ({ name, n: serial }) =>
      get(`/group?${new URLSearchParams({ name })}`)
        .then(
          (found) => namesakes(found, name, 'This makes'),
          (error) => (name ? error.message : ''),
        )
        .then((words) => ({ type: 'checked', n: serial, words })),
    make: ({ name, person }) =>
      post('/group', [name, person]).then(([made, unplaced, beside]) => {
        home({
          type: 'made',
          flow: 'make',
          q: { made, also: namesakes(beside, made, 'This is'), unplaced },
        });
        home({ type: 'again' });
        return { type: 'made', why: 'said, the groups read again' };
      }),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'typed': {
        if (message.field === 'person') return go(state, {}, { person: message.value });
        const [asked, serial] = ask(state, 'name');
        return go(
          state,
          { make: 'checking' },
          { name: message.value, asked },
          { run: 'check', name: message.value, n: serial, failed: 'checked' },
        );
      }
      case 'checked': {
        const old = late(state, 'name', message.n);
        if (old) return no(state, old);
        const make = state.at.make === 'checking' ? 'checked' : state.at.make;
        return go(state, { make }, { before: message.words ?? message.why });
      }
      case 'make':
        if (state.at.make === 'making')
          return no(state, 'a group is being made, and is answered first');
        return go(
          state,
          { make: 'making' },
          {},
          { run: 'make', name: state.name, person: state.person, failed: 'refused' },
        );
      case 'made':
        return go(state, FIRST.at, FIRST);
      case 'refused':
        return go(state, { make: 'refused' }, { said: ` ${message.why}` });
      default:
        return assertNever(state, message, 'making a group');
    }
  },
});

/**
 * @param {HOME.State} state
 * @param {Taken} message
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state, message) =>
  go(state, { said: 'made' }, { q: { ...state.q, ...message.q } });
