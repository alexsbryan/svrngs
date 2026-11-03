// SPDX-License-Identifier: AGPL-3.0-or-later
// RUNNING, the machine of "Already running here" (running-here.js
// `Running`): what runs on this computer and is not lent yet, as /running
// answers it. It looks only when the person presses Look, as often as they
// like, never as the page loads: the look knocks at every address here,
// seconds of this computer's work. Each look is counted, so an older
// look's answer is refused by name.
import { ask, assertNever, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { get } from './routes.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */

/**
 * How a kind svrngs's table knows signs each person in, and what its
 * lender does for it (share.rs `Signs`, serde's lowercase): it keeps its
 * own accounts, or takes the person from a header.
 * @typedef {{ accounts: string, header?: undefined } | { header: string, accounts?: undefined }} Signs
 */

/**
 * A program running here, as /running answers it (home.rs `shown`): its
 * port, the kind the table knows, its name once lent, how it signs each
 * person in, and the paths svrngs keeps back of it.
 * @typedef {object} Shown
 * @property {number} port
 * @property {string | null} kind
 * @property {string | null} name
 * @property {Signs | null} signs
 * @property {string[] | null} keeps
 */

/**
 * RUNNING's state: where its look is, what it found and the kinds the
 * table knows by name alone, or why it could not look.
 * @typedef {object} State
 * @property {{ running: 'unlooked' | 'looking' | 'looked' | 'unread' }} at
 * @property {Record<string, number>} [asked]
 * @property {Shown[]} [found]
 * @property {string[]} [named]
 * @property {string} [why]
 */

/**
 * The messages RUNNING takes: Look pressed; look `n` answered, or why not.
 * @typedef {{ type: 'look' }
 *   | { type: 'looked', n: number, found: Shown[], named: string[], why: string }
 *   | { type: 'unread', n: number, why: string }} Message
 */

/**
 * The effect its step answers: look `n`.
 * @typedef {{ run: 'look', n: number, failed: 'unread' }} Effect
 */

// What a lend needs first, as /running says from svrngs's table, in a
// sentence each: its own accounts, a setting, or what svrngs keeps back.
/** @param {Shown} shown */
export const needs = (shown) => {
  const accounts =
    shown.signs?.accounts &&
    `It keeps its own accounts, so make each person one in it: ${shown.signs.accounts}.`;
  const header =
    shown.signs?.header && `Before it knows who is who, set this in it: ${shown.signs.header}.`;
  const keeps =
    shown.keeps &&
    shown.keeps.length > 0 &&
    `svrngs keeps these parts of it from everyone: ${shown.keeps.join(', ')}.`;
  return [accounts, header, keeps].filter(Boolean).join(' ');
};

/** @type {Machine<State, Message, Effect>} */
export const RUNNING = {
  states: { running: ['unlooked', 'looking', 'looked', 'unread'] },
  io: {
    look: ({ n: serial }) =>
      get('/running').then(({ running: found, named }) => ({
        type: 'looked',
        n: serial,
        found,
        named,
        why: `${found.length} running`,
      })),
  },
  step: (state, message) => {
    const [asked, serial] = ask(state, 'running');
    const old = message.type === 'look' ? '' : late(state, 'running', message.n);
    if (old) return no(state, old);
    switch (message.type) {
      case 'look':
        return go(
          state,
          { running: 'looking' },
          { asked },
          { run: 'look', n: serial, failed: 'unread' },
        );
      case 'looked':
        return go(state, { running: 'looked' }, { found: message.found, named: message.named });
      case 'unread':
        return go(state, { running: 'unread' }, { why: message.why });
      default:
        return assertNever(state, message, 'what runs here');
    }
  },
};
