// SPDX-License-Identifier: AGPL-3.0-or-later
// LINK, the machine of a group's links (group-links.js `Links`), as
// `svrngs link` makes them: to invite someone, to add a phone or another
// computer, or a guest's, whose length is asked first, 48 hours chosen to
// start and nothing longer offered (GUESTS §5). One per group, so a
// second press while its link is made is refused; the last link made is
// kept while the next is made, and a refusal shown in its place.
import { assertNever, go, no } from '../ui/machine.js?v=__VERSION__';
import { post } from './routes.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */

/**
 * A press of one of the buttons: whether the link adds a device of yours,
 * and, for a guest's, how long the visit is, as said.
 * @typedef {{ type: 'make', device: boolean, guest?: true, hours?: string }} Make
 */

/**
 * The link made, as /link answers it (home.rs `link`): the client's link,
 * and the page's when this computer has one; and the press it answers.
 * @typedef {{ type: 'made', a: { client: string, https: string | null }, asked: Make, why: string }} Made
 */

/**
 * A group's links' state: where its link is, the guest's visit in seconds
 * as the Field holds it, whether a guest's link is offered, the last link
 * made, and why the last was refused.
 * @typedef {object} State
 * @property {{ link: 'none' | 'making' | 'made' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} visit
 * @property {boolean} [guest]
 * @property {Made} [shown]
 * @property {string} [why]
 */

/**
 * The messages a link takes: Have a guest pressed; the visit's length
 * chosen; a button pressed; its link made, or refused, with why.
 * @typedef {{ type: 'guest' }
 *   | { type: 'typed', field: 'visit', value: string }
 *   | Make
 *   | Made
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effect its step answers: a link made for the press `asked`, a
 * guest's for `visit` seconds.
 * @typedef {{ run: 'make', asked: Make, visit: string | undefined, failed: 'refused' }} Effect
 */

/**
 * LINK, for the group of id `group`.
 * @param {string} group
 * @returns {Machine<State, Message, Effect>}
 */
export const LINK = (group) => ({
  states: { link: ['none', 'making', 'made', 'refused'] },
  io: {
    make: ({ asked, visit }) =>
      post('/link', { group, device: asked.device, ...(visit && { visit: Number(visit) }) }).then(
        (answer) => ({ type: 'made', a: answer, asked, why: answer.https ?? answer.client }),
      ),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'guest':
        return go(state, {}, { guest: true });
      case 'typed':
        return go(state, {}, { visit: message.value });
      case 'make':
        if (state.at.link === 'making')
          return no(state, 'a link is being made, and is answered first');
        return go(
          state,
          { link: 'making' },
          {},
          {
            run: 'make',
            asked: message,
            visit: message.guest && state.visit,
            failed: 'refused',
          },
        );
      case 'made':
        return go(state, { link: 'made' }, { shown: message });
      case 'refused':
        return go(state, { link: 'refused' }, { why: message.why });
      default:
        return assertNever(state, message, 'a link');
    }
  },
});
