// SPDX-License-Identifier: AGPL-3.0-or-later
// LENDING, the machine of "What you lend" (what-you-lend.js `Lending`):
// what this computer lends, read by /lent. An app a group carries is the
// group's, not this computer's to change, so it is not listed. Each read
// is counted, so an older answer is refused by name, and the last read
// stays drawn while the next is asked.
import { reads } from '../ui/machine.js?v=__VERSION__';
import { get } from './routes.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Lend } from '../question.js?v=__VERSION__' */

/**
 * What home.rs `lent` says of a lend apps can read through an account:
 * whether one is kept, and how its lender makes one.
 * @typedef {{ kept: boolean, signs?: { accounts?: string } }} AppsAccount
 */

/**
 * The list's state: where its read is, what this computer lends, which of
 * them sign people in with svrngs, by name, which take an account apps read
 * them through, each with whether one is kept and how to make it, by name,
 * and why it could not be read.
 * @typedef {object} State
 * @property {{ lent: 'reading' | 'read' | 'unread' }} at
 * @property {Record<string, number>} [asked]
 * @property {Lend[]} [lent]
 * @property {string[]} [signing]
 * @property {Record<string, AppsAccount>} [readers]
 * @property {string} [why]
 */

/**
 * The messages the list takes: read again; read `n` answered (home.rs
 * `lent`), or unread, with why.
 * @typedef {{ type: 'read' }
 *   | { type: 'answered', n: number, lent: Lend[], signing: string[], readers: Record<string, AppsAccount>, why: string }
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
        /** @param {{ lent: Lend[], signing: string[], readers: Record<string, AppsAccount> }} answer */
        ({ lent, signing, readers }) => ({
          type: 'answered',
          n: serial,
          lent: lent.filter((lend) => !lend.group),
          signing,
          readers,
          why: `${lent.length} lent`,
        }),
      ),
  },
  step: reads({
    region: 'lent',
    start: 'read',
    reading: 'reading',
    done: 'answered',
    at: 'read',
    fields: ['lent', 'signing', 'readers'],
    taker: 'what you lend',
  }),
};
