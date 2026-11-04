// SPDX-License-Identifier: AGPL-3.0-or-later
// LINK, the machine of a group's links (group-links.js `Links`), as
// `svrngs link` makes them: to invite someone, to add a phone or another
// computer, or a guest's, whose length is asked first, 48 hours chosen to
// start and nothing longer offered (GUESTS §5). One per group, so a
// second press while its link is made is refused; the last link made is
// kept while the next is made, and a refusal shown in its place.
import { assertNever, go, no } from '../ui/machine.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
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
 * @property {{ link: 'none' | 'making' | 'made' | 'refused', publication: 'reading' | 'ready' | 'working' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} visit
 * @property {boolean} [guest]
 * @property {Made} [shown]
 * @property {string} [why]
 * @property {Publication | null} [publication]
 * @property {string} [publicationWhy]
 */

/** The native receipt's public projection; no invitation secret or API token.
 * @typedef {{ id: string, state: 'pending' | 'open' | 'closing' | 'closed' | 'ended', address: string | null, until: number }} Publication
 */

/**
 * The messages a link takes: Have a guest pressed; the visit's length
 * chosen; a button pressed; its link made, or refused, with why.
 * @typedef {{ type: 'guest' }
 *   | { type: 'typed', field: 'visit', value: string }
 *   | Make
 *   | Made
 *   | { type: 'publication', action: 'read' | 'publish' | 'close' }
 *   | { type: 'published', publication: Publication | null }
 *   | { type: 'publicationFailed', why: string }
 *   | { type: 'publicationUnread', why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effect its step answers: a link made for the press `asked`, a
 * guest's for `visit` seconds.
 * @typedef {{ run: 'make', asked: Make, visit: string | undefined, failed: 'refused' }
 *   | { run: 'publicationRead', failed: 'publicationUnread' }
 *   | { run: 'publicationChange', action: 'publish' | 'close', id?: string, failed: 'publicationFailed' }} Effect
 */

/**
 * LINK, for the group of id `group`.
 * @param {string} group
 * @returns {Machine<State, Message, Effect>}
 */
export const LINK = (group) => ({
  states: {
    link: ['none', 'making', 'made', 'refused'],
    publication: ['reading', 'ready', 'working', 'refused'],
  },
  io: {
    publicationRead: () =>
      get(`/publication?group=${encodeURIComponent(group)}`).then((publication) => ({
        type: 'published',
        publication,
      })),
    publicationChange: ({ action, id }) =>
      post(action === 'close' ? '/publication/close' : '/publication', {
        group,
        ...(id && { id }),
      }).then((publication) => ({ type: 'published', publication })),
    make: ({ asked, visit }) =>
      post('/link', { group, device: asked.device, ...(visit && { visit: Number(visit) }) }).then(
        (answer) => ({ type: 'made', a: answer, asked, why: answer.https ?? answer.client }),
      ),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'publication':
        if (state.at.link === 'making' || ['working', 'reading'].includes(state.at.publication))
          return no(state, 'the invitation request is answered first');
        return message.action === 'read'
          ? go(
              state,
              { publication: 'reading' },
              { publicationWhy: undefined },
              { run: 'publicationRead', failed: 'publicationUnread' },
            )
          : go(
              state,
              { publication: 'working' },
              { publicationWhy: undefined },
              {
                run: 'publicationChange',
                action: message.action,
                id: state.publication?.id,
                failed: 'publicationFailed',
              },
            );
      case 'published':
        return go(state, { publication: 'ready' }, { publication: message.publication });
      case 'publicationFailed':
        // A failed mutation may have committed remotely. Read the persisted
        // local receipt, never mint again or guess whether joining ended.
        return go(
          state,
          { publication: 'reading' },
          { publicationWhy: message.why },
          { run: 'publicationRead', failed: 'publicationUnread' },
        );
      case 'publicationUnread':
        return go(state, { publication: 'refused' }, { publicationWhy: message.why });
      case 'guest':
        return go(state, {}, { guest: true });
      case 'typed':
        return go(state, {}, { visit: message.value });
      case 'make':
        if (state.at.link === 'making' || ['working', 'reading'].includes(state.at.publication))
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
        return go(
          state,
          { link: 'made', publication: 'reading' },
          { shown: message, publicationWhy: undefined },
          { run: 'publicationRead', failed: 'publicationUnread' },
        );
      case 'refused':
        return go(state, { link: 'refused' }, { why: message.why });
      default:
        return assertNever(state, message, 'a link');
    }
  },
});
