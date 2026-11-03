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
    groups.map((g) =>
      g.removed === false
        ? get(`/v1/lends?namespace=${encodeURIComponent(g.id)}`).then(
            (l) => l.lent,
            () => [],
          )
        : [],
    ),
  ).then((a) => a.flat());
// The app at `back`, kept only on its own origin, as open.js judges its way back (home.js@19b63b2b:233-236).
/**
 * @param {Lent} app
 * @param {string | null | undefined} back
 * @returns {Extract<Message, { type: 'gone' }>}
 */
const leave = (app, back) => (
  location.assign(
    `/to?${new URLSearchParams({ lend: label(app), at: ((u) => (u.origin === new URL(app.origin).origin ? u.pathname + u.search + u.hash : '/'))(new URL(back ?? '/', app.origin)) })}`,
  ),
  { type: 'gone', why: `to ${label(app)}` }
);
/**
 * @param {Said} q
 * @returns {Machine<State, Message, Effect>}
 */
const ASK = (q) => ({
  states: { ask: ['reading', 'asking', 'sending', 'unlent'] },
  io: {
    read: ({ groups }) =>
      lentIn(groups).then((lent) => ({ type: 'lent', lent, why: `${lent.length} lent` })),
    yes: ({ lend, app }) =>
      post('/choices', { lend: label(lend), app: label(app), said: 'yes' }).then(() =>
        leave(app, q.back),
      ),
    leave: ({ app }) => leave(app, q.back),
  },
  // `lend` and `app` are set together, by `lent`, and only then is the
  // ask `asking`, or `sending` past it, so each step that reads them reads
  // them set.
  step: (s, m) => {
    switch (m.type) {
      case 'lent':
        return ((lend, app) =>
          !lend || !app || !lend.asks
            ? go(s, { ask: 'unlent' })
            : go(s, { ask: 'asking' }, { lend, app }))(
          .../** @type {[Lent | undefined, Lent | undefined]} */ (
            [q.lend, q.app].map((x) => (m.lent ?? []).find((l) => label(l) === x))
          ),
        );
      case 'yes':
        return s.at.ask === 'asking'
          ? go(
              s,
              { ask: 'sending' },
              {},
              {
                run: 'yes',
                lend: /** @type {Lent} */ (s.lend),
                app: /** @type {Lent} */ (s.app),
                failed: 'unsaid',
              },
            )
          : no(s, `the ask is ${s.at.ask}`);
      case 'not':
        return s.lend
          ? go(s, {}, {}, { run: 'leave', app: /** @type {Lent} */ (s.app), failed: 'unsaid' })
          : no(s, 'nothing is asked');
      case 'unsaid':
        return go(s, { ask: 'asking' }, { said: m.why });
      case 'gone':
        return go(s, {});
      default:
        return assertNever(s, m, 'the ask');
    }
  },
});

// HOME's state, `h`, past its groups' read, as `Home` draws the ask only
// then (home.js `Home`, past its first `return`), so `h.groups` is read.
/** @param {PlaceProps} props */
function Asked({ s: h }) {
  const [s, send] = useMachine(() =>
    machine('ASK', ASK(h.q), { at: { ask: 'reading' }, said: '' }, [
      { run: 'read', groups: /** @type {Group[]} */ (h.groups), failed: 'lent' },
    ]),
  );
  if (s.at.ask === 'reading') return null;
  // Past `reading` and `unlent`, the ask is `asking` or `sending`, so its
  // `lend` and `app` are set (ASK's `step`).
  return s.at.ask === 'unlent'
    ? html`<p>${unlent}</p>`
    : html`<section><h2>${question(/** @type {Lent} */ (s.app), /** @type {Lent} */ (s.lend))}</h2><blockquote>${/** @type {Lent} */ (s.lend).asks}</blockquote><button onClick=${() => send({ type: 'yes' })}>Yes</button><button onClick=${() => send({ type: 'not' })}>Not now</button>${s.said}</section>`;
}
/** @type {{ ask: (props: PlaceProps) => Drawn }} */
export const view = { ask: (props) => html`<${Asked} ...${props} />` };
