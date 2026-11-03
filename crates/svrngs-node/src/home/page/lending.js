// SPDX-License-Identifier: AGPL-3.0-or-later
// "What you lend" (page-index (e); home.js `lending` and `signed`,
// :252-287): what this computer lends, each with how many it lets in at
// once — its person's choice, a number or everyone in their groups — to
// set, clear, sign people in to or stop here, taken by the door at its next
// request. An app a group carries is the group's, not this computer's to
// change, so not listed. Its machine, LENDING, is the place's own
// (ui/view.js `useMachine`), each read counted, so an older answer is
// refused by name; it reads as it is drawn and again on each of HOME's
// rounds (`again`, `visible`), and the last read stays drawn while the next
// is asked. Each lend is a row with a machine of its own, CHANGE, its
// region `change`: a second press while one change is under way is refused.
// A change is answered by HOME's `again`, the groups and this read again,
// where v1 loaded the page again (home.js@19b63b2b:283; page-index (e)'s fix), and the
// typed number and what was said go with it, as v1's load drops them. Why
// a change or the number is refused is said once, beside the buttons, the
// last in place of the one before, where v1 appended each again (home.js@19b63b2b:283,
// :285; page-index (e)'s second fix). The
// secret from "Sign people in with svrngs" is shown in its row's `shown`,
// once, and nowhere else: never in a request, a link or a trace line
// (SIGN_IN_WITH_SVRNGS §5 "Turning it on"); a read again keeps the row, so
// keeps it shown, and a load of the page, which reads none, shows it no more.
import {
  Act,
  ask,
  assertNever,
  Field,
  go,
  html,
  late,
  machine,
  no,
  useEffect,
  useMachine,
} from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lend } from '../question.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * The field of HOME's that `take` sets: whether what you lend is read.
 * @typedef {{ lentRead?: boolean }} Fields
 */

/**
 * HOME's message this file takes: what you lend read.
 * @typedef {{ type: 'told', flow: 'lending', why: string }} Taken
 */

/**
 * The list's state: where its read is, what this computer lends, which of
 * them sign people in with svrngs, by name, and why it could not be read.
 * @typedef {object} State
 * @property {{ lent: 'reading' | 'read' | 'unread' }} at
 * @property {Record<string, number>} [asked]
 * @property {Lend[]} [lent]
 * @property {string[]} [signing]
 * @property {string} [why]
 */

/**
 * The messages the list takes: read again; read `n` answered (home.rs
 * `lent`), or unread, with why.
 * @typedef {{ type: 'read' }
 *   | { type: 'answered', n: number, lent: Lend[], signing: string[], why: string }
 *   | { type: 'unread', n: number, why: string }} Message
 */

/**
 * The effect its step answers: read `n` of what this computer lends.
 * @typedef {{ run: 'read', n: number, failed: 'unread' }} Effect
 */

/**
 * The app form's fill, the one time signing people in is turned on
 * (home.rs `change`).
 * @typedef {{ issuer: string, client_id: string, client_secret: string | null, deviations: string }} Signed
 */

/**
 * A change of a lend: how many it lets in at once, a number or everyone
 * (`null`), or its lending stopped.
 * @typedef {{ limit: number | null } | 'stop'} To
 */

/**
 * A lend's row's state: where its change is, the number typed, what was
 * last said, and the app form's fill, once.
 * @typedef {object} ChangeState
 * @property {{ change: 'idle' | 'changing' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} said
 * @property {string} [many]
 * @property {Signed} [shown]
 */

/**
 * The messages a row takes: a number typed, in the Field, which names no
 * field here; a change pressed; Sign people in pressed; Let in this many
 * pressed, with the number; the change made, signing on, or refused, with
 * why.
 * @typedef {{ type: 'typed', value: string }
 *   | { type: 'change', to: To }
 *   | { type: 'sign' }
 *   | { type: 'many', value: string }
 *   | { type: 'changed', why: string }
 *   | { type: 'signed', c: Signed, why: string }
 *   | { type: 'refused', why: string }} ChangeMessage
 */

/**
 * The effects a row's step answers: the change posted, and signing on.
 * @typedef {{ run: 'change', to: To, failed: 'refused' }
 *   | { run: 'sign', failed: 'refused' }} ChangeEffect
 */

/** @type {Machine<State, Message, Effect>} */
const LENDING = {
  states: { lent: ['reading', 'read', 'unread'] },
  io: {
    read: ({ n: serial }) =>
      get('/lent').then(
        /** @param {{ lent: Lend[], signing: string[] }} answer */
        ({ lent, signing }) => ({
          type: 'answered',
          n: serial,
          lent: lent.filter((lend) => !lend.group),
          signing,
          why: `${lent.length} lent`,
        }),
      ),
  },
  // Each message's read numbered, and an answer older than the last read
  // asked refused by name, before any case takes it.
  step: (state, message) => {
    const [asked, serial] = ask(state, 'lent');
    const old = message.type !== 'read' && late(state, 'lent', message.n);
    if (old) return no(state, old);
    switch (message.type) {
      case 'read':
        return go(
          state,
          { lent: 'reading' },
          { asked },
          { run: 'read', n: serial, failed: 'unread' },
        );
      case 'answered':
        return go(state, { lent: 'read' }, { lent: message.lent, signing: message.signing ?? [] });
      case 'unread':
        return go(state, { lent: 'unread' }, { why: message.why });
      default:
        return assertNever(state, message, 'what you lend');
    }
  },
};
// The change asked, `run` with what it carries, unless one is under way.
/**
 * @param {ChangeState} state
 * @param {ChangeEffect} effect
 * @returns {Step<ChangeState, ChangeEffect>}
 */
const changing = (state, effect) =>
  state.at.change === 'changing'
    ? no(state, 'a change is under way, and is answered first')
    : go(state, { change: 'changing' }, {}, effect);
/**
 * @param {string} lend
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<ChangeState, ChangeMessage, ChangeEffect>}
 */
const CHANGE = (lend, home) => ({
  states: { change: ['idle', 'changing', 'refused'] },
  io: {
    change: ({ to }) =>
      post('/lent', { lend, to }).then(() => {
        home({ type: 'again' });
        return { type: 'changed', why: 'the groups read again' };
      }),
    sign: () =>
      post('/lent', { lend, to: 'sign-in' }).then((signIn) => ({
        type: 'signed',
        c: signIn,
        why: 'its secret shown here once',
      })),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'typed':
        return go(state, {}, { many: message.value });
      case 'change':
        return changing(state, { run: 'change', to: message.to, failed: 'refused' });
      case 'sign':
        return changing(state, { run: 'sign', failed: 'refused' });
      case 'many':
        return /^[1-9][0-9]*$/.test(message.value)
          ? changing(state, {
              run: 'change',
              to: { limit: Number(message.value) },
              failed: 'refused',
            })
          : go(state, { change: 'refused' }, { said: ' Say how many: 1 or more.' });
      case 'changed':
        return go(state, { change: 'idle' }, { many: undefined, said: '' });
      case 'signed':
        return go(state, { change: 'idle' }, { shown: message.c });
      case 'refused':
        return go(state, { change: 'refused' }, { said: ` ${message.why}` });
      default:
        return assertNever(state, message, 'a lend here');
    }
  },
});

// What a lend's app form is filled with, the one time its lender turns
// signing people in with svrngs on: the issuer, the client and its secret,
// and the deviations in their words (home.js@19b63b2b:256).
/** @param {Signed} char */
const signed = (char) =>
  html`<p>Signing people in with svrngs. In the app’s own settings, choose OpenID Connect, and fill in: issuer <code>${char.issuer}</code>, client id <code>${char.client_id}</code>, client secret <code>${char.client_secret}</code>${`. ${char.deviations} Copy the secret now: it is not shown again. `}</p>`;
/**
 * A lend's row: the lend, whether it signs people in with svrngs, and
 * HOME's `dispatch`, told to read again once it changes.
 * @typedef {object} LentProps
 * @property {Lend} l
 * @property {boolean} signs
 * @property {HOME.PlaceProps['home']} home
 */

/** @param {LentProps} props */
function Lent({ l: lend, signs, home }) {
  const [state, send] = useMachine(() =>
    machine('CHANGE', CHANGE(lend.name, home), { at: { change: 'idle' }, said: '' }),
  );
  const limit = lend.limit;
  const value = state.many ?? String(limit ?? '');
  /**
   * @param {string} words
   * @param {ChangeMessage} on
   */
  const act = (words, on) =>
    html`<${Act} words=${words} doing=${{ state: state.at.change === 'changing' ? 'busy' : 'idle' }} on=${on} send=${send} />`;
  return html`<li>${`${lend.name}: ${limit ? `up to ${limit} ${limit === 1 ? 'person' : 'people'}` : 'everyone in your groups'} at once, ${lend.using} using it now. `}<${Field} words="How many people at once: " value=${value} attrs=${{ type: 'number', min: 1 }} send=${send} />${act('Let in this many', { type: 'many', value })}${limit && act('Let everyone in', { type: 'change', to: { limit: null } })}${signs ? 'Signs people in with svrngs. ' : act('Sign people in with svrngs', { type: 'sign' })}${act('Stop lending', { type: 'change', to: 'stop' })}${state.said}${state.shown && signed(state.shown)}</li>`;
}
// HOME's: the page's foot held until what you lend is first read, as v1
// drew its address after it (home.js@19b63b2b:326-328): drawn at once, a driver read the
// list empty (main.rs:2524 on v2).
/**
 * @param {HOME.State} state
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state) => go(state, {}, { lentRead: true });
// Asked only past the groups' read (home.js `Home`, `held`, past its first
// `return`), so `state.groups` is read.
/** @param {HOME.State} state */
export const holds = (state) => /** @type {Groups} */ (state.groups).length > 0 && !state.lentRead;
/** @typedef {NonNullable<HOME.State['groups']>} Groups */

/**
 * The list, read again on each of HOME's rounds: the round, HOME's
 * `dispatch`, and whether HOME is told it was read.
 * @typedef {object} LendingProps
 * @property {HOME.State['round']} round
 * @property {HOME.PlaceProps['home']} home
 * @property {boolean | undefined} told
 */

/** @param {LendingProps} props */
function Lending({ round, home, told }) {
  const [state, send] = useMachine(() => machine('LENDING', LENDING, { at: { lent: 'reading' } }));
  useEffect(() => send({ type: 'read' }), [round]);
  useEffect(
    () =>
      void (
        !told &&
        state.at.lent !== 'reading' &&
        home({ type: 'told', flow: 'lending', why: `what you lend is ${state.at.lent}` })
      ),
    [state.at.lent],
  );
  // `signing` is set with `lent`, by `answered`, so it is there with it.
  return html`<section id="lent"><h2>What you lend</h2>${state.at.lent === 'unread' ? html`<p>This computer cannot read what you lend: ${state.why}</p>` : state.lent && html`${!state.lent.length && html`<p>You lend nothing yet.</p>`}<ul>${state.lent.map((lend) => html`<${Lent} key=${lend.name} l=${lend} signs=${/** @type {string[]} */ (state.signing).includes(lend.name)} home=${home} />`)}</ul>`}</section>`;
}
// Drawn once the groups are read (home.js `Home`), so `state.groups` is read
// then.
/** @type {{ lent: (props: HOME.PlaceProps) => Drawn }} */
export const view = {
  lent: ({ s: state, home }) =>
    /** @type {Groups} */ (state.groups).length > 0 &&
    html`<${Lending} round=${state.round} home=${home} told=${state.lentRead} />`,
};
