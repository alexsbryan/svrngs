// SPDX-License-Identifier: AGPL-3.0-or-later
// FOLDER, the machine of a group's folder's button on the home page
// (group-folder.js `Folder`): pressed, it asks /folder to open the folder
// in a window of its own. Each open is counted, so when a person presses
// twice only the last answer is said, once, in place of the one before.
import { ask, assertNever, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { post } from './routes.js?v=__VERSION__';
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */

/**
 * A folder's button's state: where its open is, the opens asked, and the
 * last one's answer, said.
 * @typedef {object} State
 * @property {{ folder: 'idle' | 'opening' | 'opened' | 'unopened' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} said
 */

/**
 * The messages a folder's button takes: pressed, for the group's folder
 * `dir`; then open `n` opened, or not, with why.
 * @typedef {{ type: 'open', group: string, dir: string }
 *   | { type: 'opened', n: number, why: string }
 *   | { type: 'unopened', n: number, why: string }} Message
 */

/**
 * The effect its step answers: the folder opened, open `n`.
 * @typedef {{ run: 'open', n: number, group: string, dir: string, failed: 'unopened' }} Effect
 */

/**
 * How the page names a group's folder.
 * @param {{ name: string | null }} group
 */
export const its = (group) => (group.name ? `${group.name}’s folder` : 'The group’s folder');

// An open's answer said, unless a later open was asked since.
/**
 * @param {State} state
 * @param {Extract<Message, { n: number }>} message
 * @param {string} words
 * @returns {Step<State, Effect>}
 */
const answered = (state, message, words) => {
  const old = late(state, 'open', message.n);
  if (old) return no(state, old);
  return go(state, { folder: message.type }, { said: words });
};

/** @type {Machine<State, Message, Effect>} */
export const FOLDER = {
  states: { folder: ['idle', 'opening', 'opened', 'unopened'] },
  step: (state, message) => {
    switch (message.type) {
      case 'open': {
        const [asked, serial] = ask(state, 'open');
        return go(
          state,
          { folder: 'opening' },
          { asked },
          { run: 'open', n: serial, group: message.group, dir: message.dir, failed: 'unopened' },
        );
      }
      case 'opened':
        return answered(state, message, ' Opened, in a window of its own.');
      case 'unopened':
        return answered(state, message, ` It could not be opened: ${message.why}.`);
      default:
        return assertNever(state, message, 'a folder');
    }
  },
  io: {
    open: ({ n: serial, group, dir }) =>
      post('/folder', { group, dir }).then(() => ({ type: 'opened', n: serial, why: dir })),
  },
};
