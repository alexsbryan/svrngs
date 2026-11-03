// SPDX-License-Identifier: AGPL-3.0-or-later
// Copy, the first component, both pages' (page-open (a)): a button that
// writes `text` to the clipboard, and its answer. Its machine, COPY, is the
// part's own (ui/view.js `useMachine`), so each Copy on a page answers for
// itself and its trace lines say `COPY:`. The clipboard is its io, the edge,
// and a page may be refused it, so `unwritten` says how to copy by hand.
import { html, useMachine } from './view.js?v=__VERSION__';
import { assertNever, go, machine, no } from './machine.js?v=__VERSION__';
/** @import { Machine, Step, Store } from './machine.js?v=__VERSION__' */
/** @import { Drawn } from './view.js?v=__VERSION__' */

/**
 * A Copy's state: where its copy is, and each answer said, in turn.
 * @typedef {object} CopyState
 * @property {{ copy: 'idle' | 'copying' | 'copied' | 'unwritten' }} at
 * @property {Record<string, number>} [asked]
 * @property {('copied' | 'unwritten')[]} said
 */

/**
 * The messages a Copy takes: pressed, to write `text`; then written, or
 * not, with why (its write's `failed`).
 * @typedef {{ type: 'copy', text: string }
 *   | { type: 'copied' }
 *   | { type: 'unwritten', why: string }} CopyMessage
 */

/**
 * The effect its step answers: `text` written to the clipboard.
 * @typedef {{ run: 'write', text: string, failed: 'unwritten' }} CopyEffect
 */

/** @type {CopyState['at']['copy'][]} */
const AGAIN = ['idle', 'copied', 'unwritten'];
// `said` is each answer in turn, by state: a second " Copied." after the
// first, as home.js's `p.append` leaves both (home.js@19b63b2b:137, :202).
/**
 * @param {CopyState} state
 * @param {Extract<CopyMessage, { type: 'copied' | 'unwritten' }>} message
 * @returns {Step<CopyState, CopyEffect>}
 */
const answered = (state, message) =>
  state.at.copy === 'copying'
    ? go(state, { copy: message.type }, { said: [...state.said, message.type] })
    : no(state, `${message.type} answers no copy, as none is being written`);
/** @type {Machine<CopyState, CopyMessage, CopyEffect>} */
export const COPY = {
  states: { copy: ['idle', 'copying', 'copied', 'unwritten'] },
  step: (state, message) => {
    switch (message.type) {
      case 'copy':
        return AGAIN.includes(state.at.copy)
          ? go(
              state,
              { copy: 'copying' },
              {},
              { run: 'write', text: message.text, failed: 'unwritten' },
            )
          : no(state, 'a copy is being written already');
      case 'copied':
        return answered(state, message);
      case 'unwritten':
        return answered(state, message);
      default:
        return assertNever(state, message, 'a copy');
    }
  },
  io: { write: ({ text }) => navigator.clipboard.writeText(text).then(() => ({ type: 'copied' })) },
};

/** @type {CopyState} */
const FIRST = { at: { copy: 'idle' }, said: [] };
/** @param {CopyProps} props */
export const Copy = (props) => drawn(...useMachine(() => machine('COPY', COPY, FIRST)), props);

// `words` is the button's and each answer's: { copy, copied, unwritten }.
// Without `beside`, the answer is the button's word in its place
// (web/app.js@19b63b2b:611); with it, each answer is said after the button, past
// what `beside` draws there (`true`, nothing), as home.js appends them.
/**
 * The words a Copy says: its button's, `copy`, and each answer's, by the
 * state it answers in.
 * @typedef {{ copy: string } & Partial<Record<CopyState['at']['copy'], string>>} Words
 */

/**
 * A Copy's props: its button's `id`, the `text` it writes, its `words`,
 * and what `beside` draws after the button, when its answers are said
 * there.
 * @typedef {object} CopyProps
 * @property {string} [id]
 * @property {string} text
 * @property {Words} words
 * @property {Drawn} [beside]
 */

/**
 * A Copy drawn from its state.
 * @param {CopyState} state
 * @param {Store<CopyState, CopyMessage>['dispatch']} send
 * @param {CopyProps} props
 * @returns {Drawn}
 */
export function drawn(state, send, { id, text, words, beside }) {
  const press = () => send({ type: 'copy', text });
  if (!beside)
    return html`<button id=${id} onClick=${press}>${words[state.at.copy] ?? words.copy}</button>`;
  return html`<button id=${id} onClick=${press}>${words.copy}</button>${beside}${state.said.map((said) => words[said])}`;
}
