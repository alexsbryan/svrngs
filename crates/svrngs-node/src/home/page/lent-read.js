// SPDX-License-Identifier: AGPL-3.0-or-later
// LENDING, the machine of "What you lend" (what-you-lend.js `Lending`):
// what this computer lends, read by /lent. An app a group carries is the
// group's, not this computer's to change, so it is not listed. Each read
// is counted, so an older answer is refused by name, and the last read
// stays drawn while the next is asked.
import { ask, assertNever, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { get } from './routes.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Lend } from '../question.js?v=__VERSION__' */

/**
 * The list's state: where its read is, what this computer lends, which of
 * them sign people in with svrngs, by name, and why it could not be read.
 * @typedef {object} State
 * @property {{ lent: 'reading' | 'read' | 'unread' }} at
 * @property {Record<string, number>} [asked]
 * @property {Lend[]} [lent]
 * @property {string[]} [signing]
 * @property {string} [why]
 */

/**
 * The messages the list takes: read again; read `n` answered (home.rs
 * `lent`), or unread, with why.
 * @typedef {{ type: 'read' }
 *   | { type: 'answered', n: number, lent: Lend[], signing: string[], why: string }
 *   | { type: 'unread', n: number, why: string }} Message
 */

/**
 * The effect its step answers: read `n` of what this computer lends.
 * @typedef {{ run: 'read', n: number, failed: 'unread' }} Effect
 */

/** @type {Machine<State, Message, Effect>} */
export const LENDING = {
  states: { lent: ['reading', 'read', 'unread'] },
  io: {
    read: ({ n: serial }) =>
      get('/lent').then(
        /** @param {{ lent: Lend[], signing: string[] }} answer */
        ({ lent, signing }) => ({
          type: 'answered',
          n: serial,
          lent: lent.filter((lend) => !lend.group),
          signing,
          why: `${lent.length} lent`,
        }),
      ),
  },
  // Each message's read numbered, and an answer older than the last read
  // asked refused by name, before any case takes it.
  step: (state, message) => {
    const [asked, serial] = ask(state, 'lent');
    const old = message.type === 'read' ? '' : late(state, 'lent', message.n);
    if (old) return no(state, old);
    switch (message.type) {
      case 'read':
        return go(
          state,
          { lent: 'reading' },
          { asked },
          { run: 'read', n: serial, failed: 'unread' },
        );
      case 'answered':
        return go(state, { lent: 'read' }, { lent: message.lent, signing: message.signing });
      case 'unread':
        return go(state, { lent: 'unread' }, { why: message.why });
      default:
        return assertNever(state, message, 'what you lend');
    }
  },
};
