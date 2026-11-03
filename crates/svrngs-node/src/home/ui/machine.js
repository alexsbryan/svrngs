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

/** The next state, `at` naming the regions that move, and the effects to run. */
export const go = (s, at, patch = {}, ...effects) => ({
  state: { ...s, ...patch, at: { ...s.at, ...at } },
  effects,
});

/** The same state, nothing run, and why: traced, never drawn over. */
export const no = (s, why) => ({ state: s, effects: [], refused: why });

/** One more read of `kind` asked: the counts to keep and this read's number. */
export const ask = (s, kind, n = (s.asked?.[kind] ?? 0) + 1) => [{ ...s.asked, [kind]: n }, n];

/** Why an answer to read `n` of `kind` is older than the last asked, or ''. */
export const late = (s, kind, n) =>
  n === s.asked?.[kind] ? '' : `${kind} ${n} answered after ${kind} ${s.asked?.[kind]} was asked`;

/** A machine named `name` (its trace's prefix), from `state`, `first` run as it starts. */
export function machine(name, { states, step, io }, state, first = []) {
  const subs = new Set(),
    regions = Object.keys(states);
  const where = (s) => regions.map((r) => s.at[r]).join('/');
  // Each effect after the step that gave it, in its order; whatever it
  // answers but a message (a rejection, a throw, no io, nothing) is its
  // `failed` message with why.
  const run = (effects) =>
    effects.forEach((f) =>
      Promise.resolve()
        .then(() => io[f.run]?.(f))
        .then(
          (m) =>
            m ??
            Promise.reject(
              new Error(io[f.run] ? `${f.run} answered nothing` : `nothing here runs ${f.run}`),
            ),
        )
        .then(dispatch, (e) => dispatch({ type: f.failed, n: f.n, why: e?.message ?? String(e) })),
    );
  // The one trace of every step and refusal, both pages', to their sink
  // (ui/trace.js), as web/store.js:88 traces to it too.
  function dispatch(m) {
    const { state: next, effects = [], refused } = step(state, m);
    const out = refused ? '' : regions.find((r) => !states[r].includes(next.at[r]));
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
