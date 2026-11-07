// SPDX-License-Identifier: AGPL-3.0-or-later
// JOIN, the machine of "Join a group from a link" (join-form.js
// `JoinForm`): a group joined from a link someone sent, by the svrngs
// running here as `svrngs join` joins. Each link typed is asked of the
// computer that sent it, what it is for, each ask counted, so an older
// answer is refused by name (`late`) and a slow answer never names a link
// typed over. The answer is said on the button before it is pressed, and
// the name box hidden when the link names whom it joins as. Pressed, a
// second press is refused while it joins; joined, HOME says so (`take`)
// and reads the groups again, and the form is empty again; a refusal is
// said below, in place of the last.
import { ask, assertNever, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * HOME's message this file takes: a group joined, by its name, as whom,
 * `null` when it was in already, and why its folder could not be made,
 * `null` when it was (home.rs `join`).
 * @typedef {{ type: 'joined', flow: 'join', q: { joined: string, person: string | null, unplaced: string | null } }} Taken
 */

/**
 * What a link is for, as the computer that sent it says (home.rs
 * `asked`): the group's name, and whom it joins as, `null` when the
 * person names themselves.
 * @typedef {[group: string, person: string | null]} For
 */

/**
 * The form's state: where its join is, the link and name typed, what the
 * link is for once asked, and what is said below.
 * @typedef {object} State
 * @property {{ join: 'typing' | 'checking' | 'checked' | 'refused' | 'joining' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} link
 * @property {string} person
 * @property {For} [for]
 * @property {string} why
 */

/**
 * The messages the form takes: a link or a name typed; ask `n` of the
 * link answered, or refused, with why; Join pressed; the group joined, or
 * not, with why.
 * @typedef {{ type: 'typed', field: 'link' | 'person', value: string }
 *   | { type: 'checked', n: number, for: For, why: string }
 *   | { type: 'refused', n: number, why: string }
 *   | { type: 'join' }
 *   | { type: 'joined', why: string }
 *   | { type: 'unjoined', why: string }} Message
 */

/**
 * The effects its step answers: ask `n` of what `link` is for, and the
 * group joined from it.
 * @typedef {{ run: 'check', link: string, n: number, failed: 'refused' }
 *   | { run: 'join', link: string, person: string, failed: 'unjoined' }} Effect
 */

// The form as the page draws it first, and again once a join is done.
/** @type {State} */
export const FIRST = { at: { join: 'typing' }, link: '', person: '', for: undefined, why: '' };

// Whom the join names: the link's own person, else the name typed.
/** @param {State} state */
export const who = (state) => state.for?.[1] ?? state.person.trim();

/**
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<State, Message, Effect>}
 */
export const JOIN = (home) => ({
  states: { join: ['typing', 'checking', 'checked', 'refused', 'joining'] },
  io: {
    check: ({ link, n: serial }) =>
      get(`/join?${new URLSearchParams({ link })}`).then((answer) => ({
        type: 'checked',
        n: serial,
        for: answer,
        why: answer[0],
      })),
    join: ({ link, person }) =>
      post('/join', { link, person }).then(([joined, joinedAs, unplaced]) => {
        home({ type: 'joined', flow: 'join', q: { joined, person: joinedAs, unplaced } });
        home({ type: 'again' });
        return { type: 'joined', why: 'said, the groups read again' };
      }),
  },
  // An answer to an ask older than the last is refused by name, before any
  // case takes it.
  step: (state, message) => {
    const old =
      message.type === 'checked' || message.type === 'refused'
        ? late(state, 'link', message.n)
        : '';
    if (old) return no(state, old);
    switch (message.type) {
      case 'typed': {
        if (message.field === 'person') return go(state, {}, { person: message.value });
        const [asked, serial] = ask(state, 'link');
        return go(
          state,
          { join: 'checking' },
          {
            link: message.value,
            asked,
            why: 'Asking the computer that sent it what it is for…',
          },
          { run: 'check', link: message.value, n: serial, failed: 'refused' },
        );
      }
      case 'checked':
        return go(state, { join: 'checked' }, { for: message.for, why: '' });
      case 'refused':
        return go(state, { join: 'refused' }, { for: undefined, why: message.why });
      case 'join':
        if (state.at.join === 'joining')
          return no(state, 'a join is under way, and is answered first');
        if (!state.for) return no(state, 'no link is asked of yet');
        return go(
          state,
          { join: 'joining' },
          {},
          { run: 'join', link: state.link, person: who(state), failed: 'unjoined' },
        );
      case 'joined':
        return go(state, FIRST.at, FIRST);
      case 'unjoined':
        return go(state, { join: 'refused' }, { why: message.why });
      default:
        return assertNever(state, message, 'joining');
    }
  },
});

/**
 * @param {HOME.State} state
 * @param {Taken} message
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state, message) =>
  go(state, { said: 'joined' }, { q: state.grouped ? { ...state.q, ...message.q } : message.q });
