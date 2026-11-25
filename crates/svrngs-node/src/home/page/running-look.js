// SPDX-License-Identifier: AGPL-3.0-or-later
// RUNNING, the machine of "Already running here" (running-here.js
// `Running`): what runs on this computer and is not lent yet, as /running
// answers it. It looks only when the person presses Look, as often as they
// like, never as the page loads: the look knocks at every address here,
// seconds of this computer's work. Each look is counted, so an older
// look's answer is refused by name.
import { reads } from '../ui/machine.js?v=__VERSION__';
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
 * person in, and the paths svrngs keeps back of it; and, set by the look,
 * the program answering when the table knows that kind by name alone
 * (/running's `named`), for its fields.
 * @typedef {object} Shown
 * @property {number} port
 * @property {string | null} kind
 * @property {string | null} [program]
 * @property {string | null} name
 * @property {Signs | null} signs
 * @property {string[] | null} keeps
 * @property {string | null} [unknown]
 * @property {Answer | null} [answer] what answers on it, as svrngs read it
 *   (SHARE_BINDING §5.1): a program of this account's, or another account
 */

/**
 * What holds a port, as svrngs read it to show its lender (lend.rs
 * `Answer`): a program, by its process and start, another account, or a
 * systemd unit of this account's, by its name and description.
 * @typedef {{ pid?: number, start?: number, program?: string, folder?: string, account?: string, unit?: string, description?: string }} Answer
 */

// The job's description, or its name without the system's filename suffix.
// Old jobs carry a generated suffix in the description; it isn't a name.
/** @param {Answer} answer */
export const answered = ({ account, program, folder, unit, description }) => {
  if (unit)
    return description?.replace(/, kept by svrngs-keeper$/, '') ?? unit.replace(/\.service$/, '');
  return account
    ? `a program of the account ${account}`
    : `${program}${folder ? ` in ${folder}` : ''}`;
};

/**
 * RUNNING's state: where its look is, what it found, or why it could not
 * look.
 * @typedef {object} State
 * @property {{ running: 'unlooked' | 'looking' | 'looked' | 'unread' }} at
 * @property {Record<string, number>} [asked]
 * @property {Shown[]} [found]
 * @property {string} [why]
 */

/**
 * The messages RUNNING takes: Look pressed; look `n` answered, or why not.
 * @typedef {{ type: 'look' }
 *   | { type: 'looked', n: number, found: Shown[], why: string }
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
  return [accounts, header, keeps, shown.unknown].filter(Boolean).join(' ');
};

/** @type {Machine<State, Message, Effect>} */
export const RUNNING = {
  states: { running: ['unlooked', 'looking', 'looked', 'unread'] },
  io: {
    look: ({ n: serial }) =>
      get('/running').then(
        /** @param {{ running: Shown[], named: string[] }} answer */
        ({ running, named }) => ({
          type: 'looked',
          n: serial,
          found: running.map((row) => {
            const program = row.answer?.program;
            return { ...row, program: program && named.includes(program) ? program : null };
          }),
          why: `${running.length} running`,
        }),
      ),
  },
  step: reads({
    region: 'running',
    start: 'look',
    reading: 'looking',
    done: 'looked',
    at: 'looked',
    fields: ['found'],
    taker: 'what runs here',
  }),
};
