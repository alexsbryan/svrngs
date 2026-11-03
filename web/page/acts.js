// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the phone, drawn in #acts once the group is read
// (what-acts-as-you.js), and the acts it takes: a yes to an app's new
// version, a Stop of what a lend opens as you, and the carry of lines
// unsent.
// DOING is one act at a time, keyed by what it acts on, `acting`; each
// act's answer kept in `acted` by that key, so a yes said "opens at its
// new version for you now" stays said once the group, read again, lists
// the change no more. A yes is this phone's own: the computer it reaches
// names the act at /runs, and the phone signs it with its own key as its
// chain's next line, kept with the group before it is sent, so no seq is
// signed twice, then carries every line not yet taken. A Stop is kept
// with the group. After either, enter is given the group as kept, and
// reads it again; the key stays the edge's.
import { html } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { through, wasm } from './dial.js?v=__VERSION__';
import { WhatActsAsYou } from './what-acts-as-you.js?v=__VERSION__';
/** @import { Group, Line, Said } from '../store.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import { Io, Step, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Flow, Message as PageMessage, Send, State } from './page.js?v=__VERSION__' */

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
const acting = (state, key, effect, patch = {}) => {
  if (state.at.doing === 'busy') {
    return no(state, `${state.acting} is under way, and is answered first`);
  }
  const acted = { ...state.acted, [key]: { ...state.acted?.[key], ...patch } };
  return go(state, { doing: 'busy' }, { acting: key, acted }, effect);
};
/**
 * The act under way answered: done, or refused with why, which is said
 * after what it said before. Given the group as kept, enter reads it again.
 * @param {State} current
 * @param {Extract<Message, { type: 'acted' | 'unacted' }>} message
 * @param {'done' | 'refused'} state
 * @returns {Step<State, Effect | Send>}
 */
const answered = (current, message, state) => {
  if (current.at.doing !== 'busy') return no(current, 'no act is under way');
  // Busy, an act is under way, so `acting` names it and `acted` holds it (`acting`).
  const key = /** @type {string} */ (current.acting);
  const was = /** @type {Acted} */ (/** @type {Record<string, Acted>} */ (current.acted)[key]);
  const said = state === 'refused' ? `${was.said ?? ''} ${message.why}` : '';
  const acted = { ...current.acted, [key]: { ...was, state, said } };
  if (!message.group) return go(current, { doing: state }, { acted });
  /** @type {Send} */
  const reread = {
    run: 'send',
    m: { type: 'opened', group: message.group, why: 'kept here, so read again' },
  };
  return go(current, { doing: state }, { acted }, reread);
};
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
  yes: (state, { app }) => {
    const group = state.group;
    const taken = state.changed?.find((one) => one.group === group.id && one.app === app);
    return acting(
      state,
      `yes ${app}`,
      { run: 'runs', group, app, failed: 'unacted' },
      { c: taken },
    );
  },
  'stop reads': (state, { label }) =>
    acting(state, `stop ${label}`, {
      run: 'stopping',
      group: state.group,
      label,
      failed: 'unacted',
    }),
  acted: (state, message) => answered(state, message, 'done'),
  unacted: (state, message) => answered(state, message, 'refused'),
  carry: (state) => {
    const unsent = /** @type {Partial<State>} */ (state).group?.unsent?.length ?? 0;
    if (unsent === 0) return no(state, 'no line is unsent');
    return acting(state, 'carry', { run: 'carrying', group: state.group, failed: 'unacted' });
  },
};

/** @type {Required<Flow>['view']} */
export const view = {
  acts: ({ s: state, send }) => {
    // Drawn before a group is held too, when `group` is absent (page.js
    // `first`), as web/index.html's empty #acts.
    if (!(/** @type {Partial<State>} */ (state).group)) return html`<section id="acts"></section>`;
    return html`<${WhatActsAsYou} s=${state} send=${send} />`;
  },
};

// The edge: the key unsealed here, never in state, and the wasm imported
// only once app.js has run its init.
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
async function carry(group, why, kept = group) {
  try {
    for (const line of /** @type {Line[]} */ (group.unsent)) {
      await through(group, '/carry', line);
    }
    const opened = await store.opened({ ...group, unsent: [] });
    return { type: 'acted', group: opened, why: `${why}, and carried` };
  } catch (/** @type {any} */ error) {
    return { type: 'unacted', group: kept, why: error.message };
  }
}
/**
 * The yes to `app`'s new version: the act named at the computer, signed
 * here as the chain's next line, kept as unsent, then carried.
 * @param {Group} group
 * @param {string} app
 * @returns {Promise<PageMessage>}
 */
async function running(group, app) {
  const [core, key, { sign }] = await Promise.all([
    wasm(),
    store.unseal(group),
    through(group, '/runs', app),
  ]);
  const at = group.line ?? { seq: -1, id: null };
  const now = Math.floor(Date.now() / 1000);
  const signed = core.sign_line(key, group.id, now, at.seq + 1, at.id, JSON.stringify(sign));
  const line = JSON.parse(signed);
  const carried = await store.opened({
    ...group,
    line: { seq: line.seq, id: line.id },
    unsent: [...(group.unsent ?? []), line],
  });
  return carry(carried, `line ${line.seq} signed`);
}
/** @type {Io<Effect, PageMessage>} */
export const io = {
  runs: ({ group, app }) => running(group, app),
  carrying: ({ group }) =>
    carry(group, `${/** @type {Line[]} */ (group.unsent).length} line(s) unsent`, null),
  stopping: async ({ group, label }) => {
    /** @type {Said} */
    const stop = { said: 'stop', at: Math.floor(Date.now() / 1000) };
    const opened = await store.opened({ ...group, reads: { ...group.reads, [label]: stop } });
    return { type: 'acted', group: opened, why: `${label}'s reads stopped here` };
  },
};
