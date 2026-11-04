// SPDX-License-Identifier: AGPL-3.0-or-later
// VISITS, the machine of a visit's Stop on the home page (stop-visit.js
// `Stop`): /visits stops the visit and answers its Stop's time. That time
// is after every act its computer holds, so it may be a second on
// (svrngs-core command.rs `sign_lines`); the machine waits for it before
// HOME reads the groups again (`again`), since a read sooner would still
// list the guest. Each refusal is said after the ones before it.
import { assertNever, go } from '../ui/machine.js?v=__VERSION__';
import { post } from './routes.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */

/**
 * A visit's state: where its Stop is, and each refusal said, in order.
 * @typedef {object} State
 * @property {{ visit: 'on' | 'stopping' | 'stopped' | 'waited' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string[]} said
 */

/**
 * The messages a visit takes: Stop pressed, for the group `g`'s visit
 * `id`; its Stop recorded at `ts`, in seconds; that time waited for; or
 * refused, with why.
 * @typedef {{ type: 'stop', g: string, id: string }
 *   | { type: 'stopped', ts: number, why: string }
 *   | { type: 'waited', why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effects its step answers: the Stop posted, and its time waited for.
 * @typedef {{ run: 'stop', g: string, id: string, failed: 'refused' }
 *   | { run: 'wait', ts: number, failed: 'refused' }} Effect
 */

/**
 * @param {Home} home
 * @returns {Machine<State, Message, Effect>}
 */
export const VISITS = (home) => ({
  states: { visit: ['on', 'stopping', 'stopped', 'waited', 'refused'] },
  step: (state, message) => {
    switch (message.type) {
      case 'stop':
        return go(
          state,
          { visit: 'stopping' },
          {},
          { run: 'stop', g: message.g, id: message.id, failed: 'refused' },
        );
      case 'stopped':
        return go(
          state,
          { visit: 'stopped' },
          {},
          { run: 'wait', ts: message.ts, failed: 'refused' },
        );
      case 'waited':
        return go(state, { visit: 'waited' });
      case 'refused':
        return go(state, { visit: 'refused' }, { said: [...state.said, ` ${message.why}`] });
      default:
        return assertNever(state, message, 'a visit');
    }
  },
  io: {
    stop: ({ g: group, id }) =>
      post('/visits', [group, id]).then((answer) => ({
        type: 'stopped',
        ts: answer.ts_unix,
        why: `its Stop is at ${answer.ts_unix}`,
      })),
    wait: ({ ts }) =>
      new Promise((resolve) => setTimeout(resolve, ts * 1000 - Date.now())).then(() => {
        home({ type: 'again' });
        return { type: 'waited', why: 'the groups are read again' };
      }),
  },
});
