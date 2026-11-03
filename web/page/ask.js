// SPDX-License-Identifier: AGPL-3.0-or-later
// The person asked whether an app may use a lend. A lent page sends its
// person to `#ask=<lend>&app=<its label>&at=<path>` (lend.js `asking`),
// and once the group is read the page asks in svrngs's words around the
// lender's: its heading is question.js `question`, and the lender's words
// are in #words (ask-form.js). Yes posts the person's yes to its lender's
// door (uses.js `choose`), then opens the app again at `at`, as Not now
// does at once (index.js `open`). An app or a lend not lent here, or not
// asking, is said, and nothing is asked. A refused yes is said in the
// heading with the form still shown, so it can be said again.
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { question, unlent } from '../question.js?v=__VERSION__';
import { AskForm } from './ask-form.js?v=__VERSION__';
import { choose } from './uses.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Props, Said, Send, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): what is asked, by the labels of the
 * lend and the app and the path to open the app at; and why it was refused.
 * @typedef {object} Fields
 * @property {{ lend: string, app: string, at: string } | null} [asking]
 * @property {string} [answer]
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'ask', lend?: string, app?: string, at?: string }
 *   | { type: 'say' }
 *   | { type: 'not' }
 *   | { type: 'said' }
 *   | { type: 'unsaid', said?: string }} Message
 */

/**
 * The effect its steps answer: the person's yes, posted to the lender of
 * the lend asked of.
 * @typedef {{ run: 'answer', group: Group, l: Lent, app: string, failed: string }} Effect
 */

/**
 * What is asked, once `ask` is `asking`, `sending`, or `refused` with the
 * form still shown.
 * @typedef {NonNullable<Fields['asking']>} Asking
 */

export const owns = ['asking', 'answer'];
export const states = /** @type {const} */ ({ ask: ['none', 'asking', 'sending', 'refused'] });

/**
 * @param {State} state
 * @param {string | undefined} label
 */
const lendOf = (state, label) => state.lent?.find((lend) => lend.label === label);

/**
 * The app asked of opened again where it was. Sent only once something
 * is asked (`not`, `said`).
 * @param {State} state
 */
const back = (state) => {
  const asking = /** @type {Asking} */ (state.asking);
  return /** @satisfies {Send} */ ({
    run: 'send',
    m: { type: 'open', label: asking.app, at: asking.at },
  });
};

/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  ask: (state, message) => {
    if (state.at.ask !== 'none') return no(state, `the ask is ${state.at.ask} already`);
    const lend = lendOf(state, message.lend);
    const app = lendOf(state, message.app);
    if (!lend?.asks || !app) return go(state, { ask: 'refused' }, { asking: null, answer: unlent });
    // Both found, both are named.
    const asking = {
      lend: /** @type {string} */ (message.lend),
      app: /** @type {string} */ (message.app),
      at: message.at ?? '/',
    };
    return go(state, { ask: 'asking' }, { asking });
  },
  say: (state) => {
    if (!state.asking || state.at.ask === 'sending')
      return no(state, `nothing is asked: the ask is ${state.at.ask}`);
    return go(
      state,
      { ask: 'sending' },
      {},
      {
        run: 'answer',
        group: state.group,
        // Asked, its lend asks (`ask`), so it is listed.
        l: /** @type {Lent} */ (lendOf(state, state.asking.lend)),
        app: state.asking.app,
        failed: 'unsaid',
      },
    );
  },
  not: (state) => {
    if (!state.asking || state.at.ask === 'sending')
      return no(state, `nothing is asked: the ask is ${state.at.ask}`);
    return go(state, {}, {}, back(state));
  },
  said: (state) => {
    if (state.at.ask !== 'sending') return no(state, 'no answer is being sent');
    return go(state, {}, {}, back(state));
  },
  unsaid: (state, message) => {
    if (state.at.ask !== 'sending') return no(state, 'no answer is being sent');
    const answer = message.said ?? 'Your answer could not be sent.';
    return go(state, { ask: 'refused' }, { answer });
  },
};

/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  ask: (state) => {
    if (state.at.ask === 'asking' || state.at.ask === 'sending') {
      // Asking or sending, both are listed (`ask`).
      const asking = /** @type {Asking} */ (state.asking);
      const app = /** @type {Lent} */ (lendOf(state, asking.app));
      const lend = /** @type {Lent} */ (lendOf(state, asking.lend));
      return [question(app, lend), 'good'];
    }
    // Refused with the form still shown, it says why in the heading
    // (`unsaid`).
    return (
      state.at.ask === 'refused' && state.asking && [/** @type {string} */ (state.answer), 'bad']
    );
  },
};

// Refused before anything was asked, it says why (`ask`).
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  ask: (state) =>
    state.at.ask === 'refused' && !state.asking && [/** @type {string} */ (state.answer)],
};

/** @type {Io<Effect, PageMessage>} */
export const io = {
  answer: ({ group, l: lend, app }) =>
    choose(group, lend, app, 'yes').then(
      () => ({ type: 'said', why: `the yes kept at ${lend.person}'s computer` }),
      (error) => ({ type: 'unsaid', said: error.said, why: error.message }),
    ),
};

/** @type {Record<string, (props: Props) => Drawn>} */
export const view = { ask: AskForm };
