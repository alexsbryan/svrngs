// SPDX-License-Identifier: AGPL-3.0-or-later
// What apps use, on the phone (page-acts (b); web/app.js `uses` and
// `choose`, :286-296, :327-344): `details#uses` over `ul#used`, folded
// (ui/parts.js `Fold`, `fold uses`), drawn under "Opens friends’ things as
// you" (web/page/acts.js) once a lend asks, else hidden in its place, as
// web/index.html@19b63b2b:97 has it. Opened, it asks each lender that asks for its
// choices, each read counted, so an older answer is refused by name, and
// `asking` draws #used empty, so every line is an answer drawn after it
// was opened (web/tests/tabs.mjs). Each yes has Stop, the person's stop
// posted to its lender's door, and then it asks again; a refusal is said
// after the button. Lines by ui/uses.js `Uses`, as `li`. The dial is the
// one endpoint's (web/page/enter.js `dialled`), at the edge.
import { html } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { Fold } from '../ui/parts.js?v=__VERSION__';
import { Uses } from '../ui/uses.js?v=__VERSION__';
import { dialled } from './enter.js?v=__VERSION__';
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
 * @param {State} state
 * @param {Partial<State>} [patch]
 */
const asking = (state, patch = {}, [asked, serial] = ask(state, 'uses')) =>
  go(
    state,
    { uses: 'asking' },
    { ...patch, asked, choices: {} },
    // Folded open only where a lend asks (`view`), so once the list is read.
    .../** @type {Lent[]} */ (state.lent)
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
      ),
  );
/** @type {Takes<State, Message, Effect>} */
export const takes = {
  'fold uses': (state, message) =>
    message.open ? asking(state, { using: true }) : go(state, { uses: 'shut' }, { using: false }),
  chosen: (
    state,
    message,
    choices = {
      ...state.choices,
      [message.lend]: message.choices ? { choices: message.choices } : { why: message.why },
    },
  ) =>
    late(state, 'uses', message.n)
      ? no(state, late(state, 'uses', message.n))
      : // Asked of each lend that asks (`asking`), so the list is read.
        go(
          state,
          {
            uses: /** @type {Lent[]} */ (state.lent).every(
              (lend) => !lend.asks || choices[lend.label],
            )
              ? 'asked'
              : 'asking',
          },
          { choices },
        ),
  stop: (state, { lend, app }, found = state.lent?.find((one) => one.label === lend)) =>
    !found
      ? no(state, `nothing ${state.group?.name ?? 'here'} lends is ${lend}`)
      : go(state, {}, {}, { run: 'unuse', group: state.group, l: found, app, failed: 'unstopped' }),
  unused: (state) => asking(state),
  unstopped: (state, message) =>
    go(
      state,
      {},
      {
        unstopped: {
          ...state.unstopped,
          [message.of]: `${state.unstopped?.[message.of] ?? ''}${message.said ?? message.why}`,
        },
      },
    ),
};

// The Fold, for web/page/acts.js's "Opens friends’ things as you", which
// draws it only where a lend of `lent` asks (its :243), so `lent` is read.
/** @param {Props} props */
export const Used = ({ s: state, send }) =>
  html`<${Fold} id="uses" summary="What apps use" fold="uses" open=${state.using} send=${send}><ul id="used">${Uses({ lent: /** @type {Lent[]} */ (state.lent), labelOf: (/** @type {Lent} */ lend) => lend.label, choices: state.choices ?? {}, tag: 'li', said: state.unstopped, send })}</ul><//>`;
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  uses: ({ s: state }) =>
    !state.lent?.some((lend) => lend.asks) &&
    html`<details id="uses" hidden><summary>What apps use</summary><ul id="used"></ul></details>`,
};

// The edge: a lender's door dialled as the person, its words when it does
// not answer or refuses (app.js@19b63b2b:288-296), read whole before the dial closes, as
// a body after its head is cut off by the close (bash-film-load).
const trace = (/** @type {string} */ line) => globalThis.__trace?.(line);
const saying = (/** @type {string} */ said) => Object.assign(new Error(said), { said });
/**
 * @param {Group} group
 * @param {Lent} lend
 * @param {{ app: string, said: string }} [body] posted, else the choices are asked
 * @param {string} [away] said after a dial not answered
 */
const door = (group, lend, body, away = '') =>
  dialled(group, lend)
    .catch((error) =>
      Promise.reject(
        (trace(`PAGE: the dial failed: ${error}`),
        saying(`${lend.person}’s computer did not answer.${away}`)),
      ),
    )
    .then(async (connection) => {
      // With a body, `bytes` holds it, so each read of it below is under `body ?`.
      const bytes = body && new TextEncoder().encode(JSON.stringify(body));
      const head = body
        ? [
            ['content-type', 'application/json'],
            ['content-length', String(/** @type {Uint8Array<ArrayBuffer>} */ (bytes).length)],
          ]
        : [['accept', 'application/json']];
      /** @type {[boolean, number, string]} */
      const response = await connection
        .fetch(
          body ? 'POST' : 'GET',
          `/choices/${lend.label}`,
          head,
          body ? new Blob([/** @type {Uint8Array<ArrayBuffer>} */ (bytes)]).stream() : null,
        )
        // The wasm's `fetch` answers the page's `Response` (svrngs_web.d.ts `Dialled`).
        .then(
          async (/** @type {Response} */ res) =>
            /** @satisfies {[boolean, number, string]} */ ([res.ok, res.status, await res.text()]),
        )
        .finally(() => connection.close());
      return response[0]
        ? JSON.parse(response[2] || 'null')
        : Promise.reject(
            (trace(`PAGE: /choices/${lend.label} answered ${response[1]}: ${response[2]}`),
            saying(
              body
                ? `${lend.person}’s computer did not keep your answer.`
                : `answered ${response[1]}`,
            )),
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
