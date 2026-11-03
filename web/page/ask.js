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
import { go, html, no, question, unlent, useEffect } from '../ui/kit.js?v=__VERSION__';
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
const lendOf = (/** @type {State} */ s, /** @type {string | undefined} */ label) =>
    s.lent?.find((l) => l.label === label),
  // Sent back only once something is asked (`not`, `said`).
  back = (/** @type {State} */ s) =>
    /** @satisfies {Send} */ ({
      run: 'send',
      m: {
        type: 'open',
        label: /** @type {Asking} */ (s.asking).app,
        at: /** @type {Asking} */ (s.asking).at,
      },
    });
/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  ask: (s, m, [lend, app] = [m.lend, m.app].map((x) => lendOf(s, x))) =>
    s.at.ask !== 'none'
      ? no(s, `the ask is ${s.at.ask} already`)
      : !lend?.asks || !app
        ? go(s, { ask: 'refused' }, { asking: null, answer: unlent })
        : // Both found, both are named.
          go(
            s,
            { ask: 'asking' },
            {
              asking: {
                lend: /** @type {string} */ (m.lend),
                app: /** @type {string} */ (m.app),
                at: m.at ?? '/',
              },
            },
          ),
  say: (s) =>
    !s.asking || s.at.ask === 'sending'
      ? no(s, `nothing is asked: the ask is ${s.at.ask}`)
      : go(
          s,
          { ask: 'sending' },
          {},
          {
            run: 'answer',
            group: s.group,
            // Asked, its lend asks (`ask`), so it is listed.
            l: /** @type {Lent} */ (lendOf(s, s.asking.lend)),
            app: s.asking.app,
            failed: 'unsaid',
          },
        ),
  not: (s) =>
    !s.asking || s.at.ask === 'sending'
      ? no(s, `nothing is asked: the ask is ${s.at.ask}`)
      : go(s, {}, {}, back(s)),
  said: (s) => (s.at.ask !== 'sending' ? no(s, 'no answer is being sent') : go(s, {}, {}, back(s))),
  unsaid: (s, m) =>
    s.at.ask !== 'sending'
      ? no(s, 'no answer is being sent')
      : go(s, { ask: 'refused' }, { answer: m.said ?? 'Your answer could not be sent.' }),
};
/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  // Asking or sending, both are listed (`ask`).
  ask: (s) =>
    ['asking', 'sending'].includes(s.at.ask)
      ? [
          question(
            /** @type {Lent} */ (lendOf(s, /** @type {Asking} */ (s.asking).app)),
            /** @type {Lent} */ (lendOf(s, /** @type {Asking} */ (s.asking).lend)),
          ),
          'good',
        ]
      : s.at.ask === 'refused' && s.asking && [/** @type {string} */ (s.answer), 'bad'],
};
// Refused, it says why (`ask`, `unsaid`).
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  ask: (s) => s.at.ask === 'refused' && !s.asking && [/** @type {string} */ (s.answer)],
};

// Sent once the list is read and the address asks, and then not again.
/**
 * Drawn only once the address asks (`view`).
 * @typedef {object} AskingProps
 * @property {State & { wants: Wants }} s
 * @property {Props['send']} send
 */
/** @param {AskingProps} props */
const Asking = ({ s, send }) => (
  useEffect(
    () =>
      void (
        s.at.list === 'read' &&
        s.at.ask === 'none' &&
        send({
          type: 'ask',
          lend: s.wants.ask,
          app: s.wants.app,
          at: s.wants.at,
          why: `whether ${s.wants.app} may use ${s.wants.ask}`,
        })
      ),
    [s.at.list],
  ),
  null
);
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  ask: ({ s, send }) =>
    html`<form id="ask" hidden=${!s.asking || s.at.ask === 'sending'} onSubmit=${(
      /** @type {Event} */ e,
    ) =>
      (
        e.preventDefault(),
        send({ type: 'say' })
      )}><blockquote id="words">${s.asking && lendOf(s, s.asking.lend)?.asks}</blockquote><button>Yes</button><button type="button" id="not" onClick=${() => send({ type: 'not' })}>Not now</button>${s.wants?.ask && html`<${Asking} s=${s} send=${send} />`}</form>`,
};
/** @type {Io<Effect, PageMessage>} */
export const io = {
  answer: ({ group, l, app }) =>
    choose(group, l, app, 'yes').then(
      () => ({ type: 'said', why: `the yes kept at ${l.person}'s computer` }),
      (e) => ({ type: 'unsaid', said: e.said, why: e.message }),
    ),
};
