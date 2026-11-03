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
import { ask, Fold, go, html, late, no, Uses } from '../ui/kit.js?v=__VERSION__';
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
 * @param {State} s
 * @param {Partial<State>} [patch]
 */
const asking = (s, patch = {}, [asked, n] = ask(s, 'uses')) =>
  go(
    s,
    { uses: 'asking' },
    { ...patch, asked, choices: {} },
    // Folded open only where a lend asks (`view`), so once the list is read.
    .../** @type {Lent[]} */ (s.lent)
      .filter((l) => l.asks)
      .map(
        (l) =>
          /** @satisfies {Effect} */ ({ run: 'choices', group: s.group, l, n, failed: 'chosen' }),
      ),
  );
/** @type {Takes<State, Message, Effect>} */
export const takes = {
  'fold uses': (s, m) =>
    m.open ? asking(s, { using: true }) : go(s, { uses: 'shut' }, { using: false }),
  chosen: (
    s,
    m,
    choices = { ...s.choices, [m.lend]: m.choices ? { choices: m.choices } : { why: m.why } },
  ) =>
    late(s, 'uses', m.n)
      ? no(s, late(s, 'uses', m.n))
      : // Asked of each lend that asks (`asking`), so the list is read.
        go(
          s,
          {
            uses: /** @type {Lent[]} */ (s.lent).every((l) => !l.asks || choices[l.label])
              ? 'asked'
              : 'asking',
          },
          { choices },
        ),
  stop: (s, { lend, app }, l = s.lent?.find((o) => o.label === lend)) =>
    !l
      ? no(s, `nothing ${s.group?.name ?? 'here'} lends is ${lend}`)
      : go(s, {}, {}, { run: 'unuse', group: s.group, l, app, failed: 'unstopped' }),
  unused: (s) => asking(s),
  unstopped: (s, m) =>
    go(
      s,
      {},
      { unstopped: { ...s.unstopped, [m.of]: `${s.unstopped?.[m.of] ?? ''}${m.said ?? m.why}` } },
    ),
};

// The Fold, for web/page/acts.js's "Opens friends’ things as you".
/** @param {Props} props */
export const Used = ({ s, send }) =>
  html`<${Fold} id="uses" summary="What apps use" fold="uses" open=${s.using} send=${send}><ul id="used">${Uses({ lent: s.lent, labelOf: (/** @type {Lent} */ l) => l.label, choices: s.choices ?? {}, tag: 'li', said: s.unstopped, send })}</ul><//>`;
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  uses: ({ s }) =>
    !s.lent?.some((l) => l.asks) &&
    html`<details id="uses" hidden><summary>What apps use</summary><ul id="used"></ul></details>`,
};

// The edge: a lender's door dialled as the person, its words when it does
// not answer or refuses (app.js@19b63b2b:288-296), read whole before the dial closes, as
// a body after its head is cut off by the close (bash-film-load).
const trace = (/** @type {string} */ l) => globalThis.__trace?.(l),
  saying = (/** @type {string} */ said) => Object.assign(new Error(said), { said });
/**
 * @param {Group} group
 * @param {Lent} l
 * @param {'GET' | 'POST'} method
 * @param {{ app: string, said: string }} [body] posted, else the choices are asked
 * @param {string} [away] said after a dial not answered
 */
const door = (group, l, method, body, away = '') =>
  dialled(group, l)
    .catch((e) =>
      Promise.reject(
        (trace(`PAGE: the dial failed: ${e}`),
        saying(`${l.person}’s computer did not answer.${away}`)),
      ),
    )
    .then(async (d) => {
      // With a body, `bytes` holds it, so each read of it below is under `body ?`.
      const bytes = body && new TextEncoder().encode(JSON.stringify(body)),
        head = body
          ? [
              ['content-type', 'application/json'],
              ['content-length', String(/** @type {Uint8Array<ArrayBuffer>} */ (bytes).length)],
            ]
          : [['accept', 'application/json']];
      const r = await d
        .fetch(
          method,
          `/choices/${l.label}`,
          head,
          body ? new Blob([/** @type {Uint8Array<ArrayBuffer>} */ (bytes)]).stream() : null,
        )
        // The wasm's `fetch` answers the page's `Response` (svrngs_web.d.ts `Dialled`).
        .then(
          async (/** @type {Response} */ res) =>
            /** @satisfies {[boolean, number, string]} */ ([res.ok, res.status, await res.text()]),
        )
        .finally(() => d.close());
      return r[0]
        ? JSON.parse(r[2] || 'null')
        : Promise.reject(
            (trace(`PAGE: /choices/${l.label} answered ${r[1]}: ${r[2]}`),
            saying(body ? `${l.person}’s computer did not keep your answer.` : `answered ${r[1]}`)),
          );
    });
// The person's answer `said` for the app labelled `app` on `l`, for the ask too.
/**
 * @param {Group} group
 * @param {Lent} l
 * @param {string} app
 * @param {'yes' | 'stop'} said
 */
export const choose = (group, l, app, said) =>
  door(group, l, 'POST', { app, said }, ' Try again later.');
/** @type {Io<Effect, PageMessage>} */
export const io = {
  choices: ({ group, l, n }) =>
    door(group, l, 'GET').then(
      (a) => ({
        type: 'chosen',
        n,
        lend: l.label,
        choices: a.choices,
        why: `${l.name}: ${Object.keys(a.choices).length} app(s)`,
      }),
      (e) => ({ type: 'chosen', n, lend: l.label, why: e.message }),
    ),
  unuse: ({ group, l, app }) =>
    choose(group, l, app, 'stop').then(
      () => ({ type: 'unused', why: `${app} stopped using ${l.name}` }),
      (e) => ({ type: 'unstopped', of: `${l.label} ${app}`, said: e.message, why: e.message }),
    ),
};
