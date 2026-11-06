// SPDX-License-Identifier: AGPL-3.0-or-later
// The index of what the group lends (lent-index.js), and opening one.
// `open` takes a lend's label and, from the link's `#open=`, the path to
// go to, never a closure over the lend, since a message is data (svrn-docs
// rings/current/PAGE_STATE.md §3). The pass, the holder and the hand-off
// are the edge's (hand.js).
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { named } from '../question.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { handing } from './hand.js?v=__VERSION__';
import { LentIndex } from './lent-index.js?v=__VERSION__';
import { notLent } from './page.js?v=__VERSION__';
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

/**
 * The lend the page is opening, or opened last.
 * @param {State} state
 */
const opening = (state) => state.lent?.find((one) => one.label === state.opening);

/** @type {Takes<State, Message, Effect>} */
export const takes = {
  open: (state, { label, at }) => {
    const lend = state.lent?.find((one) => one.label === label);
    if (!lend) return notLent(state, label);
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
  unopened: (state, message) => {
    if (state.at.open !== 'opening') return no(state, 'nothing is opening');
    // Opening, the lend opening is one listed (`open`).
    const lend = /** @type {Lent} */ (opening(state));
    const said = message.said ?? `${named(lend)} did not open: ${message.why}`;
    return go(state, { open: 'refused' }, { unopened: said });
  },
};

// "Opening …" until the page has gone, or, refused, why it did not open
// (`unopened`).
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  enter: (state) => {
    if (state.at.open === 'refused') return [/** @type {string} */ (state.unopened)];
    const lend = opening(state);
    return lend && state.at.open !== 'shut' && [`Opening ${named(lend)}…`];
  },
};

/** @type {Io<Effect, PageMessage>} */
export const io = {
  hand: (message) => store.unseal(message.group).then((key) => handing(message, key)),
};

/** @type {Record<string, (props: Props) => Drawn>} */
export const view = { lent: LentIndex };
