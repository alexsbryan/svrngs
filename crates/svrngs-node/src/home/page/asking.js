// SPDX-License-Identifier: AGPL-3.0-or-later
// ASK, the machine of the ask at /ask (asked.js `Asked`): what the people
// of each group you are in lend, read, then whether the lend and the app
// the address names are both lent, and the lend asks. Yes posts the yes to
// /choices, then leaves for the app at `back` through /to, as Not now
// leaves at once, by location.assign at the edge. A refusal is said after
// the buttons, in place of any before it.
import { assertNever, go, no } from '../ui/machine.js?v=__VERSION__';
import { label } from '../question.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Group, Lent } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Said } from './home-reading.js?v=__VERSION__' */

/**
 * The ask's state: where it is, what was last said of it, and, once the
 * lends are read and both are lent, the lend asked for and the app asking.
 * @typedef {object} State
 * @property {{ ask: 'reading' | 'asking' | 'sending' | 'unlent' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} said
 * @property {Lent} [lend]
 * @property {Lent} [app]
 */

/**
 * The messages the ask takes: the lends read, none when they could not be
 * (its read's `failed`); Yes or Not now pressed; the yes not said, with
 * why; and the page gone to the app.
 * @typedef {{ type: 'lent', lent?: Lent[], why: string }
 *   | { type: 'yes' }
 *   | { type: 'not' }
 *   | { type: 'unsaid', why: string }
 *   | { type: 'gone', why: string }} Message
 */

/**
 * The effects its step answers: the groups' lends read, the yes posted
 * then the app gone to, and the app gone to.
 * @typedef {{ run: 'read', groups: Group[], failed: 'lent' }
 *   | { run: 'yes', lend: Lent, app: Lent, failed: 'unsaid' }
 *   | { run: 'leave', app: Lent, failed: 'unsaid' }} Effect
 */

// A group's lends, none when they cannot be read, or when who is in it
// cannot be, so one group's refusal leaves the others' lends asked.
/**
 * @param {Group[]} groups
 * @returns {Promise<Lent[]>}
 */
const lentIn = (groups) => {
  const reads = groups.map((group) => {
    if (group.removed !== false) return Promise.resolve([]);
    const namespace = encodeURIComponent(group.id);
    return get(`/v1/lends?namespace=${namespace}`).then(
      (lend) => lend.lent,
      () => [],
    );
  });
  return Promise.all(reads).then((answers) => answers.flat());
};

// The app at `back`, kept only on its own origin, as open.js judges its
// way back, so an ask cannot send the person to another site.
/**
 * @param {Lent} app
 * @param {string | null | undefined} back
 * @returns {Extract<Message, { type: 'gone' }>}
 */
const leave = (app, back) => {
  const url = new URL(back ?? '/', app.origin);
  const at = url.origin === new URL(app.origin).origin ? url.pathname + url.search + url.hash : '/';
  location.assign(`/to?${new URLSearchParams({ lend: label(app), at })}`);
  return { type: 'gone', why: `to ${label(app)}` };
};

/**
 * ASK, for the ask the address names (home-reading.js `Said`).
 * @param {Said} asked
 * @returns {Machine<State, Message, Effect>}
 */
export const ASK = (asked) => ({
  states: { ask: ['reading', 'asking', 'sending', 'unlent'] },
  io: {
    read: ({ groups }) =>
      lentIn(groups).then((lent) => ({ type: 'lent', lent, why: `${lent.length} lent` })),
    yes: ({ lend, app }) =>
      post('/choices', { lend: label(lend), app: label(app), said: 'yes' }).then(() =>
        leave(app, asked.back),
      ),
    leave: ({ app }) => leave(app, asked.back),
  },
  // `lend` and `app` are set together, by `lent`, and only then is the
  // ask `asking`, or `sending` past it, so each step that reads them reads
  // them set.
  step: (state, message) => {
    switch (message.type) {
      case 'lent': {
        const lent = message.lent ?? [];
        const lend = lent.find((each) => label(each) === asked.lend);
        const app = lent.find((each) => label(each) === asked.app);
        if (!lend || !app || !lend.asks) return go(state, { ask: 'unlent' });
        return go(state, { ask: 'asking' }, { lend, app });
      }
      case 'yes':
        if (state.at.ask !== 'asking') return no(state, `the ask is ${state.at.ask}`);
        return go(
          state,
          { ask: 'sending' },
          {},
          {
            run: 'yes',
            lend: /** @type {Lent} */ (state.lend),
            app: /** @type {Lent} */ (state.app),
            failed: 'unsaid',
          },
        );
      case 'not':
        if (!state.lend) return no(state, 'nothing is asked');
        return go(
          state,
          {},
          {},
          { run: 'leave', app: /** @type {Lent} */ (state.app), failed: 'unsaid' },
        );
      case 'unsaid':
        return go(state, { ask: 'asking' }, { said: message.why });
      case 'gone':
        return go(state, {});
      default:
        return assertNever(state, message, 'the ask');
    }
  },
});
