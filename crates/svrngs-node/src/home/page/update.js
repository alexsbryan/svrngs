// SPDX-License-Identifier: AGPL-3.0-or-later
// UPDATE, the home page's own svrngs (update-line.js `Update`): on the
// page's first draw it asks /update whether the page this computer came
// from offers a svrngs it does not run, and when one is available its
// button posts /update. A new binary is in place only after the page's
// checksum matched it; the node then stops, and the one that starts in its
// place is the new binary. The page waits for it and reloads.
import { assertNever, go } from '../ui/machine.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */

/**
 * UPDATE's state: where it is, whether the person asked this time (a check
 * the page made as it opened says nothing when it cannot be judged), and
 * the last answer's why.
 * @typedef {object} State
 * @property {{ update: 'checking' | 'current' | 'available' | 'updating' | 'updated' | 'stuck' | 'unread' | 'refused' | 'dismissed' }} at
 * @property {boolean} [tried]
 * @property {string} [why]
 */

/**
 * The messages UPDATE takes: the page's first check; the person's retry,
 * update, or later; and each answer.
 * @typedef {{ type: 'check' | 'retry' | 'update' | 'later' }
 *   | { type: 'checked', state: string, why?: string }
 *   | { type: 'updated' }
 *   | { type: 'current' }
 *   | { type: 'unread' | 'refused' | 'stuck', why: string }
 *   | { type: 'back' }} Message
 */

/**
 * The effect its step answers: the page's answer, the update's, the wait
 * for the restarted svrngs, or the reload.
 * @typedef {{ run: 'check' | 'update' | 'wait' | 'reload', failed: 'unread' | 'refused' | 'stuck' }} Effect
 */

/** The effect the page runs as it opens. */
/** @type {Effect[]} */
export const START = [{ run: 'check', failed: 'unread' }];

/** @type {Machine<State, Message, Effect>} */
export const UPDATE = {
  states: {
    update: [
      'checking',
      'current',
      'available',
      'updating',
      'updated',
      'stuck',
      'unread',
      'refused',
      'dismissed',
    ],
  },
  step: (state, message) => {
    switch (message.type) {
      case 'check':
        return go(state, { update: 'checking' }, {}, { run: 'check', failed: 'unread' });
      case 'retry':
        return go(
          state,
          { update: 'checking' },
          { tried: true },
          { run: 'check', failed: 'unread' },
        );
      case 'checked':
        if (message.state === 'current') return go(state, { update: 'current' });
        if (message.state === 'available') return go(state, { update: 'available' });
        return go(state, { update: 'unread' }, { why: message.why });
      case 'unread':
        return go(state, { update: 'unread' }, { why: message.why });
      case 'update':
        return go(state, { update: 'updating' }, {}, { run: 'update', failed: 'refused' });
      case 'updated':
        return go(state, { update: 'updated' }, {}, { run: 'wait', failed: 'stuck' });
      case 'current':
        return go(state, { update: 'current' }, {}, { run: 'reload', failed: 'stuck' });
      case 'refused':
        return go(state, { update: 'refused' }, { why: message.why });
      case 'stuck':
        return go(state, { update: 'stuck' }, { why: message.why });
      case 'back':
        if (state.at.update === 'current') return go(state, {});
        return go(state, { update: 'current' }, {}, { run: 'reload', failed: 'stuck' });
      case 'later':
        return go(state, { update: 'dismissed' });
      default:
        return assertNever(state, message, 'an update');
    }
  },
  io: {
    check: () =>
      get('/update').then((/** @type {{ state: string, why?: string }} */ answer) => ({
        type: 'checked',
        state: answer.state,
        why: answer.why,
      })),
    update: () =>
      post('/update', {}).then((/** @type {{ state: string }} */ answer) =>
        answer.state === 'current' ? { type: 'current' } : { type: 'updated' },
      ),
    wait: async () => {
      for (let i = 0; i < 120; i++) {
        await new Promise((resolve) => setTimeout(resolve, 500));
        try {
          const response = await fetch('/svrngs-session-epoch', { cache: 'no-store' });
          if (response.ok) return { type: 'back' };
        } catch {
          // Not back yet; the next turn asks again.
        }
      }
      throw new Error('this page did not hear from svrngs again');
    },
    reload: () => {
      location.reload();
      return { type: 'back' };
    },
  },
};
