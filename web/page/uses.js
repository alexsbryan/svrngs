// SPDX-License-Identifier: AGPL-3.0-or-later
// What apps use, on the phone: `details#uses` over `ul#used`, folded
// (ui/parts.js `Fold`, `fold uses`), drawn under "Opens friends’ things as
// you" (acts.js) once a lend asks (what-apps-use.js), else hidden in its
// place (uses-hidden.js). Opened, it asks each lender that asks for its
// choices, each read counted, so an older answer is refused by name.
// `asking` draws #used empty, so every line is an answer drawn after it
// was opened (web/tests/tabs.mjs). Each yes has Stop, the person's stop
// posted to its lender's door, and then it asks again; a refusal is said
// after the button.
import { ask, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { silent, unreached } from '../question.js?v=__VERSION__';
import { dialled, exchange, saying, trace } from './dial.js?v=__VERSION__';
import { notLent } from './page.js?v=__VERSION__';
import { UsesHidden } from './uses-hidden.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Asked } from '../ui/uses.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Message as PageMessage, Props, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): whether the fold is open, each
 * lender's choices as read, by the lend's label, and why a stop was not
 * kept, by the lend's label and the app's.
 * @typedef {object} Fields
 * @property {boolean} [using]
 * @property {Record<string, Asked>} [choices]
 * @property {Record<string, string>} [unstopped]
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'fold', fold: 'uses', open: boolean }
 *   | { type: 'chosen', n?: number, lend: string, choices?: Asked['choices'], why: string }
 *   | { type: 'stop', lend: string, app: string }
 *   | { type: 'unused' }
 *   | { type: 'unstopped', of: string, said?: string, why?: string }} Message
 */

/**
 * The effects its steps answer: a lender's choices asked, the `n`th read
 * of them; and the person's stop of `app` using `l`, posted to its door.
 * @typedef {{ run: 'choices', group: Group, l: Lent, n: number, failed: string }
 *   | { run: 'unuse', group: Group, l: Lent, app: string, failed: string }} Effect
 */

export const owns = ['using', 'choices', 'unstopped'];
export const states = /** @type {const} */ ({ uses: ['shut', 'asking', 'asked'] });
/**
 * The fold opened, or a stop kept: each lender that asks asked for its
 * choices again, a new read, and #used drawn empty until they answer.
 * @param {State} state
 * @param {Partial<State>} [patch]
 */
const asking = (state, patch = {}) => {
  const [asked, serial] = ask(state, 'uses');
  // Folded open only where a lend asks (what-apps-use.js), so once the
  // list is read.
  const lent = /** @type {Lent[]} */ (state.lent);
  const reads = lent
    .filter((lend) => lend.asks)
    .map(
      (lend) =>
        /** @satisfies {Effect} */ ({
          run: 'choices',
          group: state.group,
          l: lend,
          n: serial,
          failed: 'chosen',
        }),
    );
  return go(state, { uses: 'asking' }, { ...patch, asked, choices: {} }, ...reads);
};

/** @type {Takes<State, Message, Effect>} */
export const takes = {
  'fold uses': (state, message) =>
    message.open ? asking(state, { using: true }) : go(state, { uses: 'shut' }, { using: false }),
  chosen: (state, message) => {
    const older = late(state, 'uses', message.n);
    if (older) return no(state, older);
    const answer = message.choices ? { choices: message.choices } : { why: message.why };
    const choices = { ...state.choices, [message.lend]: answer };
    // Asked of each lend that asks (`asking`), so the list is read.
    const lent = /** @type {Lent[]} */ (state.lent);
    const all = lent.every((lend) => !lend.asks || choices[lend.label] !== undefined);
    return go(state, { uses: all ? 'asked' : 'asking' }, { choices });
  },
  stop: (state, { lend, app }) => {
    const found = state.lent?.find((one) => one.label === lend);
    if (!found) return notLent(state, lend);
    return go(
      state,
      {},
      {},
      { run: 'unuse', group: state.group, l: found, app, failed: 'unstopped' },
    );
  },
  unused: (state) => asking(state),
  unstopped: (state, message) => {
    const before = state.unstopped?.[message.of] ?? '';
    const said = `${before}${message.said ?? message.why}`;
    return go(state, {}, { unstopped: { ...state.unstopped, [message.of]: said } });
  },
};

/** @type {Record<string, (props: Props) => Drawn>} */
export const view = { uses: UsesHidden };

// The edge: a lender's door dialled as the person, and its words when it
// does not answer or refuses. `exchange` reads the answer whole before the
// dial closes (dial.js).
/**
 * @param {Group} group
 * @param {Lent} lend
 * @param {{ app: string, said: string }} [body] posted, else the choices are asked
 * @param {string} [away] said after a dial not answered
 */
const door = (group, lend, body, away = '') =>
  dialled(group, lend)
    .catch((error) => {
      trace(`PAGE: the dial failed: ${error}`);
      const quiet = `${silent(`${lend.person}’s computer`)}${away}`;
      return Promise.reject(saying(unreached(error, quiet)));
    })
    .then(async (connection) => {
      const bytes = body && new TextEncoder().encode(JSON.stringify(body));
      const [ok, status, text] = await exchange(connection, `/choices/${lend.label}`, bytes);
      if (ok) return JSON.parse(text || 'null');
      trace(`PAGE: /choices/${lend.label} answered ${status}: ${text}`);
      return Promise.reject(
        saying(body ? `${lend.person}’s computer did not keep your answer.` : `answered ${status}`),
      );
    });
// The person's answer `said` for the app labelled `app` on `lend`, for the ask too.
/**
 * @param {Group} group
 * @param {Lent} lend
 * @param {string} app
 * @param {'yes' | 'stop'} said
 */
export const choose = (group, lend, app, said) =>
  door(group, lend, { app, said }, ' Try again later.');
/** @type {Io<Effect, PageMessage>} */
export const io = {
  choices: ({ group, l: lend, n: serial }) =>
    door(group, lend).then(
      (answer) => ({
        type: 'chosen',
        n: serial,
        lend: lend.label,
        choices: answer.choices,
        why: `${lend.name}: ${Object.keys(answer.choices).length} app(s)`,
      }),
      (error) => ({ type: 'chosen', n: serial, lend: lend.label, why: error.message }),
    ),
  unuse: ({ group, l: lend, app }) =>
    choose(group, lend, app, 'stop').then(
      () => ({ type: 'unused', why: `${app} stopped using ${lend.name}` }),
      (error) => ({
        type: 'unstopped',
        of: `${lend.label} ${app}`,
        said: error.message,
        why: error.message,
      }),
    ),
};
