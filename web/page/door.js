// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's door, its removals and who is visiting, each drawn in its
// place once the group is read (enter.js): #door (group-door.js),
// #removals (removals.js) and #visitors (visitors.js).
//
// Its machine waits for the earliest visit listed to end, then has the
// group read again (enter.js `reread`), so a visit drops off the page as
// it ends rather than at the next open. One end is waited for at a time.
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { when } from './dial.js?v=__VERSION__';
import { GroupDoor } from './group-door.js?v=__VERSION__';
import { Removals } from './removals.js?v=__VERSION__';
import { Visitors } from './visitors.js?v=__VERSION__';
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Message as PageMessage, Props, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): the end of a visit waited for, in
 * seconds.
 * @typedef {{ ending?: number }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'ends', until: number }} Message
 */

/**
 * The effect its step answers: the end of a visit, waited for.
 * @typedef {{ run: 'end', until: number, failed: string }} Effect
 */

export const owns = ['ending'];

/** @type {Takes<State, Message, Effect>} */
export const takes = {
  ends: (state, { until }) =>
    until === state.ending
      ? no(state, `the end at ${until} is waited for already`)
      : go(state, {}, { ending: until }, { run: 'end', until, failed: 'failed' }),
};

/** @type {Io<Effect, PageMessage>} */
export const io = {
  end: ({ until }) => when(until, { type: 'reread', why: `a visit ended at ${until}` }),
};

/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  door: GroupDoor,
  removals: Removals,
  visitors: Visitors,
};
