// SPDX-License-Identifier: AGPL-3.0-or-later
// LINK, the machine of a group's links (group-links.js `Links`): the
// invitation, made by /publication and shown as its link, its short address
// and its QR code; and, as `svrngs link` makes them, a link to add a phone
// or another computer, or a guest's, whose length is asked first, 48 hours
// chosen to start and nothing longer offered (GUESTS §5). One per group, so
// a second press while its link is made is refused; the last link made is
// kept while the next is made, and a refusal shown in its place. A device
// link ends the invitation people are using to join (one open invitation
// a person and group, DESIGN §3), so while one is, its first press says so
// and its second makes it.
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
 * @property {boolean} [ending] a device link's first press said it ends the invitation
 * @property {boolean} [pressed] the invitation's last change was pressed for, so its answer takes the focus
 */

/** The native receipt as /publication shows it (publication.rs `shown`): while
 * it admits, the long link /link gives this page too, and its QR code; never
 * the API token.
 * @typedef {{ id: string, state: 'pending' | 'open' | 'closing' | 'closed' | 'ended', address: string | null, until: number, link?: string | null, qr?: { size: number, path: string } | null }} Publication
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

// The publication read again, after any change to it or a link made.
const REREAD = /** @type {Effect} */ ({ run: 'publicationRead', failed: 'publicationUnread' });
// A press for the invitation: why the last was refused, said no more.
const PRESSED = { publicationWhy: undefined, pressed: true };
// Whether a press waits: a link is being made, or the invitation read or changed.
export const busy = (/** @type {State} */ state) =>
  state.at.link === 'making' || ['working', 'reading'].includes(state.at.publication);
// The receipt /publication answered, read or changed, as the step takes it.
const published = (/** @type {Publication | null} */ publication) =>
  /** @type {Message} */ ({ type: 'published', publication });
// Whether the invitation lets people in now, so a new link would end it.
export const admits = (/** @type {Publication | null | undefined} */ receipt) =>
  ['open', 'pending'].includes(receipt?.state ?? '');

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
    publicationRead: () => get(`/publication?group=${encodeURIComponent(group)}`).then(published),
    publicationChange: ({ action, id }) =>
      post(action === 'close' ? '/publication/close' : '/publication', {
        group,
        ...(id && { id }),
      }).then(published),
    make: ({ asked, visit }) =>
      post('/link', { group, device: asked.device, ...(visit && { visit: Number(visit) }) }).then(
        (answer) => ({ type: 'made', a: answer, asked, why: answer.https ?? answer.client }),
      ),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'publication':
        if (busy(state)) return no(state, 'the invitation request is answered first');
        return message.action === 'read'
          ? go(state, { publication: 'reading' }, PRESSED, REREAD)
          : go(
              state,
              { publication: 'working' },
              { ...PRESSED, ending: undefined },
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
        return go(state, { publication: 'reading' }, { publicationWhy: message.why }, REREAD);
      case 'publicationUnread':
        return go(state, { publication: 'refused' }, { publicationWhy: message.why });
      case 'guest':
        return go(state, {}, { guest: true });
      case 'typed':
        return go(state, {}, { visit: message.value });
      case 'make':
        if (busy(state)) return no(state, 'a link is being made, and is answered first');
        if (message.device && !state.ending && admits(state.publication))
          return go(state, {}, { ending: true });
        return go(
          state,
          { link: 'making' },
          { ending: undefined, pressed: undefined },
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
          REREAD,
        );
      case 'refused':
        return go(state, { link: 'refused' }, { why: message.why });
      default:
        return assertNever(state, message, 'a link');
    }
  },
});
