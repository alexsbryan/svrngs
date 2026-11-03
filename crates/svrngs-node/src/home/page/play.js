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
import {
  Act,
  ask,
  assertNever,
  go,
  html,
  late,
  machine,
  no,
  useEffect,
  useStore,
} from '../ui/kit.js?v=__VERSION__';
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
 * port's refusals and what its press copied, by `k`, what has Play on,
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
 * carry no read's number: `old` reads `m.n` of every message.
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
    /** @param {number} n */
    (n) =>
      /**
       * @param {Playing[]} plays
       * @returns {Answered}
       */
      (plays) => ({ type: 'answered', n, plays, why: `${plays.length} on` }),
  same = /** @param {Port} a */ (a) => /** @param {Port} b */ (b) =>
    a.key === b.key && a.name === b.name,
  k = /** @param {Port} l */ (l) => `${l.key}:${l.name}`;
// Where a program here connects to what Play opened, copied, or why not.
/** @param {Playing | undefined} r */
const copy = (r) =>
  r?.open.is === 'at'
    ? navigator.clipboard.writeText(`127.0.0.1:${r.port}`).then(
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
    read: ({ n }) => get('/v1/play').then(answer(n)),
    turn: ({ n, l, off }) =>
      post('/v1/play', { [off ? 'off' : 'on']: l }).then(async (plays) => ({
        ...answer(n)(plays),
        copied: off ? '' : await copy(plays.find(same(l))),
      })),
  },
  // Each message's read numbered; an answer older than the last read asked
  // refused by name, and a read or a turn while a turn is under way, before
  // any case takes them.
  step: (s, m) => {
    const [asked, n] = ask(s, 'play');
    const old = m.n && late(s, 'play', m.n);
    if (old) return no(s, old);
    if (s.at.play === 'turning' && ['read', 'turn'].includes(m.type))
      return no(s, 'a turn is under way, and its answer is what has Play on');
    // A refusal, and an answer with what it copied, answer a turn, which
    // set `s.l`.
    switch (m.type) {
      case 'read':
        return m.round <= s.round
          ? no(s, `round ${m.round} is read already`)
          : go(
              s,
              { play: 'reading' },
              { asked, round: m.round },
              { run: 'read', n, failed: 'unread' },
            );
      case 'turn':
        return go(
          s,
          { play: 'turning' },
          { asked, was: s.at.play, l: m.l },
          { run: 'turn', n, l: m.l, off: m.off, failed: 'refused' },
        );
      case 'refused':
        return go(
          s,
          { play: s.was },
          {
            said: {
              ...s.said,
              [k(/** @type {Port} */ (s.l))]:
                `${s.said[k(/** @type {Port} */ (s.l))] ?? ''} ${m.why}`,
            },
          },
        );
      case 'answered':
        return go(
          s,
          { play: 'read' },
          {
            plays: m.plays,
            copied:
              m.copied === undefined
                ? s.copied
                : { ...s.copied, [k(/** @type {Port} */ (s.l))]: m.copied },
          },
        );
      case 'unread':
        return go(s, { play: 'unread' }, { why: m.why });
      default:
        return assertNever(s, m, 'Play');
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
 * @param {State} s
 * @param {Port} l
 */
const on = (s, l) => (s.at.play === 'unread' ? undefined : s.plays?.find(same(l)));

// A port's Play or Stop, in either list, drawn from PLAY alone: what has
// Play on said, with what its press copied, and why a turn was refused.
/**
 * A port's Play or Stop: the port, and, in "What acts as you", what is
 * said of it before its Stop.
 * @typedef {{ l: Port, words?: (l: Port) => string }} TurnProps
 */

/** @param {TurnProps} props */
const Turn = ({ l, words }) =>
  ((s, r = on(s, l)) =>
    html`<${words ? 'p' : 'span'}>${words ? words(l) : `${r ? `${r.said}${s.copied[k(l)] ?? ''}` : 'Play opens it on this computer, for a program here to connect to.'} `}<${Act} words=${r ? 'Stop' : 'Play'} doing=${{ state: s.at.play === 'turning' ? 'busy' : 'idle' }} on=${{ type: 'turn', l: { key: l.key, name: l.name }, off: Boolean(r) }} send=${play.dispatch} />${s.said[k(l)]}<//>`)(
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
export const cell = ({ l, s }) => (
  useEffect(() => play.dispatch({ type: 'read', round: s.asked?.lends ?? 1 }), [s.asked?.lends]),
  html`<${Turn} l=${l} />`
);

// Each Play on, for acts.js, with its Stop, or why what has Play on cannot
// be read; none when nothing has. A hook: acts.js calls it on each of its
// draws, so it is drawn again with each read.
/** @param {Port} r */
const stop = (r) =>
  `Open on this computer: ${r.name}${r.person ? ` (${r.person}’s)` : ''}, while Play is on. `;
/**
 * @param {State} s
 * @returns {Drawn[]}
 */
export const plays = (s) =>
  s.at.play === 'unread'
    ? [html`<p>${s.why}</p>`]
    : (s.plays ?? []).map((r) => html`<${Turn} key=${k(r)} l=${r} words=${stop} />`);
export const part = () => plays(useStore(play));
