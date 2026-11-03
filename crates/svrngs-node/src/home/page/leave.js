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
import {
  ask,
  assertNever,
  called,
  go,
  html,
  machine,
  no,
  useMachine,
} from '../ui/kit.js?v=__VERSION__';
/** @import { Group, Home, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Row } from '../question.js?v=__VERSION__' */
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
  handed: (s) => `${s.person} keeps the order of removals in ${s.name} now, so you can leave it.`,
  decides: (s) =>
    `You keep the order of removals in ${s.name}: choose who keeps it after you, then leave.`,
  not_in: (s) => `You are no longer in ${s.name}.`,
};
/** @type {Partial<Record<Outcome, (s: State) => HOME.Message>>} */
const TELLS = {
  left: (s) => ({ type: 'left', flow: 'leave', name: s.name }),
  handed: () => ({ type: 'again' }),
};
/**
 * @param {Home} home
 * @returns {Machine<State, Message, Effect>}
 */
const LEAVE = (home) => ({
  states: { leave: ['idle', 'asking', 'handed', 'decides', 'refused'] },
  step: (s, m) => {
    /**
     * An answer said after the one before, beside the button `on`.
     * @param {string} w
     * @returns {Pick<State, 'said'>}
     */
    const said = (w, on = s.on) => ({ said: [...s.said, [on, ` ${w}`]] });
    switch (m.type) {
      case 'to':
        return go(s, {}, { to: m.i });
      case 'ask':
        return go(
          s,
          { leave: 'asking' },
          { on: m.on, person: m.body.person },
          { run: 'ask', path: m.path, body: m.body, failed: 'refused' },
        );
      case 'answered': {
        // What HOME is told, read once, as tsc does not narrow a lookup
        // by a key read twice.
        const tell = TELLS[m.o];
        /** @type {Effect[]} */
        const told = tell ? [{ run: 'send', m: tell(s) }] : [];
        return go(
          s,
          {
            leave:
              /** @type {Partial<Record<Outcome, State['at']['leave']>>} */ ({
                handed: 'handed',
                decides: 'decides',
                left: 'idle',
              })[m.o] ?? 'refused',
          },
          m.o === 'left'
            ? {}
            : said(
                SAYS[m.o]?.(s) ?? `${s.name} cannot be changed on this computer now.`,
                m.o === 'handed' ? 'leave' : s.on,
              ),
          ...told,
        );
      }
      case 'refused':
        return go(
          s,
          { leave: 'refused' },
          said(`${s.name} cannot be changed on this computer now: ${m.why}`),
        );
      case 'sent':
        return go(s, {});
      default:
        return assertNever(s, m, 'leaving');
    }
  },
  io: {
    ask: ({ path, body }) =>
      fetch(path, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(body),
      }).then(async (r) =>
        ((a) =>
          a.outcome
            ? { type: 'answered', o: a.outcome, why: a.outcome }
            : Promise.reject(
                new Error(
                  r.status === 401
                    ? 'this page’s session has ended. Open svrngs from your Applications, or on Linux your applications menu, and it opens here again'
                    : a.error || `answered ${r.status}`,
                ),
              ))(await r.json().catch(() => ({}))),
      ),
    send: ({ m }) => (home(m), { type: 'sent', why: m.type }),
  },
});
/**
 * A group's Leave, and its hand-on when this computer's person keeps its
 * order of removals: the group, its people, this computer's key, HOME's
 * `dispatch`, and the group's name as said.
 * @typedef {object} LeaveProps
 * @property {Group} g
 * @property {Row[]} rows
 * @property {string} me
 * @property {Home} home
 * @property {string} [name]
 */

/** @param {LeaveProps} props */
function Leave({ g, rows, me, home, name = g.name ?? 'this group' }) {
  const [s, send] = useMachine(() =>
      machine('LEAVE', LEAVE(home), { at: { leave: 'idle' }, said: [], to: 0, name }),
    ),
    own = rows.find((r) => r.key === me)?.person_key;
  // `s.to` is the chosen option's value, a string after the first choice,
  // by which an array is indexed as by its number.
  const others = [
    ...new Map(
      rows
        .filter((r) => r.device === 'computer' && r.person_key !== own)
        .map((r) => [r.person_key, r]),
    ).values(),
  ];
  const press =
      /**
       * @param {On} on
       * @param {string} path
       * @param {Body} body
       */
      (on, path, body) => () => send({ type: 'ask', on, path, body }),
    says = /** @param {On} on */ (on) => s.said.filter((w) => w[0] === on).map((w) => w[1]);
  return html`${g.keeps && others.length > 0 && html`<p>You keep the order of removals in ${name}. Choose who keeps it after you, then leave: <select onChange=${(/** @type {{ target: HTMLSelectElement }} */ e) => send({ type: 'to', i: e.target.value })}>${others.map((r, i) => html`<option value=${i}>${called(rows, r.person_key)}</option>`)}</select><button onClick=${press('hand', '/v1/handovers', { group: g.id, ...others[Number(s.to)] })}>Hand it on</button>${says('hand')}</p>`}<p class="leave">Leaving ${name} stops it reaching you, and stops what you lend reaching its people. What you have of it stays on this computer, its folder too. Rejoining needs a new invitation, and this computer can’t rejoin yet. <button onClick=${press('leave', '/v1/leaves', { group: g.id })}>Leave ${name}</button>${says('leave')}</p>`;
}

// HOME's `left`: the leave said (home.js SAYS `left`), the groups read again.
/**
 * @param {HOME.State} s
 * @param {Taken} m
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (s, m, [asked, n] = ask(s, 'groups')) =>
  m.type === 'left'
    ? go(
        s,
        { groups: 'reading', said: 'left' },
        { asked, q: { ...s.q, left: m.name } },
        { run: 'groups', n, round: 'again', failed: 'unread' },
      )
    : no(s, `leaving takes no ${m.type}`);
/** @type {{ leave: (props: PlaceProps) => Drawn }} */
export const view = {
  leave: ({ g, s, home, me }) =>
    s.people && html`<${Leave} g=${g} rows=${s.people.people} me=${me} home=${home} />`,
};
