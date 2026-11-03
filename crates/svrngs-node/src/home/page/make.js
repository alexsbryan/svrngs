// SPDX-License-Identifier: AGPL-3.0-or-later
// "Make a group" (page-join (d); home.js@19b63b2b:415-422): a group made here as
// `svrngs new` makes one, by the svrngs running here (start-group), from
// its name and the person's. Its machine, MAKE, is the place's own
// (ui/view.js `useMachine`). A name a group here has already is said below
// as it is typed (#138), each name asked of `GET /group` counted, so an
// older answer is refused by name (`late`), where v1 compared the typed
// value (home.js@19b63b2b:420). Pressed, a second press is refused while it makes; made,
// HOME says so, and again whose namesakes there are (`take`), and reads the
// groups again, where v1 opened the page again (home.js@19b63b2b:364; page-join (d)'s
// second fix), and the form is empty again; a refusal is said beside the
// button once, the last in place of the one before, where v1 appended each
// (home.js@19b63b2b:421; page-join (d)'s first fix).
import {
  Act,
  ask,
  assertNever,
  Field,
  go,
  html,
  late,
  machine,
  namesakes,
  no,
  useMachine,
} from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

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

/** @type {State} */
const FIRST = { at: { make: 'typing' }, name: '', person: '', before: '', said: '' };
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
      post('/group', [name, person]).then(
        ([made, unplaced, beside]) => (
          home({
            type: 'made',
            flow: 'make',
            q: { made, also: namesakes(beside, made, 'This is'), unplaced },
          }),
          home({ type: 'again' }),
          { type: 'made', why: 'said, the groups read again' }
        ),
      ),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'typed':
        return message.field === 'person'
          ? go(state, {}, { person: message.value })
          : ((asked, serial) =>
              go(
                state,
                { make: 'checking' },
                { name: message.value, asked },
                { run: 'check', name: message.value, n: serial, failed: 'checked' },
              ))(...ask(state, 'name'));
      case 'checked':
        return late(state, 'name', message.n)
          ? no(state, late(state, 'name', message.n))
          : go(
              state,
              { make: state.at.make === 'checking' ? 'checked' : state.at.make },
              { before: message.words ?? message.why },
            );
      case 'make':
        return state.at.make === 'making'
          ? no(state, 'a group is being made, and is answered first')
          : go(
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
export const take = (state, message) => go(state, { said: 'made' }, { q: message.q });

// Each field's name a field of the form's `State`, by which `state[name]` reads it.
/** @param {HOME.PlaceProps} props */
const Make = ({ home }) =>
  ((state, send) =>
    html`<section id="make"><h2>Make a group</h2><p>${
      /** @satisfies {['name' | 'person', string][]} */ ([
        ['name', 'A name for the group: '],
        ['person', 'Your name in it: '],
      ]).map(
        ([name, words]) =>
          html`<${Field} name=${name} words=${words} value=${state[name]} send=${send} />`,
      )
    }<${Act} words="Make a group" doing=${{ state: state.at.make === 'making' ? 'busy' : 'idle', said: state.said }} on=${{ type: 'make' }} send=${send} /></p><p>${state.before}</p></section>`)(
    ...useMachine(() => machine('MAKE', MAKE(home), FIRST)),
  );
/** @type {{ make: (props: HOME.PlaceProps) => Drawn }} */
export const view = { make: ({ home }) => html`<${Make} home=${home} />` };
