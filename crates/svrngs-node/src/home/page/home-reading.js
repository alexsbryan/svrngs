// SPDX-License-Identifier: AGPL-3.0-or-later
// HOME, the home page's machine: it reads the person's groups from
// /v1/groups, and the address once (`first`): the ask (/ask), a group on a
// screen (?screen=), or the page, with what was just done here (?made=,
// ?joined=, ?left=). It takes `again` after a write and `visible` on coming
// back on screen, and each reads the groups again. Every other message
// names the feature's file that takes it (offered.js), each a file of
// FILES.
import { ask, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { offered } from './offered.js?v=__VERSION__';
import { get } from './routes.js?v=__VERSION__';
/** @import { Machine, Step, Store } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Group as Held, GroupProps } from './group.js?v=__VERSION__' */
/** @import { Round } from './group-reading.js?v=__VERSION__' */
/** @import { Taken } from './offered.js?v=__VERSION__' */
/** @import * as acts from './acts.js?v=__VERSION__' */
/** @import * as lending from './lending.js?v=__VERSION__' */
/** @import * as join from './join.js?v=__VERSION__' */
/** @import * as make from './make.js?v=__VERSION__' */
/** @import * as leave from './leave.js?v=__VERSION__' */

/**
 * What the address says, by name (`?made=`, `?screen=`, …), and what a
 * feature's `take` writes into it: `null` for a folder that was placed.
 * @typedef {Partial<Record<string, string | null>>} Said
 */

/**
 * HOME's state: its regions, the address read once (`first`), the groups
 * and this computer's key as last read, the round of reading they began,
 * or why they could not be read; and the fields each feature's `take`
 * sets.
 * @typedef {object} Own
 * @property {{ groups: 'reading' | 'read' | 'rereading' | 'unread', said: Done }} at
 * @property {Record<string, number>} [asked]
 * @property {boolean} ask whether the address is /ask
 * @property {Said} q
 * @property {boolean} grouped layout B, `first` decides
 * @property {string} origin
 * @property {Held[]} [groups]
 * @property {string} [key]
 * @property {Round} [round]
 * @property {string} [why]
 * @typedef {Own & acts.Fields & lending.Fields} State
 */

/**
 * HOME's messages: a round of reading begun here (`again`, after a write;
 * `visible`), the groups read or unread, and each message a feature's
 * `take` takes, which names its file (`flow`).
 * @typedef {{ type: 'again' | 'visible' }
 *   | { type: 'read', n: number, groups: Held[], key: string, round: Round['type'], why: string }
 *   | { type: 'unread', n: number, why: string }
 *   | acts.Taken
 *   | lending.Taken
 *   | join.Taken
 *   | make.Taken
 *   | leave.Taken} Message
 */

/**
 * The effect HOME's step answers: the groups read, read `n`, in a round
 * of `round`.
 * @typedef {{ run: 'groups', n: number, round: Round['type'], failed: 'unread' }} Effect
 */

/**
 * A feature's module, as HOME reads it: the step that takes its messages,
 * whether it holds the page's foot while it reads, and its places' views.
 * `take` is a method, so a `take` of its own file's `Taken` stands for any.
 * @typedef {{
 *   take?(s: State, m: Taken): Step<State, Effect>,
 *   holds?(s: State): boolean,
 *   view?: Record<string, (props: any) => Drawn>,
 * }} Flow
 */

/**
 * The features by their file's name (FILES); "What acts as you" holds
 * "What you lend" while it reads.
 * @typedef {Record<(typeof FILES)[number], Flow> & { acts: { holds(s: State): boolean } }} Flows
 */

/**
 * A place of HOME's, after the groups (home-page.js PLACES), drawn with its
 * state and its `dispatch`.
 * @typedef {{ s: State, home: Store<State, Message>['dispatch'] }} PlaceProps
 */

/**
 * Each place's view, HOME's and each group section's, by its name.
 * @typedef {Partial<Record<string, (props: PlaceProps) => Drawn>> & GroupProps['places']} Places
 */

// Each feature's file in home/page/, each served by its name (home.rs
// `served`), and each a module whose `view` names the places it draws.
export const FILES = /** @type {const} */ ([
  'shelf',
  'index',
  'running',
  'lending',
  'door',
  'visits',
  'leave',
  'screen',
  'join',
  'join-form',
  'make',
  'make-form',
  'links',
  'open',
  'acts',
  'ask',
  'wifi',
  'wifi-foot',
  'play',
  'update-line',
]);

// After a write here (`again`), the groups drawn stay, said as read again
// until they are: "You made Trip." over the groups as they were looked
// done before the new one was there.
/**
 * @param {State} state
 * @param {{ type: 'again' | 'visible' }} message
 * @returns {Step<State, Effect>}
 */
const reading = (state, message) => {
  const [asked, serial] = ask(state, 'groups');
  const rereading = message.type === 'again' && state.at.groups === 'read';
  return go(
    state,
    rereading ? { groups: 'rereading' } : {},
    { asked },
    { run: 'groups', n: serial, round: message.type, failed: 'unread' },
  );
};

// The address, read once as the page starts, and what it says was done
// here.
// What the address says was done here (home-page.js `SAID`).
const DONE = /** @type {const} */ (['made', 'joined', 'left', 'stayed', 'moved', 'nowhere']);
/** @typedef {'none' | typeof DONE[number]} Done */

/**
 * @param {Location} location
 * @returns {State}
 */
export const first = ({ pathname, search, origin, hash }) => {
  const query = Object.fromEntries(new URLSearchParams(search));
  if (pathname === '/' && !query.screen && hash.startsWith('#group-'))
    query.welcome = hash.slice(7);
  const done = DONE.find((said) => said in query);
  return {
    at: { groups: 'reading', said: done ?? 'none' },
    asked: { groups: 1 },
    ask: pathname === '/ask',
    q: query,
    // The layout's one decider: `/` is B. A, `/?ui=a`, is frozen, nothing links
    // to it, and it goes within a couple of weeks of 2026-11-06.
    grouped: query.ui !== 'a',
    origin,
  };
};

// A feature's message is offered to the feature its `flow` names, as is a
// message no type declares, which `offered` refuses: each feature's `Taken`
// is that file's. A message type HOME declares that no case names fails
// `typed lint` (switch-exhaustiveness-check), and one that names no flow
// fails `types` at `offered`.
/**
 * @param {Flows} flows
 * @returns {Machine<State, Message, Effect>}
 */
export const HOME = (flows) => ({
  states: {
    groups: ['reading', 'read', 'rereading', 'unread'],
    said: ['none', ...DONE],
  },
  step: (state, message) => {
    switch (message.type) {
      case 'again':
      case 'visible':
        return reading(state, message);
      case 'read':
      case 'unread': {
        const old = late(state, 'groups', message.n);
        if (old) return no(state, old);
        if (message.type === 'unread') return go(state, { groups: 'unread' }, { why: message.why });
        return go(
          state,
          { groups: 'read' },
          { groups: message.groups, key: message.key, round: { type: message.round } },
        );
      }
      case 'made':
      case 'joined':
      case 'left':
      case 'told':
      default:
        return offered(flows, state, message);
    }
  },
  io: {
    groups: ({ n: serial, round }) =>
      get('/v1/groups').then(
        /** @param {{ groups: Held[], key: string }} answer */
        ({ groups, key }) => ({
          type: 'read',
          n: serial,
          groups,
          key,
          round,
          why: `${groups.length} group(s)`,
        }),
        (error) => ({ type: 'unread', n: serial, why: error.message }),
      ),
  },
});
