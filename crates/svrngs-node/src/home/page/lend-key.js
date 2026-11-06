// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend told from another by its lender's computer's key and its name,
// the two parts /v1/play and /v1/room each take to turn one (play.rs
// `PlayAsk`, room.rs `RoomAsk`). PLAY and ROOM each find a lend by them,
// and keep its refusals under `keyOf`, its key in a list Preact draws too,
// and each is the one step `turns` makes.
import { ask, go, late, no } from '../ui/machine.js?v=__VERSION__';

/**
 * A lend by its lender's computer's key and its name.
 * @typedef {{ key: string, name: string }} LendKey
 */

/** @param {LendKey} one */
export const same = (one) => /** @param {LendKey} other */ (other) =>
  one.key === other.key && one.name === other.name;

/** @param {LendKey} lend */
export const keyOf = (lend) => `${lend.key}:${lend.name}`;

/**
 * The step of a list a lend is turned on and off in, PLAY's and ROOM's, in
 * `region`: each message's read numbered; an answer older than the last read
 * asked refused by name, and a read or a turn while a turn is under way,
 * before any case takes them. A refusal answers a turn, which set `state.l`:
 * `refused` words what its lend keeps. `answered` writes what was read over
 * the state. `busy` says what a turn's answer is, `taker` whose step it is.
 * @param {string} region
 * @param {{ busy: string, taker: string, refused: (said: string | undefined, why: string) => string, answered: (state: any, message: any) => any }} own
 * @returns {(state: any, message: any) => any}
 */
export const turns =
  (region, { busy, taker, refused, answered }) =>
  (state, message) => {
    const [asked, serial] = ask(state, region);
    const old = message.n === undefined ? '' : late(state, region, message.n);
    if (old) return no(state, old);
    if (state.at[region] === 'turning' && ['read', 'turn'].includes(message.type))
      return no(state, `a turn is under way, and its answer is ${busy}`);
    switch (message.type) {
      case 'read':
        if (message.round <= state.round)
          return no(state, `round ${message.round} is read already`);
        return go(
          state,
          { [region]: 'reading' },
          { asked, round: message.round },
          { run: 'read', n: serial, failed: 'unread' },
        );
      case 'turn':
        return go(
          state,
          { [region]: 'turning' },
          { asked, was: state.at[region], l: message.l },
          { run: 'turn', n: serial, l: message.l, off: message.off, failed: 'refused' },
        );
      case 'refused': {
        const key = keyOf(state.l);
        const said = refused(state.said[key], message.why);
        return go(state, { [region]: state.was }, { said: { ...state.said, [key]: said } });
      }
      case 'answered':
        return answered(state, message);
      case 'unread':
        return go(state, { [region]: 'unread' }, { why: message.why });
      default:
        return no(state, `${taker} takes no ${message.type}`);
    }
  };
