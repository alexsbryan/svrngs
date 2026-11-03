// SPDX-License-Identifier: AGPL-3.0-or-later
// LEAVE, the machine of leaving a group (leave-group.js `Leave`): each
// press asks the node (removal.rs) and each answer is said after the one
// before, beside the button pressed: the node's outcome (removal.rs
// `Outcome`, a 409's too), or a refusal in its own words, a lapsed
// session (a 401's page, home.rs:198) said as routes.js says it, since
// that page is not JSON. A leave goes on to the page saying so: HOME takes
// `left` (leave.js `take`), which says it and reads the groups again. A
// hand-on reads them again too (HOME's `again`), so the hand-on, no longer
// this computer's, goes, and what it did is said beside Leave.
import { assertNever, go } from '../ui/machine.js?v=__VERSION__';
import { ENDED, sent } from './routes.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * The node's answer to a leave or a hand-on (removal.rs `Outcome`, by its
 * `outcome`).
 * @typedef {'out' | 'asked' | 'handed' | 'left' | 'not_in_group' | 'which' | 'decides'
 *   | 'own' | 'not_decider' | 'not_decider_unread' | 'not_in' | 'fault'} Outcome
 */

/** @typedef {'hand' | 'leave'} On which button was pressed: Hand it on, or Leave */

/**
 * What a press posts: the group, and for a hand-on, the people row of whom
 * it goes to.
 * @typedef {{ group: string, person?: string }} Body
 */

/**
 * The section's state: where its leave is, each answer said beside the
 * button it answers, the hand-on's choice by its place in the list, the
 * group's name as said, the button last pressed and whom it hands on to.
 * @typedef {object} State
 * @property {{ leave: 'idle' | 'asking' | 'handed' | 'decides' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {[on: On | undefined, words: string][]} said
 * @property {number | string} to `0`, then the chosen `option`'s value
 * @property {string} name
 * @property {On} [on]
 * @property {string} [person]
 */

/**
 * The messages the section takes: whom to hand on to chosen; a button
 * pressed; the node's answer, or a refusal, with why; and HOME told.
 * @typedef {{ type: 'to', i: string }
 *   | { type: 'ask', on: On, path: string, body: Body }
 *   | { type: 'answered', o: Outcome, why: string }
 *   | { type: 'refused', why: string }
 *   | { type: 'sent', why: string }} Message
 */

/**
 * The effects its step answers: the press posted to `path`, and HOME told
 * `m`.
 * @typedef {{ run: 'ask', path: string, body: Body, failed: 'refused' }
 *   | { run: 'send', m: HOME.Message }} Effect
 */

// What is said of an outcome, and what HOME is told of it. Two statements,
// as tsc gives one statement's type to each of its declarations.
/** @type {Partial<Record<Outcome, (s: State) => string>>} */
const SAYS = {
  handed: (state) =>
    `${state.person} keeps the order of removals in ${state.name} now, so you can leave it.`,
  decides: (state) =>
    `You keep the order of removals in ${state.name}: choose who keeps it after you, then leave.`,
  not_in: (state) => `You are no longer in ${state.name}.`,
};
/** @type {Partial<Record<Outcome, (s: State) => HOME.Message>>} */
const TELLS = {
  left: (state) => ({ type: 'left', flow: 'leave', name: state.name }),
  handed: () => ({ type: 'again' }),
};
// Where an outcome leaves the section; any other is a refusal.
/** @type {Partial<Record<Outcome, State['at']['leave']>>} */
const AFTER = {
  handed: 'handed',
  decides: 'decides',
  left: 'idle',
};

/**
 * LEAVE, telling HOME through `home`.
 * @param {Home} home
 * @returns {Machine<State, Message, Effect>}
 */
export const LEAVE = (home) => ({
  states: { leave: ['idle', 'asking', 'handed', 'decides', 'refused'] },
  step: (state, message) => {
    /**
     * An answer said after the one before, beside the button `on`.
     * @param {string} words
     * @returns {Pick<State, 'said'>}
     */
    const said = (words, on = state.on) => ({ said: [...state.said, [on, ` ${words}`]] });
    switch (message.type) {
      case 'to':
        return go(state, {}, { to: message.i });
      case 'ask':
        return go(
          state,
          { leave: 'asking' },
          { on: message.on, person: message.body.person },
          { run: 'ask', path: message.path, body: message.body, failed: 'refused' },
        );
      case 'answered': {
        // What HOME is told, read once, as tsc does not narrow a lookup
        // by a key read twice.
        const tell = TELLS[message.o];
        /** @type {Effect[]} */
        const told = tell ? [{ run: 'send', m: tell(state) }] : [];
        const leave = AFTER[message.o] ?? 'refused';
        if (message.o === 'left') return go(state, { leave }, {}, ...told);
        const words =
          SAYS[message.o]?.(state) ?? `${state.name} cannot be changed on this computer now.`;
        // A hand-on's answer is said beside Leave, since the hand-on goes
        // once the groups are read again.
        const on = message.o === 'handed' ? 'leave' : state.on;
        return go(state, { leave }, said(words, on), ...told);
      }
      case 'refused':
        return go(
          state,
          { leave: 'refused' },
          said(`${state.name} cannot be changed on this computer now: ${message.why}`),
        );
      case 'sent':
        return go(state, {});
      default:
        return assertNever(state, message, 'leaving');
    }
  },
  io: {
    // An outcome is read whatever the answer's status; with none, the
    // refusal is said.
    ask: async ({ path, body }) => {
      const { response, answer } = await sent(path, body);
      /** @type {Outcome | undefined} */
      const outcome = answer.outcome;
      if (outcome) return { type: 'answered', o: outcome, why: outcome };
      if (response.status === 401) throw new Error(ENDED);
      throw new Error(answer.error || `answered ${response.status}`);
    },
    send: ({ m: message }) => {
      home(message);
      return { type: 'sent', why: message.type };
    },
  },
});
