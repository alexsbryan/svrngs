// SPDX-License-Identifier: AGPL-3.0-or-later
// A message to a feature of the home page, by `message.flow`, its file's
// name in home/page/, taken by that file's `take` or refused by name. HOME
// (home.js) and each group's GROUP (group-reading.js) offer a message no
// case of theirs takes.
import { no } from '../ui/machine.js?v=__VERSION__';
/** @import { Effect as AnyEffect, Step } from '../ui/machine.js?v=__VERSION__' */

/**
 * A message to a feature, `flow` its file's name in home/page/, which that
 * file's `take` takes (home.js).
 * @typedef {{ type: string, flow: string }} Taken
 */

/**
 * Each feature's module, by its file's name, as home.js imports them: a
 * file that takes a message exports `take`, a step of the machine `S`
 * whose effects are `E`. A method, so a `take` of one message's type
 * stands for any.
 * @template S
 * @template {AnyEffect} [E=AnyEffect]
 * @typedef {Record<string, { take?(s: S, m: Taken): Step<S, E> }>} Flows
 */

/**
 * @template S
 * @template {AnyEffect} E
 * @param {Flows<S, E>} flows
 * @param {S} state
 * @param {Taken} message
 * @returns {Step<S, E>}
 */
export function offered(flows, state, message) {
  const flow = flows[message.flow];
  if (flow?.take) return flow.take(state, message);
  return no(state, `nothing here takes ${message.type}`);
}
