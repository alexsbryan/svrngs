// SPDX-License-Identifier: AGPL-3.0-or-later
// GROUP, the machine of one group's section (group-read.js), one per group.
// It reads the group's people and what they lend, each a region of its
// own, counted, so an older answer is refused by name. Both are asked at
// once, and the people, answered first, are drawn without waiting for the
// lends, so a group shows before its slowest computer answers. It reads
// again on HOME's `again` and `visible`, each a new `round` from HOME.
import { ask, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { offered } from './offered.js?v=__VERSION__';
import { get } from './routes.js?v=__VERSION__';
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Flows } from './offered.js?v=__VERSION__' */
/** @import { Lends, People } from './group.js?v=__VERSION__' */

/** @typedef {'people' | 'lends'} Kind */

/** @typedef {'reading' | 'read' | 'unread'} Reads */

/**
 * GROUP's state: each read's region, the group's id, and what each read
 * answered last, or why it could not.
 * @typedef {object} State
 * @property {{ people: Reads, lends: Reads }} at
 * @property {Record<string, number>} [asked]
 * @property {string} id
 * @property {People} [people]
 * @property {Lends} [lends]
 * @property {string} [why]
 */

/**
 * A round of reading, by HOME's message that began it (home-reading.js `reading`).
 * @typedef {{ type: 'read' | 'again' | 'visible' }} Round
 */

/**
 * A read answered: the people, or what they lend.
 * @typedef {{ type: 'answered', kind: 'people', n: number, body: People, why: string }
 *   | { type: 'answered', kind: 'lends', n: number, body: Lends, why: string }} Answered
 */

/**
 * The messages GROUP takes: a round, each read answered, or unread with why.
 * @typedef {Round | Answered | { type: 'unread', kind: Kind, n: number, why: string }} Message
 */

/**
 * The effect its step answers: a read of `kind` for the group `id`.
 * @typedef {{ run: 'get', kind: Kind, n: number, id: string, failed: 'unread' }} Effect
 */

const ROUTES = { people: '/v1/rail/people', lends: '/v1/lends' };

// Both read, each counted (ui/machine.js `ask`), whichever round began it.
/**
 * @param {State} state
 * @returns {Step<State, Effect>}
 */
function reading(state) {
  const [peopleAsked, people] = ask(state, 'people');
  const [asked, lends] = ask({ asked: peopleAsked }, 'lends');
  /** @type {[Kind, number][]} */
  const reads = [
    ['people', people],
    ['lends', lends],
  ];
  const effects = reads.map(
    ([kind, serial]) =>
      /** @type {Effect} */ ({ run: 'get', kind, n: serial, id: state.id, failed: 'unread' }),
  );
  return go(state, { people: 'reading', lends: 'reading' }, { asked }, ...effects);
}

// A message no case takes is offered to the feature it names, as HOME's
// are: none sent to GROUP names one, so `message` there is `never`, and a
// message type GROUP declares and no case takes fails `types` at `offered`.
/**
 * @param {Flows<State, Effect>} flows
 * @returns {Machine<State, Message, Effect>}
 */
export const GROUP = (flows) => ({
  states: { people: ['reading', 'read', 'unread'], lends: ['reading', 'read', 'unread'] },
  step: (state, message) => {
    switch (message.type) {
      case 'read':
      case 'again':
      case 'visible':
        return reading(state);
      case 'answered':
      case 'unread': {
        const old = late(state, message.kind, message.n);
        if (old) return no(state, old);
        const read = message.type === 'answered';
        return go(
          state,
          /** @type {Partial<State['at']>} */ ({ [message.kind]: read ? 'read' : 'unread' }),
          /** @type {Partial<State>} */ (
            message.type === 'answered' ? { [message.kind]: message.body } : { why: message.why }
          ),
        );
      }
      default:
        return offered(flows, state, message);
    }
  },
  io: {
    get: ({ kind, n: serial, id }) =>
      get(`${ROUTES[kind]}?namespace=${encodeURIComponent(id)}`).then(
        (body) => ({ type: 'answered', kind, n: serial, body, why: `${kind} ${serial}` }),
        (error) => ({ type: 'unread', kind, n: serial, why: error.message }),
      ),
  },
});
