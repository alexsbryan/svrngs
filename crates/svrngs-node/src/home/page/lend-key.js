// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend told from another by its lender's computer's key and its name,
// the two parts /v1/play and /v1/room each take to turn one (play.rs
// `PlayAsk`, room.rs `RoomAsk`). PLAY and ROOM each find a lend by them,
// and keep its refusals under `keyOf`, its key in a list Preact draws too.

/**
 * A lend by its lender's computer's key and its name.
 * @typedef {{ key: string, name: string }} LendKey
 */

/** @param {LendKey} one */
export const same = (one) => /** @param {LendKey} other */ (other) =>
  one.key === other.key && one.name === other.name;

/** @param {LendKey} lend */
export const keyOf = (lend) => `${lend.key}:${lend.name}`;
