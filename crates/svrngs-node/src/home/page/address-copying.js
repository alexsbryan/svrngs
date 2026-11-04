// SPDX-License-Identifier: AGPL-3.0-or-later
// A copy of a lend's address, for a program on this computer, in a group's
// foot (address-copy.js). It says its last press's words in place of the
// one before, where ui/copy.js's COPY keeps each.
import { assertNever, go } from '../ui/machine.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */

/**
 * A copy's state: whether it is copied, and its last press's words.
 * @typedef {{ at: { copy: 'uncopied' | 'copied' }, asked?: Record<string, number>, said: string }} Copied
 */

/**
 * The messages a copy takes: Copy pressed; and the address copied, in its
 * words, or why not.
 * @typedef {{ type: 'copy' } | { type: 'copied', words?: string, why?: string }} Copying
 */

/**
 * @param {string} at
 * @returns {Machine<Copied, Copying, { run: 'copy', failed: 'copied' }>}
 */
export const COPY = (at) => ({
  states: { copy: ['uncopied', 'copied'] },
  io: {
    copy: () =>
      Promise.resolve()
        .then(() => navigator.clipboard.writeText(at))
        .then(
          () => ({
            type: 'copied',
            words: ` Copied: ${at}. Paste it where a program on this computer, such as a chat app, asks for an address.`,
          }),
          () => ({ type: 'copied', why: 'the clipboard refused' }),
        ),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'copy':
        return go(state, {}, {}, { run: 'copy', failed: 'copied' });
      case 'copied':
        return go(
          state,
          { copy: 'copied' },
          { said: message.words ?? ` Copy it from here: ${at}` },
        );
      default:
        return assertNever(state, message, 'a copy');
    }
  },
});
