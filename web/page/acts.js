// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the phone (page-acts (b); web/app.js@19b63b2b:490-521),
// ui/acts.js `Acts` drawn in #acts once the group is read, each part
// derived from state: the apps the group carries and their new versions
// (ui/changes.js), what opens friends' things as you, by the yes kept with
// the group (`reads`), each with Stop, then what apps use (web/page/uses.js),
// what you open knowing your name, and who decides for you, of the people
// by this phone's key, `me`, as enter reads it with the group (enter.js
// `read`), as page-members needed it too.
// DOING is one act at a time, keyed by what it acts on, `acting`; each
// act's answer kept in `acted` by that key, so a yes said "opens at its
// new version for you now" stays said once the group, read again, lists
// the change no more. A yes is this phone's own (capability-signed): the
// computer it reaches names the act at /runs, and the phone signs it with
// its own key as its chain's next line, kept with the group before it is
// sent, so no seq is signed twice, then carries every line not yet taken
// (app.js@19b63b2b:497-508). A Stop is kept with the group (app.js@19b63b2b:495). After either, enter
// is given the group as kept, and reads it again; the key stays the edge's.
// Lines a carry left unsent are carried again on each read that holds
// them, where v1 carried them only at the next yes (app.js@19b63b2b:506; page-acts (d)):
// refused, that carry asks no read again, so it is tried once a read.
import {
  Act,
  Acts,
  carrying,
  Changes,
  deciders,
  go,
  html,
  named,
  no,
  once,
  reading,
  spoken,
  useEffect,
} from '../ui/kit.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { through } from './enter.js?v=__VERSION__';
import { PASS_LIFE } from './index.js?v=__VERSION__';
import { Used } from './uses.js?v=__VERSION__';
/** @import { Group, Line, Said } from '../store.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import { Io, Step, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Flow, Message as PageMessage, Props, Send, State } from './page.js?v=__VERSION__' */

/**
 * An act's answer, kept by what it acts on: busy, done or refused, what was
 * said of it, and for a yes, the change it took.
 * @typedef {{ state?: 'busy' | 'done' | 'refused', said?: string, c?: Change }} Acted
 */

/**
 * The fields this flow owns (`owns`): each fold's open, by the fold's name;
 * the act under way, and every act's answer.
 * @typedef {object} Fields
 * @property {Partial<Record<'acts' | 'changes', boolean>>} [folds]
 * @property {string} [acting]
 * @property {Record<string, Acted>} [acted]
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'fold', fold: 'acts', open: boolean }
 *   | { type: 'fold', fold: 'changes', open: boolean }
 *   | { type: 'yes', app: string }
 *   | { type: 'stop reads', label: string }
 *   | { type: 'acted', group?: Group, why?: string }
 *   | { type: 'unacted', group?: Group | null, why?: string }
 *   | { type: 'carry' }} Message
 */

/**
 * The effects its steps answer: the yes to `app`'s new version, run at the
 * computer and signed here; a stop of what the lend `label` reads, kept
 * with the group; and the lines unsent, carried.
 * @typedef {{ run: 'runs', group: Group, app: string, failed: string }
 *   | { run: 'stopping', group: Group, label: string, failed: string }
 *   | { run: 'carrying', group: Group, failed: string }} Effect
 */

export const owns = ['folds', 'acting', 'acted'];
export const states = /** @type {const} */ ({ doing: ['idle', 'busy', 'done', 'refused'] });
/**
 * @param {State} state
 * @param {string} key what the act acts on, its answer kept by it
 * @param {Effect} effect
 * @param {Acted} [patch]
 */
const acting = (state, key, effect, patch = {}) =>
  state.at.doing === 'busy'
    ? no(state, `${state.acting} is under way, and is answered first`)
    : go(
        state,
        { doing: 'busy' },
        { acting: key, acted: { ...state.acted, [key]: { ...state.acted?.[key], ...patch } } },
        effect,
      );
/**
 * @param {State} current
 * @param {Extract<Message, { type: 'acted' | 'unacted' }>} message
 * @param {'done' | 'refused'} state
 */
const answered = (current, message, state) =>
  current.at.doing !== 'busy'
    ? no(current, 'no act is under way')
    : // Busy, an act is under way, so `acting` names it and `acted` holds it (`acting`).
      go(
        current,
        { doing: state },
        {
          acted: {
            ...current.acted,
            [/** @type {string} */ (current.acting)]: {
              .../** @type {Record<string, Acted>} */ (current.acted)[
                /** @type {string} */ (current.acting)
              ],
              state,
              said:
                state === 'refused'
                  ? `${/** @type {Acted} */ (/** @type {Record<string, Acted>} */ (current.acted)[/** @type {string} */ (current.acting)]).said ?? ''} ${message.why}`
                  : '',
            },
          },
        },
        ...(message.group
          ? [
              /** @satisfies {Send} */ ({
                run: 'send',
                m: { type: 'opened', group: message.group, why: 'kept here, so read again' },
              }),
            ]
          : []),
      );
/**
 * @param {'acts' | 'changes'} key
 * @returns {(s: State, m: { open: boolean }) => Step<State, never>}
 */
const fold = (key) => (state, message) =>
  go(state, {}, { folds: { ...state.folds, [key]: message.open } });
/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  'fold acts': fold('acts'),
  'fold changes': fold('changes'),
  yes: (state, { app }) =>
    acting(
      state,
      `yes ${app}`,
      { run: 'runs', group: state.group, app, failed: 'unacted' },
      { c: state.changed?.find((change) => change.group === state.group.id && change.app === app) },
    ),
  'stop reads': (state, { label }) =>
    acting(state, `stop ${label}`, {
      run: 'stopping',
      group: state.group,
      label,
      failed: 'unacted',
    }),
  acted: (state, message) => answered(state, message, 'done'),
  unacted: (state, message) => answered(state, message, 'refused'),
  carry: (state) =>
    state.group?.unsent?.length
      ? acting(state, 'carry', { run: 'carrying', group: state.group, failed: 'unacted' })
      : no(state, 'no line is unsent'),
};

const NOTES = {
  opens: `A Stop here holds from the next pass this phone signs for the app, within ${PASS_LIFE / 3600} hours.`,
};
/**
 * @param {State} state
 * @param {string} key
 * @returns {Acted}
 */
const doing = (state, key) =>
  state.acting === key && state.at.doing === 'busy'
    ? { state: 'busy' }
    : (state.acted?.[key] ?? {});
// Each reading lend's yes or stop as kept, the yes with Stop, its line said
// in its place once stopped (app.js@19b63b2b:512-518).
/**
 * Drawn only for a lend the group holds a yes or a stop of (`opens`), so
 * `said` is that one, and the lend reads.
 * @param {State} state
 * @param {Props['send']} send
 * @param {Lent & { reads: string[] }} lend
 */
const read = (
  state,
  send,
  lend,
  said = /** @type {Said} */ (/** @type {Record<string, Said>} */ (state.group.reads)[lend.label]),
  act = doing(state, `stop ${lend.label}`),
) =>
  act.state === 'done'
    ? html`<p>${named(lend)} stops opening your friends’ ${lend.reads.map(spoken).join(' and ')} as you within ${PASS_LIFE / 3600} hours, when the pass it holds ends.</p>`
    : html`<p>${reading(lend, said)}${said.said === 'yes' && html`<${Act} words="Stop" doing=${act} on=${{ type: 'stop reads', label: lend.label }} send=${send} />`}</p>`;
/**
 * Drawn once the tab holds a group (`view`).
 * @typedef {object} DrawnProps
 * @property {State} s
 * @property {Props['send']} send
 */
/** @param {DrawnProps} props */
function Drawn({ s: state, send }) {
  useEffect(
    () =>
      void (
        state.at.list === 'read' &&
        state.group.unsent?.length &&
        send({ type: 'carry', why: `${state.group.unsent.length} line(s) unsent` })
      ),
    [state.lent],
  );
  if (!state.lent) return html`<section id="acts"></section>`;
  // `act.c` is tested first, so it is the change each `some` reads, and
  // each the filter keeps has one.
  const done = Object.values(state.acted ?? {})
    .filter(
      (act) =>
        act.c &&
        act.state === 'done' &&
        !state.changed?.some(
          (change) =>
            change.app === /** @type {Change} */ (act.c).app &&
            change.group === /** @type {Change} */ (act.c).group,
        ),
    )
    .map((act) => /** @type {Change} */ (act.c));
  const parts = {
    reads: [
      ...once(state.lent)
        .filter((lend) => lend.group)
        .map((lend) => html`<p>${carrying(lend.kind, state.shelf)}</p>`),
      ...Changes({
        g: state.group,
        changed: state.changed && [...state.changed, ...done],
        rows: state.people,
        me: state.me,
        doing: Object.fromEntries(
          Object.keys(state.acted ?? {})
            .filter((key) => key.startsWith('yes '))
            .map((key) => [key.slice(4), doing(state, key)]),
        ),
        open: state.folds?.changes,
        send,
      }),
    ],
    opens: [
      ...state.lent
        .filter((lend) => lend.reads?.length && state.group.reads?.[lend.label])
        // Filtered to the lends that read, as `read` takes them.
        .map((lend) => read(state, send, /** @type {Lent & { reads: string[] }} */ (lend))),
      ...(state.lent.some((lend) => lend.asks) ? [html`<${Used} s=${state} send=${send} />`] : []),
    ],
    knows: [html`<p>What you open knows your name.</p>`],
    decides: [html`<p>${deciders(state.people, state.me, 'browser')}.</p>`],
  };
  return html`<${Acts} parts=${parts} notes=${NOTES} open=${state.folds?.acts} send=${send} />`;
}
/** @type {Required<Flow>['view']} */
export const view = {
  acts: ({ s: state, send }) =>
    state.group ? html`<${Drawn} s=${state} send=${send} />` : html`<section id="acts"></section>`,
};

// The edge: the key unsealed here, never in state, and the wasm imported
// only once app.js has run its init.
const wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__');
// Every line not yet taken carried, in order, then none kept as unsent; or
// why not, with `kept`, the group as kept, its unsent lines in it.
// Carried only with lines unsent: `runs` adds one, and `carry` asks only
// when some are.
/**
 * @param {Group} group
 * @param {string} why
 * @param {Group | null} [kept]
 * @returns {Promise<PageMessage>}
 */
const carry = (group, why, kept = group) =>
  /** @type {Line[]} */ (group.unsent)
    .reduce((chain, line) => chain.then(() => through(group, '/carry', line)), Promise.resolve())
    .then(() => store.opened({ ...group, unsent: [] }))
    .then(
      (opened) => ({ type: 'acted', group: opened, why: `${why}, and carried` }),
      (error) => ({ type: 'unacted', group: kept, why: error.message }),
    );
/** @type {Io<Effect, PageMessage>} */
export const io = {
  runs: ({ group, app }) =>
    Promise.all([wasm(), store.unseal(group), through(group, '/runs', app)]).then(
      ([core, key, { sign }], at = group.line ?? { seq: -1, id: null }) => {
        const line = JSON.parse(
          core.sign_line(
            key,
            group.id,
            Math.floor(Date.now() / 1000),
            at.seq + 1,
            at.id,
            JSON.stringify(sign),
          ),
        );
        return store
          .opened({
            ...group,
            line: { seq: line.seq, id: line.id },
            unsent: [...(group.unsent ?? []), line],
          })
          .then((carried) => carry(carried, `line ${line.seq} signed`));
      },
    ),
  carrying: ({ group }) =>
    carry(group, `${/** @type {Line[]} */ (group.unsent).length} line(s) unsent`, null),
  stopping: ({ group, label }) =>
    store
      .opened({
        ...group,
        reads: { ...group.reads, [label]: { said: 'stop', at: Math.floor(Date.now() / 1000) } },
      })
      .then((opened) => ({ type: 'acted', group: opened, why: `${label}'s reads stopped here` })),
};
