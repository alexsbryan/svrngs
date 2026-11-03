// SPDX-License-Identifier: AGPL-3.0-or-later
// Play (page-home-rooms (b); home.js `play` and its Stops, :82-94,
// :382-384; give-play, play.rs): a port a member lends, UDP or TCP, opened
// on this computer while Play is on, for a program here to connect to,
// said as /v1/play says it (`Open::said`), its address copied as it is
// turned on, the clipboard at the edge. PLAY, the page's one read of
// /v1/play, asked as this file loads and again once a round of the groups'
// reads, each read counted, so an older answer is refused by name, feeds
// both of its lists: the index's cell for each port (`cell`, for
// index.js), and each Play on with its Stop under "What acts as you"
// (`part`, for acts.js). What has Play on that cannot be read is said
// once, in `part`, never read as none; Play then writes it over.
import { html, useEffect, useStore } from '../ui/view.js?v=__VERSION__';
import { ask, assertNever, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act } from '../ui/parts.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as GROUP from './group.js?v=__VERSION__' */

/**
 * A port another computer lends, by that computer's key and the lend's
 * name, and its lender's name when it is said.
 * @typedef {{ key: string, name: string, person?: string | null }} Port
 */

/**
 * A port with Play on, as /v1/play answers it (play.rs `Playing`): where
 * it is open or why not (play.rs `Open`, by `is`), and that said.
 * @typedef {object} Playing
 * @property {string | null} person
 * @property {string} key
 * @property {string} name
 * @property {number} port
 * @property {{ is: 'at', words: string | null } | { is: 'apart' | 'unread' | 'taken' } | { is: 'unopened', words: string }} open
 * @property {string} said
 */

/**
 * PLAY's state: where its read or turn is, the round last read, each
 * port's refusals and what its press copied, by `keyOf`, what has Play on,
 * and, while a turn is under way, where it was and the port it turns.
 * @typedef {object} State
 * @property {{ play: 'reading' | 'read' | 'unread' | 'turning' }} at
 * @property {Record<string, number>} [asked]
 * @property {number} round
 * @property {Record<string, string>} said
 * @property {Record<string, string>} copied
 * @property {Playing[]} [plays]
 * @property {State['at']['play']} [was]
 * @property {Port} [l]
 * @property {string} [why]
 */

/**
 * What has Play on, read or turned `n`, and what a turn on copied.
 * @typedef {{ type: 'answered', n: number, plays: Playing[], why: string, copied?: string }} Answered
 */

/**
 * The messages PLAY takes: a round's read; a Play or Stop pressed; what
 * has Play on; or a read or turn that failed, with why. A read and a turn
 * carry no read's number: `old` reads `message.n` of every message.
 * @typedef {{ type: 'read', round: number, n?: undefined }
 *   | { type: 'turn', l: Port, off: boolean, n?: undefined }
 *   | Answered
 *   | { type: 'unread', n: number, why: string }
 *   | { type: 'refused', n: number, why: string }} Message
 */

/**
 * The effects its step answers: read `n` of what has Play on, and turn
 * `n`, Play on or off for the port `l`.
 * @typedef {{ run: 'read', n: number, failed: 'unread' }
 *   | { run: 'turn', n: number, l: Port, off: boolean, failed: 'refused' }} Effect
 */

const answer =
  /** @param {number} serial */
  (serial) =>
    /**
     * @param {Playing[]} plays
     * @returns {Answered}
     */
    (plays) => ({ type: 'answered', n: serial, plays, why: `${plays.length} on` });
const same = /** @param {Port} one */ (one) => /** @param {Port} other */ (other) =>
  one.key === other.key && one.name === other.name;
const keyOf = /** @param {Port} lend */ (lend) => `${lend.key}:${lend.name}`;
// Where a program here connects to what Play opened, copied, or why not.
/** @param {Playing | undefined} playing */
const copy = (playing) =>
  playing?.open.is === 'at'
    ? navigator.clipboard.writeText(`127.0.0.1:${playing.port}`).then(
        () => ' Copied.',
        () => ' Copy it from here.',
      )
    : '';
// A turn, a cell's Play or a Stop, is PLAY's: answered by what has Play on
// then, written over what was read, with what it copied kept for its lend
// (`copied`), and each refusal appended to its lend's (`said`), so every
// list draws it.
/** @type {Machine<State, Message, Effect>} */
export const PLAY = {
  states: { play: ['reading', 'read', 'unread', 'turning'] },
  io: {
    read: ({ n: serial }) => get('/v1/play').then(answer(serial)),
    turn: ({ n: serial, l: lend, off }) =>
      post('/v1/play', { [off ? 'off' : 'on']: lend }).then(async (plays) => ({
        ...answer(serial)(plays),
        copied: off ? '' : await copy(plays.find(same(lend))),
      })),
  },
  // Each message's read numbered; an answer older than the last read asked
  // refused by name, and a read or a turn while a turn is under way, before
  // any case takes them.
  step: (state, message) => {
    const [asked, serial] = ask(state, 'play');
    const old = message.n === undefined ? '' : late(state, 'play', message.n);
    if (old) return no(state, old);
    if (state.at.play === 'turning' && ['read', 'turn'].includes(message.type))
      return no(state, 'a turn is under way, and its answer is what has Play on');
    // A refusal, and an answer with what it copied, answer a turn, which
    // set `state.l`.
    switch (message.type) {
      case 'read':
        return message.round <= state.round
          ? no(state, `round ${message.round} is read already`)
          : go(
              state,
              { play: 'reading' },
              { asked, round: message.round },
              { run: 'read', n: serial, failed: 'unread' },
            );
      case 'turn':
        return go(
          state,
          { play: 'turning' },
          { asked, was: state.at.play, l: message.l },
          { run: 'turn', n: serial, l: message.l, off: message.off, failed: 'refused' },
        );
      case 'refused':
        return go(
          state,
          { play: state.was },
          {
            said: {
              ...state.said,
              [keyOf(/** @type {Port} */ (state.l))]:
                `${state.said[keyOf(/** @type {Port} */ (state.l))] ?? ''} ${message.why}`,
            },
          },
        );
      case 'answered':
        return go(
          state,
          { play: 'read' },
          {
            plays: message.plays,
            copied:
              message.copied === undefined
                ? state.copied
                : { ...state.copied, [keyOf(/** @type {Port} */ (state.l))]: message.copied },
          },
        );
      case 'unread':
        return go(state, { play: 'unread' }, { why: message.why });
      default:
        return assertNever(state, message, 'Play');
    }
  },
};
const play = machine(
  'PLAY',
  PLAY,
  { at: { play: 'reading' }, asked: { play: 1 }, round: 1, said: {}, copied: {} },
  [{ run: 'read', n: 1, failed: 'unread' }],
);
/**
 * @param {State} state
 * @param {Port} lend
 */
const on = (state, lend) =>
  state.at.play === 'unread' ? undefined : state.plays?.find(same(lend));

// A port's Play or Stop, in either list, drawn from PLAY alone: what has
// Play on said, with what its press copied, and why a turn was refused.
/**
 * A port's Play or Stop: the port, and, in "What acts as you", what is
 * said of it before its Stop.
 * @typedef {{ l: Port, words?: (l: Port) => string }} TurnProps
 */

/** @param {TurnProps} props */
const Turn = ({ l: lend, words }) =>
  ((state, open = on(state, lend)) =>
    html`<${words ? 'p' : 'span'}>${words ? words(lend) : `${open ? `${open.said}${state.copied[keyOf(lend)] ?? ''}` : 'Play opens it on this computer, for a program here to connect to.'} `}<${Act} words=${open ? 'Stop' : 'Play'} doing=${{ state: state.at.play === 'turning' ? 'busy' : 'idle' }} on=${{ type: 'turn', l: { key: lend.key, name: lend.name }, off: Boolean(open) }} send=${play.dispatch} />${state.said[keyOf(lend)]}<//>`)(
    useStore(play),
  );
// The index's cell for a port another computer lends (home.js@19b63b2b:182), asking
// PLAY again on each round of its group's reads.
/**
 * The index's cell: the port, and its group's GROUP state, whose lends'
 * read's number is the round.
 * @typedef {{ l: Port, s: GROUP.State }} CellProps
 */

/** @param {CellProps} props */
export const cell = ({ l: lend, s: state }) => {
  useEffect(
    () => play.dispatch({ type: 'read', round: state.asked?.lends ?? 1 }),
    [state.asked?.lends],
  );
  return html`<${Turn} l=${lend} />`;
};

// Each Play on, for acts.js, with its Stop, or why what has Play on cannot
// be read; none when nothing has. A hook: acts.js calls it on each of its
// draws, so it is drawn again with each read.
/** @param {Port} port */
const stop = (port) =>
  `Open on this computer: ${port.name}${port.person ? ` (${port.person}’s)` : ''}, while Play is on. `;
/**
 * @param {State} state
 * @returns {Drawn[]}
 */
export const plays = (state) =>
  state.at.play === 'unread'
    ? [html`<p>${state.why}</p>`]
    : (state.plays ?? []).map(
        (port) => html`<${Turn} key=${keyOf(port)} l=${port} words=${stop} />`,
      );
export const part = () => plays(useStore(play));
