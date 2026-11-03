// SPDX-License-Identifier: AGPL-3.0-or-later
// ACTS, the machine of "What acts as you" (acting.js `Acting`): which apps
// open your friends' things as you, the yeses /reaching gives, and what
// apps use of each asking lend, from /choices, each read counted and read
// again on each of HOME's rounds. A yes to a new version is posted to
// /runs and kept in `acted` by what it acts on, so it stays said once the
// groups are read again; a Stop on what apps use reads the groups again,
// and so the section.
import { ask, assertNever, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Asked } from '../ui/uses.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * What the person said of an app reaching their friends' lends as them,
 * and when, in seconds (door/choices.rs `Choice`, :63-68).
 * @typedef {{ said: 'yes' | 'stop', at: number }} Choice
 */

/**
 * /reaching's answer: each app's `Choice`, by its label (home.rs
 * `reached`); or why it could not be read.
 * @typedef {Record<string, Choice> | { why: string }} Yeses
 */

/**
 * A yes to a new version, kept by what it acts on: the change, and, once
 * answered, how, with each refusal said.
 * @typedef {{ c?: Change, state?: 'done' | 'refused', said?: string }} Acted
 */

/**
 * ACTS's state: where its read and its yes are, the yeses and each asking
 * lend's choices as read, which folds are open, the yes under way, by its
 * group and app, each yes kept, and each Stop's refusal, by lend and app.
 * @typedef {object} State
 * @property {{ reaching: 'reading' | 'read' | 'unread', doing: 'idle' | 'busy' | 'done' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {Yeses} [yeses]
 * @property {Record<string, Asked>} [choices]
 * @property {Record<string, boolean>} [folds]
 * @property {string} [acting]
 * @property {Record<string, Acted>} acted
 * @property {Record<string, string>} said
 */

/**
 * A yes to a new version of `app` in the group `g`, with its change.
 * @typedef {{ type: 'yes', app: string, g: string, c: Change | undefined }} Yes
 */

/**
 * The messages ACTS takes: a read of the yeses and of each asking lend's
 * choices; that read, or why not; a fold opened or closed; a yes, its
 * answer, or why not; a Stop, its answer, or why not, by its lend and
 * app; HOME told.
 * @typedef {{ type: 'read', asking: string[] }
 *   | { type: 'reached', n: number, yeses: Yeses, choices: Record<string, Asked>, why: string }
 *   | { type: 'unreached', why: string }
 *   | { type: 'fold', fold: string, open: boolean }
 *   | Yes
 *   | { type: 'acted', why: string }
 *   | { type: 'unacted', why: string }
 *   | { type: 'stop', lend: string, app: string }
 *   | { type: 'stopped', why: string }
 *   | { type: 'unstopped', lend: string, app: string, why: string }
 *   | { type: 'told', why: string }} Message
 */

/**
 * The effects its step answers: read `n` of the yeses and of `asking`'s
 * choices; a yes to `app`'s new version in the group `g`; HOME told to
 * read the groups again; and `app` stopped using `lend`, a refusal of
 * which its io answers itself, as `unstopped` with both.
 * @typedef {{ run: 'read', n: number, asking: string[], failed: 'unreached' }
 *   | { run: 'yes', g: string, app: string, failed: 'unacted' }
 *   | { run: 'again', failed: 'told' }
 *   | { run: 'stop', lend: string, app: string, failed: 'unstopped' }} Effect
 */

/**
 * ACTS, telling HOME through `home` to read the groups again.
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<State, Message, Effect>}
 */
export const ACTS = (home) => ({
  states: { reaching: ['reading', 'read', 'unread'], doing: ['idle', 'busy', 'done', 'refused'] },
  io: {
    read: ({ n: serial, asking }) =>
      Promise.all([
        get('/reaching').catch((error) => ({ why: error.message })),
        ...asking.map((asked) =>
          get(`/choices?lend=${asked}`).then(
            /** @returns {[string, Asked]} */ ({ choices }) => [asked, { choices }],
            /** @returns {[string, Asked]} */ (error) => [asked, { why: error.message }],
          ),
        ),
      ]).then(([yeses, ...choices]) => ({
        type: 'reached',
        n: serial,
        yeses,
        choices: Object.fromEntries(choices),
        why: `${choices.length} asking`,
      })),
    yes: ({ g: group, app }) =>
      post('/runs', [group, app]).then(() => ({ type: 'acted', why: `${app} at its new version` })),
    // A refused Stop answered by its own io, with its lend and app, so the
    // step keeps the refusal where `Uses` draws it, beside that Stop
    // (ui/uses.js `said`): the `failed` message `machine` makes carries
    // neither (ui/machine.js `run`). As the phone's io answers its own
    // (web/page/uses.js `unuse`).
    stop: ({ lend, app }) =>
      post('/choices', { lend, app, said: 'stop' }).then(
        () => ({ type: 'stopped', why: `${app} stopped using ${lend}` }),
        (error) => ({ type: 'unstopped', lend, app, why: error.message }),
      ),
    again: () => {
      home({ type: 'again' });
      return { type: 'told', why: 'the groups read again' };
    },
  },
  step: (state, message) => {
    switch (message.type) {
      case 'read': {
        const [asked, serial] = ask(state, 'reaching');
        return go(
          state,
          {},
          { asked },
          { run: 'read', n: serial, asking: message.asking, failed: 'unreached' },
        );
      }
      case 'reached': {
        const old = late(state, 'reaching', message.n);
        if (old) return no(state, old);
        return go(state, { reaching: 'read' }, { yeses: message.yeses, choices: message.choices });
      }
      case 'unreached':
        return go(state, { reaching: 'unread' }, { yeses: { why: message.why }, choices: {} });
      case 'fold':
        return go(state, {}, { folds: { ...state.folds, [message.fold]: message.open } });
      case 'yes': {
        if (state.at.doing === 'busy')
          return no(state, `${state.acting} is under way, and is answered first`);
        const acting = `${message.g} ${message.app}`;
        const acted = { ...state.acted, [acting]: { ...state.acted[acting], c: message.c } };
        return go(
          state,
          { doing: 'busy' },
          { acting, acted },
          { run: 'yes', g: message.g, app: message.app, failed: 'unacted' },
        );
      }
      // A yes's answer, to the yes that set `acting` and kept it in `acted`:
      // `acting` read once, cast, where it was read at each use.
      case 'acted': {
        const acting = /** @type {string} */ (state.acting);
        return go(
          state,
          { doing: 'done' },
          { acted: { ...state.acted, [acting]: { ...state.acted[acting], state: 'done' } } },
          { run: 'again', failed: 'told' },
        );
      }
      case 'unacted': {
        const acting = /** @type {string} */ (state.acting);
        return go(
          state,
          { doing: 'refused' },
          {
            acted: {
              ...state.acted,
              [acting]: {
                ...state.acted[acting],
                state: 'refused',
                said: `${/** @type {Acted} */ (state.acted[acting]).said ?? ''} ${message.why}`,
              },
            },
          },
        );
      }
      case 'stop':
        return go(
          state,
          {},
          {},
          { run: 'stop', lend: message.lend, app: message.app, failed: 'unstopped' },
        );
      case 'stopped':
        return go(state, {}, {}, { run: 'again', failed: 'told' });
      case 'unstopped':
        return go(
          state,
          {},
          { said: { ...state.said, [`${message.lend} ${message.app}`]: message.why } },
        );
      case 'told':
        return go(state, {});
      default:
        return assertNever(state, message, 'what acts as you');
    }
  },
});
