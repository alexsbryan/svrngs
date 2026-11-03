// SPDX-License-Identifier: AGPL-3.0-or-later
// The index of what the group lends, and opening one (page-index (a)):
// web/app.js's list (:530-541), a line each, and `open` (:227-284). Each
// line is its open, named, then what opening it agrees to (`agreed`),
// " Full now." when it is, and on a fine pointer the address to keep;
// drawn from state's `lent`, so each read draws it again. `open` takes a
// lend's label and, from `#open=` (page-join), the path to go to, never a
// closure's lend. The pass, the holder and the hand-off are the edge's
// (web/page/hand.js).
// A guest's lends are drawn here too (web/page/visit.js `listed`).
import { html } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { Keep, Row } from '../ui/parts.js?v=__VERSION__';
import { agreed, alias, aliased, named, once, opens } from '../question.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { handing, here } from './hand.js?v=__VERSION__';
import { listed } from './visit.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Hand } from './hand.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Props, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): the label of the lend opening, and
 * why it did not open.
 * @typedef {{ opening?: string, unopened?: string }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'open', label: string, at?: string }
 *   | { type: 'handed' }
 *   | { type: 'unopened', said?: string, why?: string }} Message
 */

/**
 * The effect its steps answer: the open, handed.
 * @typedef {Hand & { run: 'hand', group: Group, failed: string }} Effect
 */

export const owns = ['opening', 'unopened'];
export const states = /** @type {const} */ ({ open: ['shut', 'opening', 'handed', 'refused'] });
/** @type {Takes<State, Message, Effect>} */
export const takes = {
  open: (state, { label, at }, lend = state.lent?.find((one) => one.label === label)) => {
    if (!lend)
      return no(
        state,
        `nothing ${/** @type {Partial<State>} */ (state).group?.name ?? 'here'} lends is ${label}`,
      );
    if (state.at.open === 'opening') return no(state, `${state.opening} is opening already`);
    return go(
      state,
      { open: 'opening' },
      { opening: label, unopened: '' },
      {
        run: 'hand',
        group: state.group,
        // `lend` is found in it, so the list is read.
        lent: /** @type {Lent[]} */ (state.lent),
        unanswered: state.unanswered,
        label,
        at,
        failed: 'unopened',
      },
    );
  },
  handed: (state) =>
    state.at.open === 'opening' ? go(state, { open: 'handed' }) : no(state, 'nothing is opening'),
  // Opening, the lend opening is one listed (`open`).
  unopened: (
    state,
    message,
    lend = /** @type {Lent} */ (
      /** @type {Lent[]} */ (state.lent).find((one) => one.label === state.opening)
    ),
  ) =>
    state.at.open === 'opening'
      ? go(
          state,
          { open: 'refused' },
          { unopened: message.said ?? `${named(lend)} did not open: ${message.why}` },
        )
      : no(state, 'nothing is opening'),
};
// "Opening …" until the page has gone, or why it did not (app.js@19b63b2b:238).
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  // Refused, it says why (`unopened`).
  enter: (state, lend = state.lent?.find((one) => one.label === state.opening)) =>
    state.at.open === 'refused'
      ? [/** @type {string} */ (state.unopened)]
      : lend && state.at.open !== 'shut' && [`Opening ${named(lend)}…`],
};

// The address to keep for `lend` on this page, `here` (svrn-docs
// rings/current/IN_THE_BROWSER.md §4): its alias, or, where two share one,
// its label, which never collides; the alias as the page reads it, a space
// as `+` (app.js@19b63b2b:530).
// Drawn only once the list is read (`view`).
/**
 * @param {State} state
 * @param {Lent} lend
 * @param {string} [name]
 */
const kept = (state, lend, name = alias(lend, /** @type {Lent[]} */ (state.lent), state.people)) =>
  `${here()}#open=${aliased(name, /** @type {Lent[]} */ (state.lent), state.people).length === 1 ? encodeURIComponent(name).replaceAll('%20', '+').replaceAll('%40', '@') : lend.label}`;
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  lent: ({ s: state, send }) =>
    html`<table class="index"><tbody id="lent">${listed(state, send)}${once(state.lent ?? []).map((lend) => html`<${Row} key=${lend.label} cells=${[html`<span><button onClick=${() => send({ type: 'open', label: lend.label })}>${named(lend)}</button>${` ${agreed(lend, state.shelf, state.group)}${opens(lend)[1] === 'full' ? ' Full now.' : ''}`}${state.fine && html`${' '}<${Keep} at=${kept(state, lend)} />`}</span>`]} />`)}</tbody></table>`,
};

/** @type {Io<Effect, PageMessage>} */
export const io = {
  hand: (message) => store.unseal(message.group).then((key) => handing(message, key)),
};
