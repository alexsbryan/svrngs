// SPDX-License-Identifier: AGPL-3.0-or-later
// The pages' one core, both pages' (page-scaffold (c); svrn-docs
// rings/current/PAGE_STATE.md §3). A machine owns its page's state: views
// read it and `dispatch` a message `m`, and never write it. `step(s, m)` is
// the page's, pure and synchronous, and answers the next state and the
// effects to run, as data; the edge's `io` runs each and answers one
// message, done, or `failed` with why, so nothing idles quietly. `s.at`
// holds one named state per region, each region a closed set (`states`), and
// a step that answers outside it is refused here (AGENTS.md 9). No DOM, no
// Preact, no clock, no storage, no state of the module's own.

/**
 * What every machine's state holds beside its page's own fields: `at`, one
 * named state per region, and `asked`, the number of the last read asked of
 * each kind (`ask`).
 * @typedef {{ at: Record<string, string>, asked?: Record<string, number> }} Core
 */

/**
 * An effect, as data: `run` names the io that runs it, and `failed` the
 * message it answers when it cannot, carrying `n`, the read it answers.
 * @typedef {{ run: string, failed?: string, n?: number }} Effect
 */

/**
 * What a step answers: the next state and the effects to run, or the same
 * state, nothing to run, and why it was refused.
 * @template State
 * @template {Effect} [E=Effect]
 * @typedef {{ state: State, effects: E[], refused?: string }} Step
 */

/**
 * The io of a machine: for each effect's `run`, the one function that runs
 * it and answers one message, or a promise of one.
 * @template {Effect} E
 * @template Message
 * @typedef {{ [Run in E['run']]: (effect: Extract<E, { run: Run }>) => Message | Promise<Message> }} Io
 */

/**
 * The key a message is taken by, as web/page/page.js `compose` looks it up:
 * a part's own message (ui/parts.js: Fold's `{ type: 'fold', fold }`,
 * Field's `{ type: 'typed', field }`) by its type and its part's name,
 * `fold acts`, `typed name`; every other by its type.
 * @template {{ type: string }} M
 * @typedef {M extends { fold: string } ? `${M['type']} ${M['fold']}` : M extends { field: string } ? `${M['type']} ${M['field']}` : M['type']} TakenBy
 */

/**
 * A table of steps, as each phone flow's `takes` (web/page/page.js
 * `compose`): for each message of `Message`, by the key it is taken by
 * (`TakenBy`), the one step that takes it, given that message's own type.
 * A message with no step, or a step for no message, fails `types`: the
 * check that a table is exhaustive, as `assertNever` is a `switch`'s.
 * @template State
 * @template {{ type: string }} Message
 * @template {Effect} [E=Effect]
 * @typedef {{ [M in Message as TakenBy<M>]: (s: State, m: M) => Step<State, E> }} Takes
 */

/**
 * The io as `machine` reads it, by an effect's `run`, which may name no
 * function: the one place a run and its effect are not paired by type, so
 * its effect is `any` (`Io` pairs them for every machine that declares one).
 * @template Message
 * @typedef {Partial<Record<string, (effect: any) => Message | Promise<Message>>>} Runners
 */

/**
 * A page's machine, `Machine<State, Message>`: each region of `State['at']`
 * with its closed set of named states, `step`, pure and synchronous, and the
 * `io` that runs the effects it answers.
 * @template {Core} State
 * @template {{ type: string, why?: string }} Message
 * @template {Effect} [E=Effect]
 * @typedef {object} Machine
 * @property {{ [Region in keyof State['at']]: readonly State['at'][Region][] }} states
 * @property {(state: State, message: Message) => Step<State, E>} step
 * @property {Io<E, Message>} io
 */

/**
 * A machine running: its state now, the one way to change it, and
 * `subscribe`, which answers the way to stop.
 * @template State
 * @template Message
 * @typedef {object} Store
 * @property {() => State} get
 * @property {(message: Message) => void} dispatch
 * @property {(changed: () => void) => () => void} subscribe
 */

/**
 * The next state, `at` naming the regions that move, and the effects to run.
 * @template {Core} State
 * @template {Effect} E
 * @param {State} s
 * @param {Partial<State['at']>} at
 * @param {Partial<State>} [patch]
 * @param {...E} effects
 * @returns {Step<State, E>}
 */
export const go = (s, at, patch = {}, ...effects) => ({
  state: { ...s, ...patch, at: { ...s.at, ...at } },
  effects,
});

/**
 * The same state, nothing run, and why: traced, never drawn over.
 * @template State
 * @param {State} s
 * @param {string} why
 * @returns {Step<State, never>}
 */
export const no = (s, why) => ({ state: s, effects: [], refused: why });

/**
 * One more read of `kind` asked: the counts to keep and this read's number.
 * @param {Core} s
 * @param {string} kind
 * @param {number} [n]
 * @returns {[Record<string, number>, number]}
 */
export const ask = (s, kind, n = (s.asked?.[kind] ?? 0) + 1) => [{ ...s.asked, [kind]: n }, n];

/**
 * Why an answer to read `n` of `kind` is older than the last asked, or ''.
 * @param {Core} s
 * @param {string} kind
 * @param {number | undefined} n
 */
export const late = (s, kind, n) =>
  n === s.asked?.[kind] ? '' : `${kind} ${n} answered after ${kind} ${s.asked?.[kind]} was asked`;

/**
 * The end of a step's `switch (m.type)`, where the compiler gives `m` the
 * type `never` once every case is taken: a message type a step declares and
 * no case takes fails `types` here. A message no type foresaw, such as an
 * io's answer the page does not declare, still reaches it as the page runs,
 * and is refused with why, as every refusal is: "`taker` takes no <type>".
 * @template State
 * @param {State} s
 * @param {never} m
 * @param {string} taker
 * @returns {Step<State, never>}
 */
export const assertNever = (s, m, taker) =>
  no(s, `${taker} takes no ${/** @type {{ type: string }} */ (m).type}`);

/**
 * A machine named `name` (its trace's prefix), from `state`, `first` run as it starts.
 * @template {Core} State
 * @template {{ type: string, why?: string }} Message
 * @template {Effect} E
 * @param {string} name
 * @param {Machine<State, Message, E>} machine
 * @param {State} state
 * @param {E[]} [first]
 * @returns {Store<State, Message>}
 */
export function machine(name, { states, step, io }, state, first = []) {
  const subs = new Set(),
    regions = /** @type {(keyof State['at'] & string)[]} */ (Object.keys(states));
  /** @param {State} s */
  const where = (s) => regions.map((r) => s.at[r]).join('/');
  // Each effect after the step that gave it, in its order; whatever it
  // answers but a message (a rejection, a throw, no io, nothing) is its
  // `failed` message with why.
  /** @param {E[]} effects */
  const run = (effects) =>
    effects.forEach((f) =>
      Promise.resolve()
        .then(() => /** @type {Runners<Message>} */ (io)[f.run]?.(f))
        .then(
          (m) =>
            m ??
            Promise.reject(
              new Error(
                /** @type {Runners<Message>} */ (io)[f.run]
                  ? `${f.run} answered nothing`
                  : `nothing here runs ${f.run}`,
              ),
            ),
        )
        // Its `failed` message, which no type here can show its machine's
        // Message declares: a step that does not take it refuses it, by name.
        .then(dispatch, (e) =>
          dispatch(
            /** @type {Message} */ (
              /** @type {unknown} */ ({ type: f.failed, n: f.n, why: e?.message ?? String(e) })
            ),
          ),
        ),
    );
  // The one trace of every step and refusal, both pages', to their sink
  // (ui/trace.js), as web/store.js:88 traces to it too.
  /** @param {Message} m */
  function dispatch(m) {
    const { state: next, effects = [], refused } = step(state, m);
    const out = refused
      ? ''
      : regions.find((r) => !(/** @type {readonly unknown[]} */ (states[r]).includes(next.at[r])));
    const why =
      refused || (out && `${out} ${next.at[out]} is not one of ${states[out].join(', ')}`);
    if (why) return globalThis.__trace?.(`${name}: ${m.type} refused in ${where(state)}: ${why}`);
    const running = effects.length ? `, running ${effects.map((f) => f.run).join(', ')}` : '';
    globalThis.__trace?.(
      `${name}: ${m.type}${m.why ? ` (${m.why})` : ''}: ${where(state)} -> ${where(next)}${running}`,
    );
    if (next !== state) {
      state = next;
      subs.forEach((f) => f());
    }
    run(effects);
  }
  run(first);
  return { get: () => state, dispatch, subscribe: (f) => (subs.add(f), () => subs.delete(f)) };
}
