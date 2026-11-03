// SPDX-License-Identifier: AGPL-3.0-or-later
// The person asked whether an app may use a lend (page-acts (b); web/app.js
// `asking`, :298-325): a lent page sends its person to
// `#ask=<lend>&app=<its label>&at=<path>` (lend.js `asking`), and once the
// group is read the page asks in svrngs's words around the lender's, its
// heading question.js `question`, the lender's in #words; Yes posts the
// person's yes to its lender's door (web/page/uses.js `choose`), then opens
// the app again at `at`, as Not now does at once (web/page/index.js `open`).
// An app or a lend not lent here, or not asking, is said, and nothing asked.
// A refused yes is said in the heading with the form still shown, so it
// can be said again, where v1 left the form hidden (app.js@19b63b2b:316; page-acts (d)).
// The ask is sent by the page once its list is read (`Asking`), as no flow
// sends it: the read is enter's.
import { html, useEffect } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { question, unlent } from '../question.js?v=__VERSION__';
import { choose } from './uses.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lent, Wants } from './enter.js?v=__VERSION__' */
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
const lendOf = (/** @type {State} */ state, /** @type {string | undefined} */ label) =>
  state.lent?.find((lend) => lend.label === label);
// Sent back only once something is asked (`not`, `said`).
const back = (/** @type {State} */ state) =>
  /** @satisfies {Send} */ ({
    run: 'send',
    m: {
      type: 'open',
      label: /** @type {Asking} */ (state.asking).app,
      at: /** @type {Asking} */ (state.asking).at,
    },
  });
/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  ask: (
    state,
    message,
    [lend, app] = [message.lend, message.app].map((label) => lendOf(state, label)),
  ) => {
    if (state.at.ask !== 'none') return no(state, `the ask is ${state.at.ask} already`);
    if (!lend?.asks || !app) return go(state, { ask: 'refused' }, { asking: null, answer: unlent });
    // Both found, both are named.
    return go(
      state,
      { ask: 'asking' },
      {
        asking: {
          lend: /** @type {string} */ (message.lend),
          app: /** @type {string} */ (message.app),
          at: message.at ?? '/',
        },
      },
    );
  },
  say: (state) =>
    !state.asking || state.at.ask === 'sending'
      ? no(state, `nothing is asked: the ask is ${state.at.ask}`)
      : go(
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
        ),
  not: (state) =>
    !state.asking || state.at.ask === 'sending'
      ? no(state, `nothing is asked: the ask is ${state.at.ask}`)
      : go(state, {}, {}, back(state)),
  said: (state) =>
    state.at.ask !== 'sending'
      ? no(state, 'no answer is being sent')
      : go(state, {}, {}, back(state)),
  unsaid: (state, message) =>
    state.at.ask !== 'sending'
      ? no(state, 'no answer is being sent')
      : go(state, { ask: 'refused' }, { answer: message.said ?? 'Your answer could not be sent.' }),
};
/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  // Asking or sending, both are listed (`ask`).
  ask: (state) =>
    ['asking', 'sending'].includes(state.at.ask)
      ? [
          question(
            /** @type {Lent} */ (lendOf(state, /** @type {Asking} */ (state.asking).app)),
            /** @type {Lent} */ (lendOf(state, /** @type {Asking} */ (state.asking).lend)),
          ),
          'good',
        ]
      : state.at.ask === 'refused' && state.asking && [/** @type {string} */ (state.answer), 'bad'],
};
// Refused, it says why (`ask`, `unsaid`).
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  ask: (state) =>
    state.at.ask === 'refused' && !state.asking && [/** @type {string} */ (state.answer)],
};

// Sent once the list is read and the address asks, and then not again.
/**
 * Drawn only once the address asks (`view`).
 * @typedef {object} AskingProps
 * @property {State & { wants: Wants }} s
 * @property {Props['send']} send
 */
/** @param {AskingProps} props */
const Asking = ({ s: state, send }) => {
  useEffect(
    () =>
      void (
        state.at.list === 'read' &&
        state.at.ask === 'none' &&
        send({
          type: 'ask',
          lend: state.wants.ask,
          app: state.wants.app,
          at: state.wants.at,
          why: `whether ${state.wants.app} may use ${state.wants.ask}`,
        })
      ),
    [state.at.list],
  );
  return null;
};
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  ask: ({ s: state, send }) =>
    html`<form id="ask" hidden=${!state.asking || state.at.ask === 'sending'} onSubmit=${(
      /** @type {Event} */ event,
    ) => {
      event.preventDefault();
      return send({ type: 'say' });
    }}><blockquote id="words">${state.asking && lendOf(state, state.asking.lend)?.asks}</blockquote><button>Yes</button><button type="button" id="not" onClick=${() => send({ type: 'not' })}>Not now</button>${state.wants?.ask && html`<${Asking} s=${state} send=${send} />`}</form>`,
};
/** @type {Io<Effect, PageMessage>} */
export const io = {
  answer: ({ group, l: lend, app }) =>
    choose(group, lend, app, 'yes').then(
      () => ({ type: 'said', why: `the yes kept at ${lend.person}'s computer` }),
      (error) => ({ type: 'unsaid', said: error.said, why: error.message }),
    ),
};
