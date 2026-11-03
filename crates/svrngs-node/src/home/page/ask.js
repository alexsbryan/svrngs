// SPDX-License-Identifier: AGPL-3.0-or-later
// The ask, at /ask (page-acts (c); home.js `lentIn` and `ask`, :63-66,
// :238-246): what the people of each group you are in lend, read, then
// svrngs's question in its one wording (question.js `question`) around the
// lender's words, or why it is not asked (`unlent`). Yes posts the yes to
// /choices, then leaves for the app at `back` through /to, as Not now
// leaves at once, by location.assign at the edge; a refusal is said after
// the buttons, in place of any before it, where v1 appended each (home.js@19b63b2b:343;
// page-acts (d)). Its machine, ASK, is the place's own (ui/view.js
// `useMachine`).
import {
  assertNever,
  go,
  html,
  machine,
  no,
  question,
  unlent,
  useMachine,
} from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';
import { label } from './index.js?v=__VERSION__';
/** @import { Group, Lent } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PlaceProps, Said } from '../home.js?v=__VERSION__' */

/**
 * The ask's state: where it is, what was last said of it, and, once the
 * lends are read and both are lent, the lend asked for and the app asking.
 * @typedef {object} State
 * @property {{ ask: 'reading' | 'asking' | 'sending' | 'unlent' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} said
 * @property {Lent} [lend]
 * @property {Lent} [app]
 */

/**
 * The messages the ask takes: the lends read, none when they could not be
 * (its read's `failed`); Yes or Not now pressed; the yes not said, with
 * why; and the page gone to the app.
 * @typedef {{ type: 'lent', lent?: Lent[], why: string }
 *   | { type: 'yes' }
 *   | { type: 'not' }
 *   | { type: 'unsaid', why: string }
 *   | { type: 'gone', why: string }} Message
 */

/**
 * The effects its step answers: the groups' lends read, the yes posted
 * then the app gone to, and the app gone to.
 * @typedef {{ run: 'read', groups: Group[], failed: 'lent' }
 *   | { run: 'yes', lend: Lent, app: Lent, failed: 'unsaid' }
 *   | { run: 'leave', app: Lent, failed: 'unsaid' }} Effect
 */

// A group's lends, none when they cannot be read, or who is in it cannot (home.js@19b63b2b:63-66).
/**
 * @param {Group[]} groups
 * @returns {Promise<Lent[]>}
 */
const lentIn = (groups) =>
  Promise.all(
    groups.map((group) =>
      group.removed === false
        ? get(`/v1/lends?namespace=${encodeURIComponent(group.id)}`).then(
            (lend) => lend.lent,
            () => [],
          )
        : [],
    ),
  ).then((answers) => answers.flat());
// The app at `back`, kept only on its own origin, as open.js judges its way back (home.js@19b63b2b:233-236).
/**
 * @param {Lent} app
 * @param {string | null | undefined} back
 * @returns {Extract<Message, { type: 'gone' }>}
 */
const leave = (app, back) => (
  location.assign(
    `/to?${new URLSearchParams({ lend: label(app), at: ((url) => (url.origin === new URL(app.origin).origin ? url.pathname + url.search + url.hash : '/'))(new URL(back ?? '/', app.origin)) })}`,
  ),
  { type: 'gone', why: `to ${label(app)}` }
);
/**
 * @param {Said} asked
 * @returns {Machine<State, Message, Effect>}
 */
const ASK = (asked) => ({
  states: { ask: ['reading', 'asking', 'sending', 'unlent'] },
  io: {
    read: ({ groups }) =>
      lentIn(groups).then((lent) => ({ type: 'lent', lent, why: `${lent.length} lent` })),
    yes: ({ lend, app }) =>
      post('/choices', { lend: label(lend), app: label(app), said: 'yes' }).then(() =>
        leave(app, asked.back),
      ),
    leave: ({ app }) => leave(app, asked.back),
  },
  // `lend` and `app` are set together, by `lent`, and only then is the
  // ask `asking`, or `sending` past it, so each step that reads them reads
  // them set.
  step: (state, message) => {
    switch (message.type) {
      case 'lent':
        return ((lend, app) =>
          !lend || !app || !lend.asks
            ? go(state, { ask: 'unlent' })
            : go(state, { ask: 'asking' }, { lend, app }))(
          .../** @type {[Lent | undefined, Lent | undefined]} */ (
            [asked.lend, asked.app].map((wanted) =>
              (message.lent ?? []).find((lend) => label(lend) === wanted),
            )
          ),
        );
      case 'yes':
        return state.at.ask === 'asking'
          ? go(
              state,
              { ask: 'sending' },
              {},
              {
                run: 'yes',
                lend: /** @type {Lent} */ (state.lend),
                app: /** @type {Lent} */ (state.app),
                failed: 'unsaid',
              },
            )
          : no(state, `the ask is ${state.at.ask}`);
      case 'not':
        return state.lend
          ? go(
              state,
              {},
              {},
              { run: 'leave', app: /** @type {Lent} */ (state.app), failed: 'unsaid' },
            )
          : no(state, 'nothing is asked');
      case 'unsaid':
        return go(state, { ask: 'asking' }, { said: message.why });
      case 'gone':
        return go(state, {});
      default:
        return assertNever(state, message, 'the ask');
    }
  },
});

// HOME's state, `home`, past its groups' read, as `Home` draws the ask only
// then (home.js `Home`, past its first `return`), so `home.groups` is read.
/** @param {PlaceProps} props */
function Asked({ s: home }) {
  const [state, send] = useMachine(() =>
    machine('ASK', ASK(home.q), { at: { ask: 'reading' }, said: '' }, [
      { run: 'read', groups: /** @type {Group[]} */ (home.groups), failed: 'lent' },
    ]),
  );
  if (state.at.ask === 'reading') return null;
  // Past `reading` and `unlent`, the ask is `asking` or `sending`, so its
  // `lend` and `app` are set (ASK's `step`).
  return state.at.ask === 'unlent'
    ? html`<p>${unlent}</p>`
    : html`<section><h2>${question(/** @type {Lent} */ (state.app), /** @type {Lent} */ (state.lend))}</h2><blockquote>${/** @type {Lent} */ (state.lend).asks}</blockquote><button onClick=${() => send({ type: 'yes' })}>Yes</button><button onClick=${() => send({ type: 'not' })}>Not now</button>${state.said}</section>`;
}
/** @type {{ ask: (props: PlaceProps) => Drawn }} */
export const view = { ask: (props) => html`<${Asked} ...${props} />` };
