// SPDX-License-Identifier: AGPL-3.0-or-later
// Leaving a group, in its section on the home page (page-members (d);
// home.js@19b63b2b:40-42, :145-153; groups-leave), what stays and what stops said
// before; whoever keeps the order of removals chooses, beside it, who of
// the group's other people keeps it next. Each press is LEAVE, a machine
// of the section's own, which asks the node (removal.rs) and says each
// answer after the one before, beside the button pressed, as v1's
// `p.append` does: the node's outcome (removal.rs `Outcome`, a 409 one
// too), or a refusal in its own words, its session lapsed (a 401's page,
// home.rs:198) said as group.js `get` says it, where v1 read every
// answer as JSON (home.js@19b63b2b:42) and said a parse error. A leave goes on to
// the page saying so: HOME takes `left` here (`take`), which says it and
// reads the groups again, never a reload (v1's `location.assign`). A
// hand-on reads them again too (HOME's `again`), so the hand-on, no
// longer this computer's, goes, and what it did is said beside Leave,
// where v1 went on saying "You keep the order of removals" until a reload.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, assertNever, go, machine, no } from '../ui/machine.js?v=__VERSION__';
import { called } from '../question.js?v=__VERSION__';
import { ENDED, sent } from './routes.js?v=__VERSION__';
/** @import { Group, Home, PlaceProps, Taken as Offered } from './group.js?v=__VERSION__' */
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * HOME's message this file takes: a group left, by its name.
 * @typedef {{ type: 'left', flow: 'leave', name: string }} Taken
 */

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
/**
 * @param {Home} home
 * @returns {Machine<State, Message, Effect>}
 */
const LEAVE = (home) => ({
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
        return go(
          state,
          {
            leave:
              /** @type {Partial<Record<Outcome, State['at']['leave']>>} */ ({
                handed: 'handed',
                decides: 'decides',
                left: 'idle',
              })[message.o] ?? 'refused',
          },
          message.o === 'left'
            ? {}
            : said(
                SAYS[message.o]?.(state) ?? `${state.name} cannot be changed on this computer now.`,
                message.o === 'handed' ? 'leave' : state.on,
              ),
          ...told,
        );
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
/**
 * A group's Leave, and its hand-on when this computer's person keeps its
 * order of removals: the group, its people, this computer's key, HOME's
 * `dispatch`, and the group's name as said.
 * @typedef {object} LeaveProps
 * @property {Group} g
 * @property {PeopleRow[]} rows
 * @property {string} me
 * @property {Home} home
 * @property {string} [name]
 */

/** @param {LeaveProps} props */
function Leave({ g: group, rows, me, home, name = group.name ?? 'this group' }) {
  const [state, send] = useMachine(() =>
    machine('LEAVE', LEAVE(home), { at: { leave: 'idle' }, said: [], to: 0, name }),
  );
  const own = rows.find((row) => row.key === me)?.person_key;
  // `state.to` is the chosen option's value, a string after the first choice,
  // by which an array is indexed as by its number.
  const others = [
    ...new Map(
      rows
        .filter((row) => row.device === 'computer' && row.person_key !== own)
        .map((row) => [row.person_key, row]),
    ).values(),
  ];
  const press =
    /**
     * @param {On} on
     * @param {string} path
     * @param {Body} body
     */
    (on, path, body) => () => send({ type: 'ask', on, path, body });
  const says = /** @param {On} on */ (on) =>
    state.said.filter((said) => said[0] === on).map((said) => said[1]);
  return html`${group.keeps && others.length > 0 && html`<p>You keep the order of removals in ${name}. Choose who keeps it after you, then leave: <select onChange=${(/** @type {{ target: HTMLSelectElement }} */ chosen) => send({ type: 'to', i: chosen.target.value })}>${others.map((row, i) => html`<option value=${i}>${called(rows, row.person_key)}</option>`)}</select><button onClick=${press('hand', '/v1/handovers', { group: group.id, ...others[Number(state.to)] })}>Hand it on</button>${says('hand')}</p>`}<p class="leave">Leaving ${name} stops it reaching you, and stops what you lend reaching its people. What you have of it stays on this computer, its folder too. Rejoining needs a new invitation, and this computer can’t rejoin yet. <button onClick=${press('leave', '/v1/leaves', { group: group.id })}>Leave ${name}</button>${says('leave')}</p>`;
}

// HOME's `left`: the leave said (home.js SAYS `left`), the groups read again.
// Any message to this flow is offered here (group.js `offered`), so one of
// another type is refused.
/**
 * @param {HOME.State} state
 * @param {Offered} message
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state, message, [asked, serial] = ask(state, 'groups')) =>
  message.type === 'left'
    ? go(
        state,
        { groups: 'reading', said: 'left' },
        { asked, q: { ...state.q, left: /** @type {Taken} */ (message).name } },
        { run: 'groups', n: serial, round: 'again', failed: 'unread' },
      )
    : no(state, `leaving takes no ${String(message.type)}`);
/** @type {{ leave: (props: PlaceProps) => Drawn }} */
export const view = {
  leave: ({ g: group, s: state, home, me }) =>
    state.people && html`<${Leave} g=${group} rows=${state.people.people} me=${me} home=${home} />`,
};
